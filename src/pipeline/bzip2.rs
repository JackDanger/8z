//! BZip2 coder — delegates to the `bzippy2` sub-crate (Phase 1 wrapper).
//!
//! 7z's BZip2 codec (method ID `[0x04, 0x02, 0x02]`) stores a standard BZip2
//! bitstream. No codec-specific properties blob is needed.

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// BZip2 coder backed by bzippy2 (bzip2 crate under the hood).
pub struct Bzip2Coder;

impl Coder for Bzip2Coder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        bzippy2::decode(packed).map_err(|e| SevenZippyError::Coder(Box::new(e)))
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        // Use compression level 6 (good balance of speed/ratio, matches 7zz default).
        bzippy2::encode(unpacked, 6).map_err(|e| SevenZippyError::Coder(Box::new(e)))
    }

    fn method_id(&self) -> MethodId {
        MethodId::bzip2()
    }

    fn properties(&self) -> Vec<u8> {
        // BZip2 has no codec-specific properties in 7z.
        Vec::new()
    }
}
