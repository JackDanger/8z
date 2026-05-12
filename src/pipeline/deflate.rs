//! Deflate coder — raw DEFLATE (no gzip/zlib header) via `flate2`.
//!
//! 7z's Deflate codec (method ID `[0x04, 0x01, 0x08]`) stores a raw DEFLATE
//! bitstream with no wrapper headers. The `flate2` crate's
//! `DeflateDecoder`/`DeflateEncoder` types operate on exactly this format.
//!
//! Note: gzippy (the permanent Phase 2 backend) is not yet published to
//! crates.io. This Phase 1 implementation uses flate2 instead. Once gzippy
//! ships a raw-deflate public API on crates.io, swap the import.

use std::io::Read;

use flate2::bufread::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// Raw-DEFLATE coder backed by flate2 (Phase 1).
pub struct DeflateCoder;

impl Coder for DeflateCoder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        let cursor = std::io::BufReader::new(packed);
        let mut decoder = DeflateDecoder::new(cursor);
        let mut out = Vec::new();
        decoder
            .read_to_end(&mut out)
            .map_err(|e| SevenZippyError::Coder(Box::new(e)))?;
        Ok(out)
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        let cursor = std::io::BufReader::new(unpacked);
        let mut encoder = DeflateEncoder::new(cursor, Compression::default());
        let mut out = Vec::new();
        encoder
            .read_to_end(&mut out)
            .map_err(|e| SevenZippyError::Coder(Box::new(e)))?;
        Ok(out)
    }

    fn method_id(&self) -> MethodId {
        MethodId::deflate()
    }

    fn properties(&self) -> Vec<u8> {
        // Deflate has no codec-specific properties in 7z.
        Vec::new()
    }
}
