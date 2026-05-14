//! High-level archive writing: builder API to produce a 7z file byte-for-byte
//! compatible with the 7z specification (and extractable by `7zz`).
//!
//! # Design
//!
//! `ArchiveBuilder` accumulates files and their coder choices, then [`build`]
//! serialises a complete `.7z` archive:
//!
//! ```text
//! Offset  Content
//!      0  SignatureHeader (32 bytes)
//!     32  Packed data (all files concatenated, after coder encoding)
//!  32+PS  End-header (Header block)
//! ```
//!
//! For Phase C the archive uses one folder per file (the simplest layout).
//!
//! # Format reference
//!
//! § 5.3 of `7zFormat.txt` (<https://github.com/ip7z/7zip/blob/main/DOC/7zFormat.txt>).

use crate::container::crc::crc32;
use crate::container::signature_header::SIGNATURE;
use crate::container::Folder;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::{self, Coder, CopyCoder};

// ── Internal types ────────────────────────────────────────────────────────────

struct EncodedFile {
    name: String,
    packed: Vec<u8>,
    folder: Folder,
    unpacked_crc: u32,
}

// ── ArchiveBuilder ────────────────────────────────────────────────────────────

/// Builder for constructing a new `.7z` archive.
///
/// Add files with [`add_file`](ArchiveBuilder::add_file) or the convenience
/// wrapper [`add_copy_file`](ArchiveBuilder::add_copy_file), then call
/// [`build`](ArchiveBuilder::build) to get the archive bytes.
pub struct ArchiveBuilder {
    files: Vec<BuildEntry>,
}

enum BuildEntry {
    SingleCoder {
        name: String,
        content: Vec<u8>,
        coder: Box<dyn Coder>,
    },
    #[cfg(feature = "aes")]
    Encrypted {
        name: String,
        content: Vec<u8>,
        password: String,
    },
}

impl ArchiveBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        ArchiveBuilder { files: Vec::new() }
    }

    /// Add a file with the given name and content, encoded with `coder`.
    pub fn add_file(
        &mut self,
        name: impl Into<String>,
        content: Vec<u8>,
        coder: Box<dyn Coder>,
    ) -> &mut Self {
        self.files.push(BuildEntry::SingleCoder {
            name: name.into(),
            content,
            coder,
        });
        self
    }

    /// Add a file using the in-tree Copy coder (no compression).
    pub fn add_copy_file(&mut self, name: impl Into<String>, content: Vec<u8>) -> &mut Self {
        self.add_file(name, content, Box::new(CopyCoder))
    }

    /// Add a file with AES-256+LZMA2 encryption and compression.
    ///
    /// The file is first compressed with LZMA2, then encrypted with AES-256-CBC
    /// using the given password. The resulting folder will have 2 coders and 1 bond.
    #[cfg(feature = "aes")]
    pub fn add_encrypted_file(
        &mut self,
        name: impl Into<String>,
        content: Vec<u8>,
        password: impl Into<String>,
    ) -> &mut Self {
        self.files.push(BuildEntry::Encrypted {
            name: name.into(),
            content,
            password: password.into(),
        });
        self
    }

    /// Emit a complete `.7z` archive as a byte vector.
    ///
    /// The output is extractable by the reference `7zz` implementation.
    ///
    /// # Errors
    ///
    /// Propagates any error from the coder pipelines.
    pub fn build(self) -> SevenZippyResult<Vec<u8>> {
        // ── Step 1: encode each file through its coder pipeline ───────────────
        let mut encoded_files: Vec<EncodedFile> = Vec::with_capacity(self.files.len());
        for entry in &self.files {
            match entry {
                BuildEntry::SingleCoder {
                    name,
                    content,
                    coder,
                } => {
                    let (packed, folder) =
                        pipeline::encode_single_coder_folder(coder.as_ref(), content)?;
                    let unpacked_crc = crc32(content);
                    encoded_files.push(EncodedFile {
                        name: name.clone(),
                        packed,
                        folder,
                        unpacked_crc,
                    });
                }
                #[cfg(feature = "aes")]
                BuildEntry::Encrypted {
                    name,
                    content,
                    password,
                } => {
                    let (packed, folder) = pipeline::encode_aes_lzma2_folder(content, password)?;
                    let unpacked_crc = crc32(content);
                    encoded_files.push(EncodedFile {
                        name: name.clone(),
                        packed,
                        folder,
                        unpacked_crc,
                    });
                }
            }
        }

        // ── Step 2: concatenate packed streams ────────────────────────────────
        let mut packed_streams_data: Vec<u8> = Vec::new();
        for ef in &encoded_files {
            packed_streams_data.extend_from_slice(&ef.packed);
        }
        let pack_total = packed_streams_data.len();

        // ── Step 3: build the end-header ──────────────────────────────────────
        let header_bytes = build_header(&encoded_files)?;

        // ── Step 4: signature header ──────────────────────────────────────────
        let next_header_offset = pack_total as u64;
        let next_header_size = header_bytes.len() as u64;
        let next_header_crc = crc32(&header_bytes);

        let sig_header =
            build_signature_header(next_header_offset, next_header_size, next_header_crc);

        // ── Step 5: concatenate ───────────────────────────────────────────────
        let mut out = Vec::with_capacity(32 + pack_total + header_bytes.len());
        out.extend_from_slice(&sig_header);
        out.extend_from_slice(&packed_streams_data);
        out.extend_from_slice(&header_bytes);

        Ok(out)
    }
}

