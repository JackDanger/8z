//! Coder method-ID registry and per-coder property parsing.
//!
//! A `Coder` corresponds to one entry in a 7z folder's coder chain.
//! Each coder is identified by a variable-length byte string (the `MethodId`)
//! and carries optional codec-specific properties.

use crate::container::properties::{read_bytes, read_u8, read_uint64};
use crate::error::SevenZippyResult;

// ── MethodId ──────────────────────────────────────────────────────────────────

/// Variable-length method identifier (1–15 bytes).
///
/// Known values are provided as constructor fns (`MethodId::copy()`, etc.)
/// for use in tests and dispatch code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodId(pub Vec<u8>);

impl MethodId {
    // ── Well-known method IDs ─────────────────────────────────────────────────

    /// Copy coder — no compression.
    pub fn copy() -> MethodId {
        MethodId(vec![0x00])
    }

    /// LZMA (classic).
    pub fn lzma() -> MethodId {
        MethodId(vec![0x03, 0x01, 0x01])
    }

    /// LZMA2.
    pub fn lzma2() -> MethodId {
        MethodId(vec![0x21])
    }

    /// Deflate (zlib-compatible).
    pub fn deflate() -> MethodId {
        MethodId(vec![0x04, 0x01, 0x08])
    }

    /// Deflate64.
    pub fn deflate64() -> MethodId {
        MethodId(vec![0x04, 0x01, 0x09])
    }

    /// BZip2.
    pub fn bzip2() -> MethodId {
        MethodId(vec![0x04, 0x02, 0x02])
    }

    /// PPMd.
    pub fn ppmd() -> MethodId {
        MethodId(vec![0x03, 0x04, 0x01])
    }

    /// BCJ (x86 branch converter).
    pub fn bcj() -> MethodId {
        MethodId(vec![0x03, 0x03, 0x01, 0x03])
    }

    /// BCJ2 (enhanced branch converter with 4 streams).
    pub fn bcj2() -> MethodId {
        MethodId(vec![0x03, 0x03, 0x01, 0x1B])
    }

    /// Delta filter.
    pub fn delta() -> MethodId {
        MethodId(vec![0x03])
    }

    /// AES-256 + SHA-256 encryption.
    pub fn aes_sha256() -> MethodId {
        MethodId(vec![0x06, 0xF1, 0x07, 0x01])
    }

    /// Return a human-readable name for this method ID, if recognised.
    /// Used in debug output and error messages.
    pub fn known_name(&self) -> Option<&'static str> {
        match self.0.as_slice() {
            [0x00] => Some("Copy"),
            [0x03, 0x01, 0x01] => Some("LZMA"),
            [0x21] => Some("LZMA2"),
            [0x04, 0x01, 0x08] => Some("Deflate"),
            [0x04, 0x01, 0x09] => Some("Deflate64"),
            [0x04, 0x02, 0x02] => Some("BZip2"),
            [0x03, 0x04, 0x01] => Some("PPMd"),
            [0x03, 0x03, 0x01, 0x03] => Some("BCJ"),
            [0x03, 0x03, 0x01, 0x1B] => Some("BCJ2"),
            [0x03] => Some("Delta"),
            [0x06, 0xF1, 0x07, 0x01] => Some("AES-SHA256"),
            _ => None,
        }
    }
}

// ── Coder ─────────────────────────────────────────────────────────────────────

