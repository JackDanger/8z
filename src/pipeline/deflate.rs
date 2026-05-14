//! Deflate coder — raw DEFLATE (no gzip/zlib header) via `gzippy` 0.7.
//!
//! 7z's Deflate codec (method ID `[0x04, 0x01, 0x08]`) stores a raw DEFLATE
//! bitstream with no wrapper headers. The `gzippy` crate's
//! `deflate_encode`/`deflate_decode` functions operate on exactly this format.
//!
//! Compression level 6 is used for encoding, matching gzip's default and
//! the behaviour of the former `flate2` Phase 1 backend (`Compression::default()`).

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Compression level used for Deflate encode: 6 (gzip default).
const DEFLATE_LEVEL: u8 = 6;

/// Raw-DEFLATE coder backed by gzippy 0.7.
pub struct DeflateCoder;

impl Coder for DeflateCoder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        #[cfg(feature = "deflate")]
        {
            gzippy::deflate_decode(packed).map_err(|e| SevenZippyError::Coder(Box::new(e)))
        }
        #[cfg(not(feature = "deflate"))]
        {
            Err(SevenZippyError::missing_coder("Deflate"))
        }
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        #[cfg(feature = "deflate")]
        {
            gzippy::deflate_encode(unpacked, DEFLATE_LEVEL)
                .map_err(|e| SevenZippyError::Coder(Box::new(e)))
        }
        #[cfg(not(feature = "deflate"))]
        {
            Err(SevenZippyError::missing_coder("Deflate"))
        }
    }

    fn method_id(&self) -> MethodId {
        MethodId::deflate()
    }

    fn properties(&self) -> Vec<u8> {
        // Deflate has no codec-specific properties in 7z.
        Vec::new()
    }
}