impl Default for ArchiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Signature header builder ──────────────────────────────────────────────────

fn build_signature_header(offset: u64, size: u64, header_crc: u32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..6].copy_from_slice(&SIGNATURE);
    buf[6] = 0; // version_major
    buf[7] = 4; // version_minor
                // bytes 12..20: NextHeaderOffset
    buf[12..20].copy_from_slice(&offset.to_le_bytes());
    // bytes 20..28: NextHeaderSize
    buf[20..28].copy_from_slice(&size.to_le_bytes());
    // bytes 28..32: NextHeaderCRC
    buf[28..32].copy_from_slice(&header_crc.to_le_bytes());
    // bytes 8..12: StartHeaderCRC = CRC32 of bytes 12..32
    let start_crc = crc32(&buf[12..32]);
    buf[8..12].copy_from_slice(&start_crc.to_le_bytes());
    buf
}

// ── End-header builder ────────────────────────────────────────────────────────

/// Emit one coder record in the 7z header format (7zFormat.txt §5.3.5).
///
/// Flag byte encoding:
///   bits 3:0 = CodecIdSize (1..15)
///   bit  4   = 0 → simple coder (1 in / 1 out); 1 → complex coder
///   bit  5   = 0 → no attributes; 1 → attributes follow
///   bits 7:6 = reserved (0)
fn write_coder_record(h: &mut Vec<u8>, coder: &crate::container::Coder) -> SevenZippyResult<()> {
    let id_bytes = &coder.method_id.0;
    let id_len = id_bytes.len();

    // The 7z header encodes the method-ID length in the low nibble of the flag
    // byte (a 4-bit field). Spec §5.3.5: value 0 is reserved, so valid lengths
    // are 1..=15.
    if id_len == 0 {
        return Err(SevenZippyError::invalid_header(
            "coder method ID must be at least 1 byte",
        ));
    }
    if id_len > 15 {
        return Err(SevenZippyError::invalid_header(format!(
            "coder method ID must be 1..=15 bytes, got {id_len}"
        )));
    }

    let id_size = id_len as u8;
    let has_attrs = !coder.properties.is_empty();
    let is_complex = coder.num_in_streams != 1 || coder.num_out_streams != 1;

    let mut flag: u8 = id_size;
    if is_complex {
        flag |= 0x10;
    }
    if has_attrs {
        flag |= 0x20;
    }

    h.push(flag);
    h.extend_from_slice(id_bytes);

    if is_complex {
        write_uint64(h, coder.num_in_streams);
        write_uint64(h, coder.num_out_streams);
    }

    if has_attrs {
        write_uint64(h, coder.properties.len() as u64);
        h.extend_from_slice(&coder.properties);
    }

    Ok(())
}

