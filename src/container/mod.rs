//! 7z container parser: signature header, end-header, folders, coders, streams.
//!
//! The top-level entry point is [`Archive::parse`], which takes the raw bytes
//! of a `.7z` file and returns an [`Archive`] struct.
//!
//! ## Parse flow
//!
//! 1. Read the first 32 bytes → [`SignatureHeader`].
//! 2. Validate `start_header_crc` (CRC over bytes 12..32).
//! 3. Locate the end-header block: bytes `[32 + offset .. 32 + offset + size]`.
//! 4. Validate `next_header_crc` against the end-header block.
//! 5. Peek at the first byte of the block:
//!    - `0x01` → parse as plain [`Header`].
//!    - `0x17` → return [`SevenZippyError::not_yet_implemented("encoded header")`].
//! 6. Assemble [`Archive`].

pub mod coders;
pub mod crc;
pub mod folders;
pub mod header;
pub mod properties;
pub mod signature_header;
pub mod streams;

pub use coders::{Coder, MethodId};
pub use folders::{Bond, Folder, UnpackSize};
pub use header::{FileEntry, Header};
pub use properties::PropertyId;
pub use signature_header::SignatureHeader;
pub use streams::{PackedStreams, UnpackedStreams};

use crate::container::crc::crc32;
use crate::error::{SevenZippyError, SevenZippyResult};

// ── Archive ───────────────────────────────────────────────────────────────────

/// A parsed `.7z` archive.
///
/// `'a` is the lifetime of the original input slice; the archive holds a
/// reference into it for the packed (compressed) data bytes so that callers
/// can extract without copying.
pub struct Archive<'a> {
    /// Parsed fixed header.
    pub signature_header: SignatureHeader,
    /// Parsed metadata (end-header).
    pub header: Header,
    /// Raw bytes of packed stream data (from byte 32 of the file through the
    /// start of the end-header block). Callers use this together with
    /// `header.packed_streams` to locate and decompress each folder.
    pub packed_data: &'a [u8],
}

impl<'a> Archive<'a> {
    /// Parse a `.7z` archive from its raw bytes.
    ///
    /// # Errors
    ///
    /// - [`SevenZippyError::Truncated`] if the input is shorter than expected.
    /// - [`SevenZippyError::InvalidSignature`] if the magic or CRCs don't match.
    /// - [`SevenZippyError::InvalidHeader`] if the metadata is malformed.
    /// - [`SevenZippyError::NotYetImplemented`] if the header uses the
    ///   compressed (`EncodedHeader`) form (Phase C limitation).
    pub fn parse(input: &'a [u8]) -> SevenZippyResult<Archive<'a>> {
        // ── Step 1: signature header ──────────────────────────────────────────
        if input.len() < 32 {
            return Err(SevenZippyError::truncated(format!(
                "archive is {} bytes; need at least 32 for the signature header",
                input.len()
            )));
        }
        let sig_raw: &[u8; 32] = input[..32].try_into().unwrap();
        let signature_header = SignatureHeader::parse(sig_raw)?;

        // ── Step 2+3: locate end-header block ─────────────────────────────────
        let offset = signature_header.next_header_offset as usize;
        let size = signature_header.next_header_size as usize;
        let end_header_start = 32usize
            .checked_add(offset)
            .ok_or_else(|| SevenZippyError::invalid_header("next_header_offset overflow"))?;
        let end_header_end = end_header_start
            .checked_add(size)
            .ok_or_else(|| SevenZippyError::invalid_header("next_header_size overflow"))?;

        if input.len() < end_header_end {
            return Err(SevenZippyError::truncated(format!(
                "archive is {} bytes; end-header block needs bytes {end_header_start}..{end_header_end}",
                input.len()
            )));
        }

        let end_header_bytes = &input[end_header_start..end_header_end];

        // ── Step 4: validate end-header CRC ───────────────────────────────────
        let computed_crc = crc32(end_header_bytes);
        if computed_crc != signature_header.next_header_crc {
            return Err(SevenZippyError::invalid_signature(format!(
                "NextHeaderCRC mismatch: stored {:#010x}, computed {computed_crc:#010x}",
                signature_header.next_header_crc
            )));
        }

        // ── Step 5: parse end-header ──────────────────────────────────────────
        let header = header::parse(end_header_bytes)?;

        // ── Step 6: packed data slice ─────────────────────────────────────────
        // The packed data lives between the end of the signature header (byte 32)
        // and the start of the end-header block.
        let packed_data = &input[32..end_header_start];

        Ok(Archive {
            signature_header,
            header,
            packed_data,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        // Walk up from the manifest dir to the workspace root, then into corpora.
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest.join("corpora/fixtures/archives").join(name)
    }

    #[test]
    fn parses_copy_only_fixture() {
        let path = fixture_path("copy_only.7z");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
        let archive =
            Archive::parse(&bytes).unwrap_or_else(|e| panic!("failed to parse copy_only.7z: {e}"));

        // ── Files ─────────────────────────────────────────────────────────────
        assert_eq!(archive.header.files.len(), 1, "expected 1 file entry");
        let f = &archive.header.files[0];
        assert_eq!(f.name, "8z-fixture-input.txt", "file name mismatch");

        // ── Main streams ──────────────────────────────────────────────────────
        let ms = archive
            .header
            .main_streams
            .as_ref()
            .expect("main_streams must be Some");
        assert_eq!(ms.folders.len(), 1, "expected 1 folder");

        let folder = &ms.folders[0];
        assert_eq!(folder.coders.len(), 1, "expected 1 coder");
        assert_eq!(
            folder.coders[0].method_id,
            MethodId::copy(),
            "expected Copy coder (method ID [0x00])"
        );

        // ── Sizes ─────────────────────────────────────────────────────────────
        // File size comes either from SubStreamsInfo or from the folder unpack size.
        let file_size = if !ms.unpack_stream_sizes.is_empty() {
            ms.unpack_stream_sizes[0]
        } else {
            *folder
                .unpack_sizes
                .last()
                .expect("folder must have unpack sizes")
        };
        assert_eq!(file_size, 19, "file size should be 19 bytes");

        // ── Packed data ───────────────────────────────────────────────────────
        assert_eq!(
            archive.packed_data, b"Hello, 7z umbrella!",
            "packed data should be the raw file content"
        );
    }

    #[test]
    fn parses_copy_only_64k_fixture() {
        let path = fixture_path("copy_only_64k.7z");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
        let archive = Archive::parse(&bytes)
            .unwrap_or_else(|e| panic!("failed to parse copy_only_64k.7z: {e}"));

        // Should have exactly 1 file
        assert_eq!(archive.header.files.len(), 1);
        let f = &archive.header.files[0];
        assert_eq!(f.name, "random_64k.bin");

        let ms = archive
            .header
            .main_streams
            .as_ref()
            .expect("main_streams must be Some");
        let folder = &ms.folders[0];

        let file_size = if !ms.unpack_stream_sizes.is_empty() {
            ms.unpack_stream_sizes[0]
        } else {
            *folder
                .unpack_sizes
                .last()
                .expect("folder must have unpack sizes")
        };
        assert_eq!(file_size, 65536, "file size should be 65536 bytes");

        // Packed data should be 65536 bytes
        assert_eq!(archive.packed_data.len(), 65536);

        // Folder has exactly 1 Copy coder
        assert_eq!(folder.coders.len(), 1);
        assert_eq!(folder.coders[0].method_id, MethodId::copy());
    }
}
