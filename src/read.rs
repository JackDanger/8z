//! High-level archive reading: parse a 7z archive, iterate entries, extract bytes.
//!
//! The two main types are:
//! - [`Archive`] — parsed, owned archive (holds a copy of the raw bytes).
//! - [`ArchiveReader`] — extraction handle borrowed from an [`Archive`].
//!
//! # Example
//!
//! ```rust,no_run
//! use sevenzippy::{Archive, ArchiveReader};
//!
//! let bytes = std::fs::read("archive.7z").unwrap();
//! let archive = Archive::parse(&bytes).unwrap();
//! println!("{} file(s)", archive.file_count());
//!
//! let reader = archive.reader();
//! let content = reader.extract(0).unwrap();
//! ```

use crate::container;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline;

// ── Archive ───────────────────────────────────────────────────────────────────

/// A parsed, owned 7z archive.
///
/// Holds a copy of the original bytes so that [`ArchiveReader`] can extract
/// files without the caller keeping the original `&[u8]` alive.
pub struct Archive {
    /// Parsed container metadata (includes packed_streams inside).
    header: container::Header,
    /// The raw packed data region (bytes 32..end-header-start of the file).
    packed_data: Vec<u8>,
}

impl Archive {
    /// Parse a `.7z` archive from its raw bytes.
    ///
    /// Allocates an owned copy of the packed data so the input slice can be
    /// dropped after this call returns.
    ///
    /// # Errors
    ///
    /// Propagates all errors from [`container::Archive::parse`].
    pub fn parse(bytes: &[u8]) -> SevenZippyResult<Archive> {
        let c = container::Archive::parse(bytes)?;
        Ok(Archive {
            header: c.header,
            packed_data: c.packed_data.to_vec(),
        })
    }

    /// Borrow an [`ArchiveReader`] that can extract file contents.
    pub fn reader(&self) -> ArchiveReader<'_> {
        ArchiveReader { archive: self }
    }

    /// Number of logical files stored in the archive.
    pub fn file_count(&self) -> usize {
        self.header.files.len()
    }

    /// Name of the file at `idx`, or `None` if `idx` is out of range.
    pub fn file_name(&self, idx: usize) -> Option<&str> {
        self.header.files.get(idx).map(|f| f.name.as_str())
    }

    /// Access the parsed container header directly.
    pub fn header(&self) -> &container::Header {
        &self.header
    }
}

// ── ArchiveReader ─────────────────────────────────────────────────────────────

/// Extraction handle borrowed from an [`Archive`].
pub struct ArchiveReader<'a> {
    archive: &'a Archive,
}

impl<'a> ArchiveReader<'a> {
    /// Extract the unpacked bytes for file `idx`.
    ///
    /// For Phase C (Copy-only, one file per folder), `idx` maps directly to
    /// folder `idx`.
    ///
    /// # Errors
    ///
    /// - [`SevenZippyError::InvalidArgument`] if `idx` is out of range.
    /// - [`SevenZippyError::InvalidHeader`] if the archive has no stream metadata.
    /// - Any error from the coder pipeline (truncated data, unsupported coder, …).
    pub fn extract(&self, idx: usize) -> SevenZippyResult<Vec<u8>> {
        let file_count = self.archive.file_count();
        if idx >= file_count {
            return Err(SevenZippyError::invalid_argument(format!(
                "file index {idx} out of range (archive has {file_count} files)"
            )));
        }

        let ms =
            self.archive.header.main_streams.as_ref().ok_or_else(|| {
                SevenZippyError::invalid_header("archive has no main-streams info")
            })?;

        // For Phase C: one folder per file.
        // The folder index equals the file index.
        if idx >= ms.folders.len() {
            return Err(SevenZippyError::invalid_header(format!(
                "file {idx} has no corresponding folder (only {} folders)",
                ms.folders.len()
            )));
        }

        let folder = &ms.folders[idx];

        // Locate the packed bytes for this folder.
        // The packed-stream geometry comes from the PackedStreams (PackInfo).
        let packed_slice = self.packed_slice_for_folder(idx, ms)?;

        pipeline::decode_folder(folder, packed_slice)
    }