fn build_header(files: &[EncodedFile]) -> SevenZippyResult<Vec<u8>> {
    let num_files = files.len();

    if num_files == 0 {
        // Empty archive: Header tag + End tag
        return Ok(vec![0x01, 0x00]);
    }

    // Validate that every folder produces exactly one packed stream.
    //
    // TODO(phase1-closure / BCJ2-encode): generalize for multi-packed-stream
    // folders. Currently every supported encoder (Copy, LZMA, LZMA2, AES+LZMA2)
    // produces exactly one packed stream per folder, so num_pack_streams ==
    // num_files. When BCJ2 encode lands (which produces 4 packed streams per
    // folder), this needs to become sum-of-per-folder-pack-stream-counts and
    // PackSizes needs corresponding generalization. The validation just below
    // (in build_header) enforces this invariant until then.
    for ef in files {
        let folder = &ef.folder;
        let num_in_total: u64 = folder.coders.iter().map(|c| c.num_in_streams).sum();
        let num_bonds = folder.bonds.len() as u64;
        let folder_pack_streams = num_in_total - num_bonds;
        if folder_pack_streams != 1 {
            return Err(SevenZippyError::not_yet_implemented(
                "multi-packed-stream folders not yet supported by the writer; \
                 only single-packed-stream folders (e.g., Copy, LZMA, LZMA2, AES+LZMA2) \
                 can be written",
            ));
        }
    }

    let mut h: Vec<u8> = Vec::new();

    // Header (0x01)
    h.push(0x01);

    // MainStreamsInfo (0x04)
    h.push(0x04);

    // ── PackInfo (0x06) ───────────────────────────────────────────────────────
    // num_pack_streams currently equals num_files because each supported encoder
    // produces exactly one packed stream per file (enforced by the validation
    // above). This is a documented Phase 1 limitation — see the TODO above for
    // how this must change when BCJ2 encode lands.
    h.push(0x06);
    write_uint64(&mut h, 0); // pack_pos = 0
    write_uint64(&mut h, num_files as u64); // num_pack_streams (== num_files; Phase 1 invariant)
                                            // Size (0x09)
    h.push(0x09);
    for ef in files {
        write_uint64(&mut h, ef.packed.len() as u64);
    }
    h.push(0x00); // End (PackInfo)

    // ── UnpackInfo (0x07) ─────────────────────────────────────────────────────
    h.push(0x07);

    // Folder (0x0B)
    h.push(0x0B);
    write_uint64(&mut h, num_files as u64); // num_folders
    h.push(0x00); // external = 0

    // Each folder: serialize NumCoders, each coder, bonds, packed stream indices.
    for ef in files {
        let folder = &ef.folder;
        write_uint64(&mut h, folder.coders.len() as u64);

        for coder in &folder.coders {
            write_coder_record(&mut h, coder)?;
        }

        // Bonds: per spec, NumBindPairs = NumOutStreamsTotal - 1.
        // The count is implicit; we just emit each bond's (InIndex, OutIndex).
        for bond in &folder.bonds {
            write_uint64(&mut h, bond.in_index);
            write_uint64(&mut h, bond.out_index);
        }

        // Packed stream indices: per spec, only emitted when NumPackedStreams > 1.
        // NumPackedStreams = NumInStreamsTotal - NumBindPairs.
        let num_in_total: u64 = folder.coders.iter().map(|c| c.num_in_streams).sum();
        let num_bonds = folder.bonds.len() as u64;
        let num_pack_streams = num_in_total - num_bonds;
        if num_pack_streams > 1 {
            for &idx in &folder.packed_stream_indices {
                write_uint64(&mut h, idx);
            }
        }
        // When num_pack_streams == 1, the index is implicitly 0 (not written).
    }

    // CodersUnpackSize (0x0C): one UINT64 per coder output stream, for each folder.
    // For a 2-coder AES+LZMA2 folder: unpack_sizes = [aes_out_size, lzma2_out_size].
    h.push(0x0C);
    for ef in files {
        for &sz in &ef.folder.unpack_sizes {
            write_uint64(&mut h, sz);
        }
    }

    h.push(0x00); // End (UnpackInfo)

    // ── SubStreamsInfo (0x08) ─────────────────────────────────────────────────
    // We have exactly 1 sub-stream per folder (1 file per folder), so
    // NumUnpackStream is not emitted (the parser defaults it to 1).
    // We only emit the sub-stream CRCs.
    h.push(0x08);

    // Crc (0x0A): one CRC per sub-stream (file), all defined
    h.push(0x0A);
    h.push(0x01); // all_defined = 1
    for ef in files {
        h.extend_from_slice(&ef.unpacked_crc.to_le_bytes());
    }

    h.push(0x00); // End (SubStreamsInfo)

    h.push(0x00); // End (MainStreamsInfo)

    // ── FilesInfo (0x05) ──────────────────────────────────────────────────────
    h.push(0x05);
    write_uint64(&mut h, num_files as u64);

    // Name property (0x11)
    {
        // Encode: BYTE External=0, then UTF-16LE null-terminated names
        let mut name_bytes: Vec<u8> = vec![0x00]; // external = 0
        for ef in files {
            for ch in ef.name.encode_utf16() {
                name_bytes.extend_from_slice(&ch.to_le_bytes());
            }
            name_bytes.push(0x00); // UTF-16LE null terminator low byte
            name_bytes.push(0x00); // UTF-16LE null terminator high byte
        }
        h.push(0x11); // Name property tag
        write_uint64(&mut h, name_bytes.len() as u64);
        h.extend_from_slice(&name_bytes);
    }

    h.push(0x00); // End (FilesInfo)

    h.push(0x00); // End (Header)

    Ok(h)
}

