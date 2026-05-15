//! Deflate64 coder — raw Deflate64 encode + decode via `gzippy` 0.8.
//!
//! Deflate64 is a 7-Zip / WinZip extension of DEFLATE that uses a 64 KiB
//! sliding window and a slightly extended set of literal/length codes.
//! Method ID: `[0x04, 0x01, 0x09]`.
//!
//! # Encode vs Decode
//!
//! **Decode**: fully supported via `gzippy::decompress_deflate64` (pure Rust).
//!
//! **Encode**: fully supported via `gzippy::compress_deflate64` (added in
//! gzippy 0.8). The encoder does not accept a level parameter; gzippy selects
//! an appropriate compression level internally.

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Raw-Deflate64 coder backed by gzippy 0.8 (encode + decode).
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

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        #[cfg(feature = "deflate64")]
        {
            gzippy::compress_deflate64(unpacked).map_err(|e| SevenZippyError::Coder(Box::new(e)))
        }
        #[cfg(not(feature = "deflate64"))]
        {
            let _ = unpacked;
            Err(SevenZippyError::missing_coder("Deflate64"))
        }
    }

    fn method_id(&self) -> MethodId {
        MethodId::deflate64()
    }

    fn properties(&self) -> Vec<u8> {
        Vec::new()
    }
}
