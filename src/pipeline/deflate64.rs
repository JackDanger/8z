//! Deflate64 coder — raw Deflate64 (decode-only) via `gzippy` 0.7.
//!
//! Deflate64 is a 7-Zip / WinZip extension of DEFLATE that uses a 64 KiB
//! sliding window and a slightly extended set of literal/length codes.
//! Method ID: `[0x04, 0x01, 0x09]`.
//!
//! # Encode vs Decode
//!
//! **Decode**: fully supported via `gzippy::decompress_deflate64` (pure Rust).
//!
//! **Encode**: gzippy 0.7 ships Deflate64 decode only; no production-quality
//! pure-Rust Deflate64 encoder exists. 7-Zip itself uses plain Deflate for new
//! archives; Deflate64 only appears when *extracting* older archives. The
//! `encode` method returns `Err(SevenZippyError::NotYetImplemented(...))` with a
//! clear message; the `⬜ encode` column in STATUS.md is intentional.
//! Opus will surface the Phase 1 closure decision for this column separately.

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Raw-Deflate64 coder backed by gzippy 0.7 (decode-only).
pub struct Deflate64Coder;

impl Coder for Deflate64Coder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        #[cfg(feature = "deflate64")]
        {
            gzippy::decompress_deflate64(packed).map_err(|e| SevenZippyError::Coder(Box::new(e)))
        }
        #[cfg(not(feature = "deflate64"))]
        {
            Err(SevenZippyError::missing_coder("Deflate64"))
        }
    }

    fn encode(&self, _unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        // gzippy 0.7 ships Deflate64 decode only; no pure-Rust Deflate64 encoder exists.
        // 7-Zip uses plain Deflate for new archives; Deflate64 is extraction-only.
        // See Phase 1 closure decision (Opus will surface separately).
        Err(SevenZippyError::not_yet_implemented(
            "Deflate64 encode not yet supported (gzippy 0.7 ships decode only); see Phase 1 closure decision",
        ))
    }

    fn method_id(&self) -> MethodId {
        MethodId::deflate64()
    }

    fn properties(&self) -> Vec<u8> {
        Vec::new()
    }
}
