//! Deflate coder — raw DEFLATE (no gzip/zlib header) via `gzippy`.
//!
//! 7z's Deflate codec (method ID `[0x04, 0x01, 0x08]`) stores a raw DEFLATE
//! bitstream with no wrapper headers. `gzippy::deflate_encode`/`deflate_decode`
//! operate on exactly this format, backed by ISA-L SIMD on x86_64 and
//! libdeflate elsewhere — Phase-2-quality performance for free.
//!
//! Prior Phase 1 note: flate2 was used until gzippy 0.6 exposed the raw-deflate
//! API. See: https://github.com/JackDanger/gzippy/pull/92

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Raw-DEFLATE coder backed by gzippy (Phase 2 quality via ISA-L/libdeflate).
pub struct DeflateCoder;

impl Coder for DeflateCoder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        gzippy::deflate_decode(packed).map_err(|e| SevenZippyError::Coder(Box::new(e)))
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        // Level 6 matches 7zz's default Deflate compression level.
        gzippy::deflate_encode(unpacked, 6).map_err(|e| SevenZippyError::Coder(Box::new(e)))
    }

    fn method_id(&self) -> MethodId {
        MethodId::deflate()
    }

    fn properties(&self) -> Vec<u8> {
        // Deflate has no codec-specific properties in 7z.
        Vec::new()
    }
}
