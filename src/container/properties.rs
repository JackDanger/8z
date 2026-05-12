//! Low-level I/O helpers for parsing the 7z property-stream encoding.
//!
//! The 7z format uses Igor Pavlov's own variable-length UINT64 encoding
//! (distinct from LEB-128). It also defines a set of property-ID tags that
//! label every block inside the end-header.

use crate::error::{EightZError, EightZResult};

// ── Primitive readers ─────────────────────────────────────────────────────────

/// Read a single byte and advance the cursor.
pub(crate) fn read_u8(input: &mut &[u8]) -> EightZResult<u8> {
    if input.is_empty() {
        return Err(EightZError::truncated("expected 1 byte"));
    }
    let b = input[0];
    *input = &input[1..];
    Ok(b)
}

/// Slice off exactly `n` bytes and advance the cursor.
pub(crate) fn read_bytes<'a>(input: &mut &'a [u8], n: usize) -> EightZResult<&'a [u8]> {
    if input.len() < n {
        return Err(EightZError::truncated(format!(
            "need {n} bytes, only {} available",
            input.len()
        )));
    }
    let (head, tail) = input.split_at(n);
    *input = tail;
    Ok(head)
}

/// Read a 4-byte little-endian `u32`.
pub(crate) fn read_u32_le(input: &mut &[u8]) -> EightZResult<u32> {
    let b = read_bytes(input, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Read an 8-byte little-endian `u64`.
pub(crate) fn read_u64_le(input: &mut &[u8]) -> EightZResult<u64> {
    let b = read_bytes(input, 8)?;
    Ok(u64::from_le_bytes(b.try_into().unwrap()))
}

/// Read a 7z variable-length UINT64.
///
/// This uses Igor Pavlov's encoding (from 7zIn.cpp `CInArchive::ReadNumber`),
/// which differs from LEB-128:
///
/// ```text
/// first_byte = read_byte()
/// mask = 0x80
/// value = 0
/// for i in 0..8:
///     if (first_byte & mask) == 0:
///         high_part = first_byte & (mask - 1)
///         value |= high_part << (i * 8)
///         return value
///     value |= read_byte() << (i * 8)
///     mask >>= 1
/// return value   // all 8 bits were set → 8 extra bytes (max 64-bit value)
/// ```
///
/// In the special case where all 8 bits of `first_byte` are set (first_byte ==
/// 0xFF), `mask` reaches 0 after 8 iterations and the loop terminates, having
/// read 8 extra bytes. This can represent the full u64 range.
pub(crate) fn read_uint64(input: &mut &[u8]) -> EightZResult<u64> {
    let first = read_u8(input)?;
    let mut mask: u8 = 0x80;
    let mut value: u64 = 0;

    for i in 0..8u32 {
        if (first & mask) == 0 {
            // High portion lives in the low bits of `first` (below `mask`).
            let high_part = u64::from(first & mask.wrapping_sub(1));
            value |= high_part << (i * 8);
            return Ok(value);
        }
        // This bit was set: consume one more byte for this byte-position.
        let b = read_u8(input)?;
        value |= u64::from(b) << (i * 8);
        mask >>= 1;
    }
    // All 8 bits were set (first_byte == 0xFF): value contains 8 read bytes.
    Ok(value)
}

/// Read a bit-vector of `n` items, packed MSB-first into whole bytes.
///
/// Returns a `Vec<bool>` of length `n`. The packed byte stream is ceil(n/8)
/// bytes wide; the most-significant bit of the first byte is item 0.
pub(crate) fn read_bit_vector(input: &mut &[u8], n: usize) -> EightZResult<Vec<bool>> {
    let num_bytes = n.div_ceil(8);
    let raw = read_bytes(input, num_bytes)?;
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let byte_idx = i / 8;
        let bit_idx = 7 - (i % 8); // MSB first
        result.push((raw[byte_idx] >> bit_idx) & 1 == 1);
    }
    Ok(result)
}

// ── Property-ID enum ──────────────────────────────────────────────────────────

/// Property tag bytes as they appear in the 7z header stream.
///
/// Values taken from 7zFormat.txt section 5.2 (NID constants).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PropertyId {
    End = 0x00,
    Header = 0x01,
    ArchiveProperties = 0x02,
    AdditionalStreamsInfo = 0x03,
    MainStreamsInfo = 0x04,
    FilesInfo = 0x05,
    PackInfo = 0x06,
    UnpackInfo = 0x07,
    SubStreamsInfo = 0x08,
    Size = 0x09,
    Crc = 0x0A,
    Folder = 0x0B,
    CodersUnpackSize = 0x0C,
    NumUnpackStream = 0x0D,
    EmptyStream = 0x0E,
    EmptyFile = 0x0F,
    Anti = 0x10,
    Name = 0x11,
    CTime = 0x12,
    ATime = 0x13,
    MTime = 0x14,
    Attributes = 0x15,
    Comment = 0x16,
    EncodedHeader = 0x17,
    StartPos = 0x18,
    Dummy = 0x19,
}

