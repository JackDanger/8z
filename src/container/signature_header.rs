//! Parsing the 7z signature header — the fixed 32-byte block at the very start
//! of every `.7z` file.
//!
//! Layout (from 7zFormat.txt):
//!
//! ```text
//! Offset  Size  Field
//!      0     6  Signature  "7z\xBC\xAF\x27\x1C"
//!      6     1  VersionMajor
//!      7     1  VersionMinor
//!      8     4  StartHeaderCRC   — CRC32 of bytes 12..32
//!     12     8  NextHeaderOffset — distance from byte 32 to the metadata block
//!     20     8  NextHeaderSize
//!     28     4  NextHeaderCRC    — CRC32 of the metadata block
//! ```

use crate::container::crc::crc32;
use crate::container::properties::{read_u32_le, read_u64_le};
use crate::error::{SevenZippyError, SevenZippyResult};

/// Magic bytes at the start of every 7z archive.
pub const SIGNATURE: [u8; 6] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

/// The fixed 32-byte header at offset 0 of a `.7z` file.
#[derive(Debug, Clone)]
pub struct SignatureHeader {
    /// Must equal `SIGNATURE` (`"7z\xBC\xAF\x27\x1C"`).
    pub signature: [u8; 6],
    pub version_major: u8,
    pub version_minor: u8,
    /// CRC32 of bytes 12..32 of the signature header (the 20 bytes that
    /// immediately follow this field).
    pub start_header_crc: u32,
    /// Byte offset of the end-header block, measured from byte 32 (i.e. the
    /// first byte after the signature header).
    pub next_header_offset: u64,
    /// Byte length of the end-header block.
    pub next_header_size: u64,
    /// CRC32 of the end-header block.
    pub next_header_crc: u32,
}

impl SignatureHeader {
    /// Parse and validate the 32-byte signature header.
    ///
    /// Returns [`SevenZippyError::InvalidSignature`] if:
    /// - The magic bytes don't match.
    /// - The `start_header_crc` doesn't match the CRC32 of bytes 12..32.
    pub fn parse(input: &[u8; 32]) -> SevenZippyResult<SignatureHeader> {
        let sig: [u8; 6] = input[0..6].try_into().unwrap();
        if sig != SIGNATURE {
            return Err(SevenZippyError::invalid_signature(format!(
                "bad magic: expected {:02X?}, got {:02X?}",
                SIGNATURE, sig
            )));
        }

        let version_major = input[6];
        let version_minor = input[7];

        let stored_crc = u32::from_le_bytes([input[8], input[9], input[10], input[11]]);
        // CRC covers the 20 bytes at offset 12..32
        let computed_crc = crc32(&input[12..32]);
        if stored_crc != computed_crc {
            return Err(SevenZippyError::invalid_signature(format!(
                "StartHeaderCRC mismatch: stored {stored_crc:#010x}, computed {computed_crc:#010x}"
            )));
        }

        let mut tail: &[u8] = &input[12..];
        let next_header_offset = read_u64_le(&mut tail)?;
        let next_header_size = read_u64_le(&mut tail)?;
        let next_header_crc = read_u32_le(&mut tail)?;

        // Sanity: tail should be exhausted now (28 bytes consumed: 8+8+4 = 20; started at 12).
        debug_assert!(tail.is_empty());

        Ok(SignatureHeader {
            signature: sig,
            version_major,
            version_minor,
            start_header_crc: stored_crc,
            next_header_offset,
            next_header_size,
            next_header_crc,
        })
    }

    /// Return a byte slice to the bytes that `start_header_crc` covers,
    /// from a full raw input slice. Useful for callers that want to re-verify.
    pub fn crc_region(raw: &[u8; 32]) -> &[u8] {
        &raw[12..32]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a valid 32-byte signature-header from components.
    fn make_header(
        sig: [u8; 6],
        major: u8,
        minor: u8,
        next_offset: u64,
        next_size: u64,
        next_crc: u32,
    ) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[0..6].copy_from_slice(&sig);
        buf[6] = major;
        buf[7] = minor;
        // Bytes 12..32: offset(8) + size(8) + crc(4)
        buf[12..20].copy_from_slice(&next_offset.to_le_bytes());
        buf[20..28].copy_from_slice(&next_size.to_le_bytes());
        buf[28..32].copy_from_slice(&next_crc.to_le_bytes());
        // Now compute and fill start_header_crc
        let crc = crc32(&buf[12..32]);
        buf[8..12].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    #[test]
    fn valid_header_parses() {
        let raw = make_header(SIGNATURE, 0, 4, 19, 98, 0x7AD6_5038);
        let sh = SignatureHeader::parse(&raw).expect("should parse");
        assert_eq!(sh.signature, SIGNATURE);
        assert_eq!(sh.version_major, 0);
        assert_eq!(sh.version_minor, 4);
        assert_eq!(sh.next_header_offset, 19);
        assert_eq!(sh.next_header_size, 98);
        assert_eq!(sh.next_header_crc, 0x7AD6_5038);
    }

    #[test]
    fn bad_magic_rejected() {
        let mut raw = make_header(SIGNATURE, 0, 4, 0, 0, 0);
        raw[0] = 0xFF; // corrupt first magic byte
                       // CRC will also be wrong, but magic check comes first
        let err = SignatureHeader::parse(&raw).unwrap_err();
        assert!(
            matches!(err, crate::error::SevenZippyError::InvalidSignature(_)),
            "expected InvalidSignature, got {err:?}"
        );
    }

    #[test]
    fn bad_crc_rejected() {
        let mut raw = make_header(SIGNATURE, 0, 4, 0, 0, 0);
        // Flip a bit in the CRC field to corrupt it
        raw[8] ^= 0xFF;
        let err = SignatureHeader::parse(&raw).unwrap_err();
        assert!(matches!(
            err,
            crate::error::SevenZippyError::InvalidSignature(_)
        ));
    }

    #[test]
    fn truncated_input_does_not_panic() {
        // The function takes &[u8; 32] so truncation can't happen at the type
        // level — but we can verify the fixture bytes parse correctly.
        let raw = make_header(SIGNATURE, 0, 4, 0, 0, 0);
        // Just ensure it does not panic
        let _ = SignatureHeader::parse(&raw);
    }

    #[test]
    fn parses_real_fixture_bytes() {
        // First 32 bytes of corpora/fixtures/archives/copy_only.7z
        let raw: [u8; 32] = [
            0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c, // magic
            0x00, 0x04, // version
            0xb0, 0x8b, 0x42, 0xba, // start_header_crc
            0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // offset = 19
            0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // size = 98
            0x38, 0x50, 0xd6, 0x7a, // next_header_crc
        ];
        let sh = SignatureHeader::parse(&raw).expect("real fixture must parse");
        assert_eq!(sh.next_header_offset, 19);
        assert_eq!(sh.next_header_size, 98);
        assert_eq!(sh.next_header_crc, 0x7AD6_5038);
    }
}
