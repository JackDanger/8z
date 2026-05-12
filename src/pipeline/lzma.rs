//! LZMA coder — delegates to the `lazippy` sub-crate (Phase 1 wrapper).
//!
//! The 7z LZMA coder properties are 5 bytes:
//!   [props_byte(1)] [dict_size_le32(4)]
//!
//! These are stored in the `Coder.properties` field of the archive metadata and
//! must be written back when building archives.

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Default dictionary size for encoding (1 MiB — small enough to be fast in tests).
const DEFAULT_DICT_SIZE: u32 = 1 << 20;

/// LZMA coder backed by lazippy (lzma-rust2 under the hood).
pub struct LzmaCoder {
    /// The 5-byte properties blob: [props_byte(1)] [dict_size_le32(4)].
    /// Supplied when decoding (read from archive); computed when encoding.
    props: Option<Vec<u8>>,
}

impl LzmaCoder {
    /// Create a coder for encoding with the default properties.
    pub fn new() -> Self {
        Self { props: None }
    }

    /// Create a coder for decoding, supplying the stored 5-byte properties.
    pub fn with_props(props: Vec<u8>) -> Self {
        Self { props: Some(props) }
    }
}

impl Default for LzmaCoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Coder for LzmaCoder {
    fn decode(&self, packed: &[u8], unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        let props = self.props.as_deref().unwrap_or(&[]);
        lazippy::decode::decode_7z(packed, props, unpacked_size).map_err(SevenZippyError::from)
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        let (_props_blob, compressed) = lazippy::encode::encode_7z(unpacked, DEFAULT_DICT_SIZE)
            .map_err(SevenZippyError::from)?;
        Ok(compressed)
    }

    fn method_id(&self) -> MethodId {
        MethodId::lzma()
    }

    fn properties(&self) -> Vec<u8> {
        // When we have stored props (decode path) return them unchanged.
        if let Some(p) = &self.props {
            return p.clone();
        }
        // Encoding path: generate default 5-byte props blob.
        // Standard LZMA defaults: lc=3, lp=0, pb=2
        // props_byte = (pb*5 + lp)*9 + lc = (2*5)*9 + 3 = 93 = 0x5D
        const PROPS_BYTE: u8 = (2 * 5) * 9 + 3; // lc=3, lp=0, pb=2
        let mut props = Vec::with_capacity(5);
        props.push(PROPS_BYTE);
        props.extend_from_slice(&DEFAULT_DICT_SIZE.to_le_bytes());
        props
    }
}