impl PropertyId {
    /// Convert a raw byte to a [`PropertyId`], returning [`EightZError::InvalidHeader`]
    /// if the byte is not a known property tag.
    pub fn from_u8(b: u8) -> EightZResult<Self> {
        match b {
            0x00 => Ok(Self::End),
            0x01 => Ok(Self::Header),
            0x02 => Ok(Self::ArchiveProperties),
            0x03 => Ok(Self::AdditionalStreamsInfo),
            0x04 => Ok(Self::MainStreamsInfo),
            0x05 => Ok(Self::FilesInfo),
            0x06 => Ok(Self::PackInfo),
            0x07 => Ok(Self::UnpackInfo),
            0x08 => Ok(Self::SubStreamsInfo),
            0x09 => Ok(Self::Size),
            0x0A => Ok(Self::Crc),
            0x0B => Ok(Self::Folder),
            0x0C => Ok(Self::CodersUnpackSize),
            0x0D => Ok(Self::NumUnpackStream),
            0x0E => Ok(Self::EmptyStream),
            0x0F => Ok(Self::EmptyFile),
            0x10 => Ok(Self::Anti),
            0x11 => Ok(Self::Name),
            0x12 => Ok(Self::CTime),
            0x13 => Ok(Self::ATime),
            0x14 => Ok(Self::MTime),
            0x15 => Ok(Self::Attributes),
            0x16 => Ok(Self::Comment),
            0x17 => Ok(Self::EncodedHeader),
            0x18 => Ok(Self::StartPos),
            0x19 => Ok(Self::Dummy),
            other => Err(EightZError::invalid_header(format!(
                "unknown property ID {other:#04x}"
            ))),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_uint64_zero() {
        let mut buf: &[u8] = &[0x00];
        assert_eq!(read_uint64(&mut buf).unwrap(), 0);
    }

    #[test]
    fn read_uint64_max_one_byte() {
        // 0x7F = 127 — no leading 1-bits, so no extra bytes.
        let mut buf: &[u8] = &[0x7F];
        assert_eq!(read_uint64(&mut buf).unwrap(), 0x7F);
    }

    #[test]
    fn read_uint64_two_byte_boundary_low() {
        // 0x80 => first byte has 1 leading 1-bit → 1 extra byte follows.
        // first_reduced = 0x00, extra = 0x80 → value = 0x80
        let mut buf: &[u8] = &[0x80, 0x80];
        assert_eq!(read_uint64(&mut buf).unwrap(), 0x80);
    }

    #[test]
    fn read_uint64_two_byte_range() {
        // 2-byte form: first byte = 0xBF = 1011_1111
        // Decode: mask=0x80, (0xBF & 0x80) != 0 → read extra byte, place at bit 0.
        //   extra byte = 0x01 → value |= 0x01 << 0 = 0x01
        //   mask = 0x40, (0xBF & 0x40) != 0 → but wait we stop after reading extra byte
        //   No: we check (first & mask) not (running & mask). So:
        //   i=0: (0xBF & 0x80) != 0 → read byte 0x01, value = 0x01
        //   i=1: (0xBF & 0x40) != 0 → ... wait first_byte = 0xBF, and bit 6 is 0!
        //   0xBF = 1011_1111, bit 6 = 0 → (0xBF & 0x40) == 0 → STOP
        //   high_part = 0xBF & (0x40 - 1) = 0xBF & 0x3F = 0x3F
        //   value |= 0x3F << (1 * 8) = 0x3F00
        //   total: 0x3F01 = 16129
        let mut buf: &[u8] = &[0xBF, 0x01];
        assert_eq!(read_uint64(&mut buf).unwrap(), 0x3F01);
    }

    #[test]
    fn read_uint64_0x3fff() {
        // 0x3FFF = 16383 in 2-byte encoding.
        // Encoding: first byte = 0x80 | (0x3FFF >> 8) = 0x80 | 0x3F = 0xBF,
        // second byte = 0xFF.
        // Decode: i=0: (0xBF & 0x80) != 0 → value |= 0xFF; mask=0x40
        //         i=1: (0xBF & 0x40) == 0 → high_part = 0xBF & 0x3F = 0x3F
        //                                    value |= 0x3F << 8 = 0x3F00
        //         total = 0xFF | 0x3F00 = 0x3FFF ✓
        let mut buf: &[u8] = &[0xBF, 0xFF];
        assert_eq!(read_uint64(&mut buf).unwrap(), 0x3FFF);
    }

    #[test]
    fn read_uint64_0x4000() {
        // 0x4000 = 16384 needs 3-byte encoding.
        // Encoding: first byte = 0xC0, extra bytes = [0x00, 0x40] (LE low→high).
        // Decode: i=0: (0xC0 & 0x80) != 0 → value |= 0x00 << 0; mask=0x40
        //         i=1: (0xC0 & 0x40) != 0 → value |= 0x40 << 8 = 0x4000; mask=0x20
        //         i=2: (0xC0 & 0x20) == 0 → high_part = 0xC0 & 0x1F = 0x00
        //                                    value |= 0 << 16 = 0
        //         total = 0x4000 ✓
        let mut buf: &[u8] = &[0xC0, 0x00, 0x40];
        assert_eq!(read_uint64(&mut buf).unwrap(), 0x4000);
    }

    #[test]
    fn read_uint64_large() {
        // 0x1FFF_FFFF_FFFF_FFFF requires the 9-byte form (first_byte = 0xFF).
        // Per spec: "11111111 BYTE y[8]: y" — all 8 bits of first_byte are set,
        // so all 8 loop iterations read an extra byte, yielding 8 LE bytes of value.
        //
        // Encoding: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F]
        //   i=0: read 0xFF → value |= 0xFF << 0
        //   i=1: read 0xFF → value |= 0xFF << 8
        //   ...
        //   i=7: read 0x1F → value |= 0x1F << 56
        //   loop ends (i==8), return value = 0x1FFF_FFFF_FFFF_FFFF ✓
        let val: u64 = 0x1FFF_FFFF_FFFF_FFFF;
        let buf: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F];
        let mut cursor = buf;
        assert_eq!(read_uint64(&mut cursor).unwrap(), val);
    }

    #[test]
    fn read_uint64_nine_byte_form() {
        // 0xFF prefix → 8 pure LE bytes
        let val: u64 = u64::MAX;
        let mut bytes = vec![0xFF_u8];
        bytes.extend_from_slice(&val.to_le_bytes());
        let mut buf: &[u8] = &bytes;
        assert_eq!(read_uint64(&mut buf).unwrap(), val);
    }

    #[test]
    fn read_uint64_truncated() {
        let mut buf: &[u8] = &[0x80]; // promises 1 extra byte but none follow
        assert!(read_uint64(&mut buf).is_err());
    }

    #[test]
    fn bit_vector_basic() {
        // 2 items: first byte 0b1010_0000 MSB-first → [true, false]
        let mut buf: &[u8] = &[0b1010_0000];
        let bv = read_bit_vector(&mut buf, 2).unwrap();
        assert_eq!(bv, vec![true, false]);
    }

    #[test]
    fn bit_vector_eight_items() {
        let mut buf: &[u8] = &[0b1111_0000];
        let bv = read_bit_vector(&mut buf, 8).unwrap();
        assert_eq!(bv, vec![true, true, true, true, false, false, false, false]);
    }

    #[test]
    fn property_id_round_trip() {
        for b in 0x00_u8..=0x19 {
            let id = PropertyId::from_u8(b).unwrap();
            assert_eq!(id as u8, b);
        }
    }

    #[test]
    fn property_id_unknown() {
        assert!(PropertyId::from_u8(0x1A).is_err());
        assert!(PropertyId::from_u8(0xFF).is_err());
    }
}