    /// Extract all files, returning `(name, bytes)` pairs.
    pub fn extract_all(&self) -> SevenZippyResult<Vec<(String, Vec<u8>)>> {
        (0..self.archive.file_count())
            .map(|i| {
                let name = self.archive.file_name(i).unwrap_or("").to_string();
                let bytes = self.extract(i)?;
                Ok((name, bytes))
            })
            .collect()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Return the packed-data slice for folder `folder_idx`.
    ///
    /// Phase C assumption: each folder corresponds to exactly one packed stream,
    /// and folders are laid out sequentially in the packed-data region starting
    /// at `pack_pos` (usually 0).
    fn packed_slice_for_folder<'b>(
        &self,
        folder_idx: usize,
        ms: &'b container::UnpackedStreams,
    ) -> SevenZippyResult<&'b [u8]>
    where
        'a: 'b,
    {
        // The packed streams geometry comes from header.packed_streams (PackInfo).
        // Build an offset table: for folder i, packed bytes start at
        //   pack_pos + sum(pack_sizes[0..i])
        // and are pack_sizes[i] bytes long.
        let (pack_pos, pack_sizes) = if let Some(ps) = self.archive.header.packed_streams.as_ref() {
            (ps.pack_pos as usize, ps.pack_sizes.clone())
        } else {
            // Fall back: derive from folder unpack sizes (valid for Copy-only archives)
            let sizes: Vec<u64> = ms
                .folders
                .iter()
                .map(|f| f.unpack_sizes.first().copied().unwrap_or(0))
                .collect();
            (0usize, sizes)
        };

        if folder_idx >= pack_sizes.len() {
            return Err(SevenZippyError::invalid_header(format!(
                "folder {folder_idx} has no pack-size entry (only {} entries)",
                pack_sizes.len()
            )));
        }

        let offset: usize = pack_pos
            + pack_sizes[..folder_idx]
                .iter()
                .map(|&s| s as usize)
                .sum::<usize>();
        let size = pack_sizes[folder_idx] as usize;

        if offset + size > self.archive.packed_data.len() {
            return Err(SevenZippyError::truncated(format!(
                "folder {folder_idx} packed data [{offset}..{}] out of range (packed_data len={})",
                offset + size,
                self.archive.packed_data.len()
            )));
        }

        // SAFETY: we just bounds-checked. The slice is from the archive's
        // owned Vec<u8>. We need to extend the lifetime to 'b here, which is
        // sound because 'a: 'b and both &self and ms are borrowed from 'a.
        //
        // Actually, self.archive.packed_data lives for 'a, and 'a: 'b, so
        // returning a &'b slice from a &'a Vec<u8> is fine.
        let full = self.archive.packed_data.as_slice();
        Ok(&full[offset..offset + size])
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest.join("corpora/fixtures/archives").join(name)
    }

    #[test]
    fn extract_copy_only_fixture() {
        let path = fixture_path("copy_only.7z");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.file_count(), 1);
        assert_eq!(archive.file_name(0), Some("8z-fixture-input.txt"));

        let content = archive.reader().extract(0).unwrap();
        assert_eq!(content, b"Hello, 7z umbrella!");
    }

    #[test]
    fn extract_copy_only_64k_fixture() {
        let path = fixture_path("copy_only_64k.7z");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.file_count(), 1);
        assert_eq!(archive.file_name(0), Some("random_64k.bin"));

        let content = archive.reader().extract(0).unwrap();
        assert_eq!(content.len(), 65536);
    }

    #[test]
    fn out_of_range_index_is_error() {
        let path = fixture_path("copy_only.7z");
        let bytes = std::fs::read(&path).unwrap();
        let archive = Archive::parse(&bytes).unwrap();
        let result = archive.reader().extract(1); // only 1 file (index 0)
        assert!(result.is_err());
    }
}