/// One entry in a folder's coder chain.
#[derive(Clone, Debug)]
pub struct Coder {
    /// The codec identifier (variable length, 1–15 bytes).
    pub method_id: MethodId,
    /// Number of input streams this coder consumes.
    /// Always 1 for simple coders; >1 only for multi-stream coders (e.g. BCJ2).
    pub num_in_streams: u64,
    /// Number of output streams this coder produces.
    /// Always 1 for simple coders.
    pub num_out_streams: u64,
    /// Codec-specific properties blob (e.g. LZMA's 5-byte props).
    /// Empty for coders that carry no properties (Copy, most filters).
    pub properties: Vec<u8>,
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Parse one coder entry from the cursor.
///
/// 7z coder encoding (7zFormat.txt §5.3.5):
///
/// ```text
/// FlagByte:
///   bits 3:0  = CodecIdSize (1..15)
///   bit  4    = 0 → simple coder (1 in / 1 out); 1 → complex coder
///   bit  5    = 0 → no attributes; 1 → attributes follow
///   bits 7:6  = reserved (should be 0)
///
/// CodecId[CodecIdSize]
/// if complex:
///   NumInStreams  UINT64
///   NumOutStreams UINT64
/// if attributes:
///   PropertiesSize UINT64
///   Properties[PropertiesSize]
/// ```
pub(crate) fn parse_coder(input: &mut &[u8]) -> SevenZippyResult<Coder> {
    let flag = read_u8(input)?;
    let id_size = (flag & 0x0F) as usize;
    let is_complex = (flag >> 4) & 1 != 0;
    let has_attrs = (flag >> 5) & 1 != 0;

    let id_bytes = read_bytes(input, id_size)?;
    let method_id = MethodId(id_bytes.to_vec());

    let (num_in_streams, num_out_streams) = if is_complex {
        (read_uint64(input)?, read_uint64(input)?)
    } else {
        (1, 1)
    };

    let properties = if has_attrs {
        let props_size = read_uint64(input)? as usize;
        read_bytes(input, props_size)?.to_vec()
    } else {
        Vec::new()
    };

    Ok(Coder {
        method_id,
        num_in_streams,
        num_out_streams,
        properties,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_copy_coder() {
        // Flag byte: 0x01 → id_size=1, simple, no attrs
        // Method ID: 0x00
        let bytes: &[u8] = &[0x01, 0x00];
        let mut cursor = bytes;
        let coder = parse_coder(&mut cursor).expect("should parse Copy coder");
        assert_eq!(coder.method_id, MethodId::copy());
        assert_eq!(coder.num_in_streams, 1);
        assert_eq!(coder.num_out_streams, 1);
        assert!(coder.properties.is_empty());
    }

    #[test]
    fn parse_lzma_coder_with_props() {
        // Flag: 0x23 → id_size=3, no complex (bit4=0), has_attrs (bit5=1)
        // Wait: 0x23 = 0010_0011 → id_size=3, complex=0, has_attrs=1 ✓
        // Method ID: 03 01 01 (LZMA)
        // Props size: 0x05
        // Props: 5d 00 10 00 00 (typical LZMA props)
        let bytes: &[u8] = &[0x23, 0x03, 0x01, 0x01, 0x05, 0x5D, 0x00, 0x10, 0x00, 0x00];
        let mut cursor = bytes;
        let coder = parse_coder(&mut cursor).expect("should parse LZMA coder");
        assert_eq!(coder.method_id, MethodId::lzma());
        assert_eq!(coder.num_in_streams, 1);
        assert_eq!(coder.num_out_streams, 1);
        assert_eq!(coder.properties, &[0x5D, 0x00, 0x10, 0x00, 0x00]);
    }

    #[test]
    fn parse_complex_coder() {
        // Flag: 0x11 → id_size=1, complex (bit4=1), no attrs (bit5=0)
        // Method ID: 0x21 (LZMA2)
        // NumInStreams: 0x02, NumOutStreams: 0x01
        let bytes: &[u8] = &[0x11, 0x21, 0x02, 0x01];
        let mut cursor = bytes;
        let coder = parse_coder(&mut cursor).expect("should parse complex coder");
        assert_eq!(coder.method_id, MethodId::lzma2());
        assert_eq!(coder.num_in_streams, 2);
        assert_eq!(coder.num_out_streams, 1);
    }

    #[test]
    fn known_names() {
        assert_eq!(MethodId::copy().known_name(), Some("Copy"));
        assert_eq!(MethodId::lzma().known_name(), Some("LZMA"));
        assert_eq!(MethodId::lzma2().known_name(), Some("LZMA2"));
        assert_eq!(MethodId::deflate().known_name(), Some("Deflate"));
        assert_eq!(MethodId::bcj2().known_name(), Some("BCJ2"));
        assert_eq!(MethodId(vec![0xDE, 0xAD]).known_name(), None);
    }

    #[test]
    fn method_id_equality() {
        assert_eq!(MethodId::copy(), MethodId::copy());
        assert_ne!(MethodId::copy(), MethodId::lzma());
    }
}
