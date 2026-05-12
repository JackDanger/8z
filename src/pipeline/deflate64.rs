//! Deflate64 coder — raw Deflate64 (decode-only) via the `deflate64` crate.
//!
//! Deflate64 is a 7-Zip / WinZip extension of DEFLATE that uses a 64 KiB
//! sliding window and a slightly extended set of literal/length codes.
//! Method ID: `[0x04, 0x01, 0x09]`.
//!
//! # Encode vs Decode
//!
//! **Decode**: fully supported via the `deflate64` crate (pure Rust, decode-only).
//!
//! **Encode**: no production-quality pure-Rust Deflate64 encoder exists.
//! 7-Zip itself uses plain Deflate for new archives; Deflate64 only appears
//! when *extracting* older archives. Phase 1 therefore ships decode only.
//! The `encode` method returns `Err(SevenZippyError::Coder(...))` with a clear
//! message; the `⬜ encode` column in STATUS.md is intentional.

use std::io::Read;

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Raw-Deflate64 coder backed by the `deflate64` crate (Phase 1, decode-only).
pub struct Deflate64Coder;

impl Coder for Deflate64Coder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        #[cfg(feature = "deflate64")]
        {
            use deflate64::Deflate64Decoder;
            let cursor = std::io::Cursor::new(packed);
            let mut decoder = Deflate64Decoder::new(cursor);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(|e| SevenZippyError::Coder(Box::new(e)))?;
            Ok(out)
        }
        #[cfg(not(feature = "deflate64"))]
        {
            Err(SevenZippyError::missing_coder("Deflate64"))
        }
    }

    fn encode(&self, _unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        // No production-quality pure-Rust Deflate64 encoder exists.
        // 7-Zip uses plain Deflate for new archives; Deflate64 is extraction-only.
        Err(SevenZippyError::not_yet_implemented(
            "Deflate64 encode: no pure-Rust encoder exists; use Deflate for encoding",
        ))
    }

    fn method_id(&self) -> MethodId {
        MethodId::deflate64()
    }

    fn properties(&self) -> Vec<u8> {
        Vec::new()
    }
}