// ── UINT64 variable-length encoder ───────────────────────────────────────────

/// Encode `value` using 7z's variable-length UINT64 encoding and append to `out`.
///
/// Encoding (inverse of `properties::read_uint64`):
/// - 0x00..0x7F: 1 byte (no leading 1-bit)
/// - 0x80..0x3FFF: 2 bytes (1 leading 1-bit; high 6 bits in first byte, low 8 in second)
/// - ...up to 9 bytes for full u64.
pub(crate) fn write_uint64(out: &mut Vec<u8>, value: u64) {
    // Inverse of `properties::read_uint64`.
    //
    // The encoding uses a leading-ones scheme:
    //   n leading 1-bits in first_byte → n extra bytes follow.
    //   The remaining (7-n) bits of first_byte hold the highest bits of value.
    //
    // Thresholds (value_max = 2^(7-n + n*8) - 1 = 2^(7+7n) - 1):
    //   n=0: value < 2^7  = 0x80           → 1 total byte
    //   n=1: value < 2^14 = 0x4000         → 2 bytes
    //   n=2: value < 2^21 = 0x200000       → 3 bytes
    //   n=3: value < 2^28 = 0x10000000     → 4 bytes
    //   n=4: value < 2^35 = 0x800000000    → 5 bytes
    //   n=5: value < 2^42 = 0x40000000000  → 6 bytes
    //   n=6: value < 2^49 = 0x2000000000000 → 7 bytes
    //   n=7: value < 2^56 = 0x100000000000000 → 8 bytes
    //   n=8: first_byte = 0xFF, then 8 LE bytes → 9 bytes (full u64)
    //
    // For each case with n extra bytes:
    //   first_byte = (0xFF << (8-n)) | (high bits: value >> (n*8)) & mask(7-n)
    //   extra_bytes = value as n LE bytes (low n*8 bits)

    // For Phase C archives, all values fit in 3 bytes (max file size < 2^21).
    // We implement the full encoding for correctness.

    let n: u32 = match value {
        v if v < (1 << 7) => 0,
        v if v < (1 << 14) => 1,
        v if v < (1 << 21) => 2,
        v if v < (1 << 28) => 3,
        v if v < (1 << 35) => 4,
        v if v < (1 << 42) => 5,
        v if v < (1 << 49) => 6,
        v if v < (1 << 56) => 7,
        _ => 8,
    };

    if n == 8 {
        // Special 9-byte form: first_byte = 0xFF, then 8 LE bytes
        out.push(0xFF);
        out.extend_from_slice(&value.to_le_bytes());
        return;
    }

    // first_byte:
    //   upper n bits = 1  (leading ones count)
    //   bit (7-n) = 0     (terminator)
    //   lower (7-n) bits  = high bits of value (bits [n*8 + (7-n) - 1 .. n*8])
    let high_bits = (value >> (n * 8)) as u8; // the top (7-n) bits of value
    let leading_ones: u8 = if n == 0 { 0 } else { !(0xFF_u8 >> n) };
    // mask for the low (7-n) bits of first_byte (the terminator bit is 0 by construction)
    let mask: u8 = (0xFF_u8 >> n) >> 1; // (7-n) bits
    let first_byte = leading_ones | (high_bits & mask);
    out.push(first_byte);

    // Then n little-endian bytes (low n*8 bits)
    for i in 0..n {
        out.push((value >> (i * 8)) as u8);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::Archive;

    #[test]
    fn write_uint64_small() {
        let mut out = Vec::new();
        write_uint64(&mut out, 0);
        assert_eq!(out, &[0x00]);

        let mut out = Vec::new();
        write_uint64(&mut out, 19);
        assert_eq!(out, &[0x13]);

        let mut out = Vec::new();
        write_uint64(&mut out, 0x7F);
        assert_eq!(out, &[0x7F]);
    }

    #[test]
    fn write_uint64_two_byte() {
        let mut out = Vec::new();
        write_uint64(&mut out, 0x80);
        assert_eq!(out, &[0x80, 0x80]);

        let mut out = Vec::new();
        write_uint64(&mut out, 0x3FFF);
        assert_eq!(out, &[0xBF, 0xFF]);
    }

    #[test]
    fn write_uint64_three_byte() {
        let mut out = Vec::new();
        write_uint64(&mut out, 0x4000);
        assert_eq!(out, &[0xC0, 0x00, 0x40]);

        // 65536 = 0x10000
        let mut out = Vec::new();
        write_uint64(&mut out, 65536);
        assert_eq!(out, &[0xC1, 0x00, 0x00]);
    }

    #[test]
    fn round_trip_empty_archive() {
        let builder = ArchiveBuilder::new();
        let bytes = builder.build().unwrap();
        // Minimal valid archive: sig(32) + no packed data + header(2 bytes: 0x01 0x00)
        assert_eq!(bytes.len(), 34);
        // Must parse without error
        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.file_count(), 0);
    }

    #[test]
    fn round_trip_one_file() {
        let mut builder = ArchiveBuilder::new();
        builder.add_copy_file("hello.txt", b"Hello, world!".to_vec());
        let bytes = builder.build().unwrap();

        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.file_count(), 1);
        assert_eq!(archive.file_name(0), Some("hello.txt"));

        let extracted = archive.reader().extract(0).unwrap();
        assert_eq!(extracted, b"Hello, world!");
    }

    #[test]
    fn round_trip_two_files() {
        let mut builder = ArchiveBuilder::new();
        builder.add_copy_file("a.txt", b"first".to_vec());
        builder.add_copy_file("b.txt", b"second".to_vec());
        let bytes = builder.build().unwrap();

        let archive = Archive::parse(&bytes).unwrap();
        assert_eq!(archive.file_count(), 2);
        assert_eq!(archive.reader().extract(0).unwrap(), b"first");
        assert_eq!(archive.reader().extract(1).unwrap(), b"second");
    }

    /// Verify that a folder with multiple packed streams (e.g. BCJ2-style with
    /// 4 input streams) is rejected with a `NotYetImplemented` error, not silently
    /// written as a corrupt archive.
    #[test]
    fn multi_pack_stream_folder_is_rejected() {
        use crate::container::{Coder, Folder, MethodId};
        use crate::error::SevenZippyError;

        // Construct a synthetic 2-coder folder that has 2 packed (unbound) input
        // streams — mimicking the BCJ2 topology (4 in-streams, 3 bonds in the
        // real case; here we use 2 in-streams, 1 bond = 1 remaining pack stream is
        // OK, so let's use 2 in-streams, 0 bonds = 2 pack streams which is invalid).
        let coder_a = Coder {
            method_id: MethodId(vec![0x00]), // Copy
            num_in_streams: 2,               // two unbound input streams
            num_out_streams: 1,
            properties: vec![],
        };
        let folder = Folder {
            coders: vec![coder_a],
            bonds: vec![], // 0 bonds → 2 packed streams
            packed_stream_indices: vec![0, 1],
            unpack_sizes: vec![5],
            unpack_crc: None,
        };

        let files = vec![EncodedFile {
            name: "test.bin".to_string(),
            packed: b"hello".to_vec(),
            folder,
            unpacked_crc: 0,
        }];

        let result = build_header(&files);
        match result {
            Err(SevenZippyError::NotYetImplemented(msg)) => {
                assert!(
                    msg.contains("multi-packed-stream"),
                    "error message should mention multi-packed-stream, got: {msg}"
                );
            }
            other => panic!("expected NotYetImplemented, got: {other:?}"),
        }
    }

    /// Verify that a coder with an empty method ID is rejected.
    #[test]
    fn empty_method_id_is_rejected() {
        use crate::container::{Coder, Folder, MethodId};
        use crate::error::SevenZippyError;

        let coder = Coder {
            method_id: MethodId(vec![]), // empty — invalid
            num_in_streams: 1,
            num_out_streams: 1,
            properties: vec![],
        };
        let folder = Folder {
            coders: vec![coder],
            bonds: vec![],
            packed_stream_indices: vec![0],
            unpack_sizes: vec![5],
            unpack_crc: None,
        };

        let files = vec![EncodedFile {
            name: "test.bin".to_string(),
            packed: b"hello".to_vec(),
            folder,
            unpacked_crc: 0,
        }];

        let result = build_header(&files);
        match result {
            Err(SevenZippyError::InvalidHeader(msg)) => {
                assert!(
                    msg.contains("at least 1 byte"),
                    "error message should mention minimum size, got: {msg}"
                );
            }
            other => panic!("expected InvalidHeader, got: {other:?}"),
        }
    }

    /// Verify that a coder with a method ID exceeding 15 bytes is rejected.
    #[test]
    fn oversized_method_id_is_rejected() {
        use crate::container::{Coder, Folder, MethodId};
        use crate::error::SevenZippyError;

        let coder = Coder {
            method_id: MethodId(vec![0u8; 16]), // 16 bytes — exceeds 4-bit nibble max of 15
            num_in_streams: 1,
            num_out_streams: 1,
            properties: vec![],
        };
        let folder = Folder {
            coders: vec![coder],
            bonds: vec![],
            packed_stream_indices: vec![0],
            unpack_sizes: vec![5],
            unpack_crc: None,
        };

        let files = vec![EncodedFile {
            name: "test.bin".to_string(),
            packed: b"hello".to_vec(),
            folder,
            unpacked_crc: 0,
        }];

        let result = build_header(&files);
        match result {
            Err(SevenZippyError::InvalidHeader(msg)) => {
                assert!(
                    msg.contains("1..=15 bytes"),
                    "error message should mention 1..=15 constraint, got: {msg}"
                );
            }
            other => panic!("expected InvalidHeader, got: {other:?}"),
        }
    }
}
