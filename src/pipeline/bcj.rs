//! BCJ (Branch/Call/Jump) family filter coders — in-tree implementation.
//!
//! BCJ filters are pre-conditioners that convert architecture-specific
//! relative branch offsets to absolute addresses, making byte patterns more
//! repetitive and thus improving subsequent compression (typically LZMA).
//!
//! # 7z method IDs
//!
//! | Filter        | Method ID bytes                  |
//! |---------------|----------------------------------|
//! | BCJ x86       | `[0x03, 0x03, 0x01, 0x03]`       |
//! | BCJ PowerPC   | `[0x03, 0x03, 0x02, 0x05]`       |
//! | BCJ IA64      | `[0x03, 0x03, 0x04, 0x01]`       |
//! | BCJ ARM       | `[0x03, 0x03, 0x05, 0x01]`       |
//! | BCJ ARM-Thumb | `[0x03, 0x03, 0x07, 0x01]`       |
//! | BCJ SPARC     | `[0x03, 0x03, 0x08, 0x05]`       |
//!
//! # Phase 1
//!
//! Phase 1 uses the `lzma-rust2` BCJ filter module (`lzma_rust2::filter::bcj`)
//! which is an established pure-Rust implementation of all ISA variants. A
//! future Phase 2 `jumpzippy` sub-crate will own these implementations directly.
//!
//! # Properties
//!
//! BCJ filters in 7z carry an optional 4-byte start-position property. When
//! absent (the common case), `start_pos = 0`. When present, bytes 0–3 are a
//! LE u32.
//!
//! # Encoding vs decoding symmetry
//!
//! BCJ is a self-inverse filter: applying it twice returns the original data.
//! We use `BcjWriter` (encode) and `BcjReader` (decode), both from lzma_rust2.

use std::io::{Read, Write};

use lzma_rust2::filter::bcj::{BcjReader, BcjWriter};

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

// ── BCJ architecture variants ─────────────────────────────────────────────────

/// Which ISA the BCJ filter targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BcjArch {
    X86,
    PowerPc,
    Ia64,
    Arm,
    ArmThumb,
    Sparc,
}

impl BcjArch {
    /// Return the 7z method ID for this BCJ variant.
    pub fn method_id(self) -> MethodId {
        match self {
            BcjArch::X86 => MethodId(vec![0x03, 0x03, 0x01, 0x03]),
            BcjArch::PowerPc => MethodId(vec![0x03, 0x03, 0x02, 0x05]),
            BcjArch::Ia64 => MethodId(vec![0x03, 0x03, 0x04, 0x01]),
            BcjArch::Arm => MethodId(vec![0x03, 0x03, 0x05, 0x01]),
            BcjArch::ArmThumb => MethodId(vec![0x03, 0x03, 0x07, 0x01]),
            BcjArch::Sparc => MethodId(vec![0x03, 0x03, 0x08, 0x05]),
        }
    }
}

// ── BcjCoder ─────────────────────────────────────────────────────────────────

/// BCJ filter coder backed by `lzma_rust2::filter::bcj` (Phase 1).
pub struct BcjCoder {
    arch: BcjArch,
    /// Starting byte offset for the filter (LE u32 from the properties blob, or 0).
    start_pos: u32,
}

impl BcjCoder {
    /// Create a BCJ coder for the given architecture with `start_pos = 0`.
    pub fn new(arch: BcjArch) -> Self {
        Self { arch, start_pos: 0 }
    }

    /// Create a BCJ coder from the optional property blob stored in a 7z archive.
    ///
    /// The properties are either empty (→ `start_pos = 0`) or 4 bytes (LE u32).
    pub fn from_arch_props(arch: BcjArch, props: &[u8]) -> SevenZippyResult<Self> {
        let start_pos = match props.len() {
            0 => 0,
            4 => u32::from_le_bytes([props[0], props[1], props[2], props[3]]),
            n => {
                return Err(SevenZippyError::Coder(
                    format!("BCJ properties must be 0 or 4 bytes, got {n}").into(),
                ))
            }
        };
        Ok(Self { arch, start_pos })
    }

    /// Apply the BCJ encode transform to `input`, returning filtered bytes.
    fn apply_encode(&self, input: &[u8]) -> SevenZippyResult<Vec<u8>> {
        let pos = self.start_pos as usize;
        // Use a macro to avoid repeating the write_all + finish pattern for each variant.
        macro_rules! encode_with {
            ($writer:expr) => {{
                let mut w = $writer;
                w.write_all(input).map_err(|e| {
                    SevenZippyError::Coder(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                })?;
                w.finish().map_err(|e| {
                    SevenZippyError::Coder(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                })
            }};
        }
        match self.arch {
            BcjArch::X86 => encode_with!(BcjWriter::new_x86(Vec::new(), pos)),
            BcjArch::PowerPc => encode_with!(BcjWriter::new_ppc(Vec::new(), pos)),
            BcjArch::Ia64 => encode_with!(BcjWriter::new_ia64(Vec::new(), pos)),
            BcjArch::Arm => encode_with!(BcjWriter::new_arm(Vec::new(), pos)),
            BcjArch::ArmThumb => encode_with!(BcjWriter::new_arm_thumb(Vec::new(), pos)),
            BcjArch::Sparc => encode_with!(BcjWriter::new_sparc(Vec::new(), pos)),
        }
    }

    /// Apply the BCJ decode transform to `packed`, returning original bytes.
    fn apply_decode(&self, packed: &[u8]) -> SevenZippyResult<Vec<u8>> {
        let pos = self.start_pos as usize;
        let cursor = std::io::Cursor::new(packed);
        let mut out = Vec::with_capacity(packed.len());
        macro_rules! decode_with {
            ($reader:expr) => {{
                let mut r = $reader;
                r.read_to_end(&mut out).map_err(|e| {
                    SevenZippyError::Coder(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                })?;
            }};
        }
        match self.arch {
            BcjArch::X86 => decode_with!(BcjReader::new_x86(cursor, pos)),
            BcjArch::PowerPc => decode_with!(BcjReader::new_ppc(cursor, pos)),
            BcjArch::Ia64 => decode_with!(BcjReader::new_ia64(cursor, pos)),
            BcjArch::Arm => decode_with!(BcjReader::new_arm(cursor, pos)),
            BcjArch::ArmThumb => decode_with!(BcjReader::new_arm_thumb(cursor, pos)),
            BcjArch::Sparc => decode_with!(BcjReader::new_sparc(cursor, pos)),
        }
        Ok(out)
    }
}

impl Coder for BcjCoder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        self.apply_decode(packed)
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        self.apply_encode(unpacked)
    }

    fn method_id(&self) -> MethodId {
        self.arch.method_id()
    }

    fn properties(&self) -> Vec<u8> {
        // Omit properties when start_pos is 0 (the common case).
        if self.start_pos == 0 {
            Vec::new()
        } else {
            self.start_pos.to_le_bytes().to_vec()
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// BCJ filters are self-inverse: encode(encode(x)) == x.
    fn self_inverse_round_trip(arch: BcjArch, data: &[u8]) {
        let coder = BcjCoder::new(arch);
        // Encode is the forward transform; decode is the inverse.
        // So decode(encode(x)) must equal x.
        let encoded = coder.encode(data).unwrap();
        let decoded = coder.decode(&encoded, data.len() as u64).unwrap();
        assert_eq!(decoded, data, "BCJ {arch:?} round-trip failed");
    }

    #[test]
    fn x86_round_trip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        self_inverse_round_trip(BcjArch::X86, &data);
    }

    #[test]
    fn arm_round_trip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        self_inverse_round_trip(BcjArch::Arm, &data);
    }

    #[test]
    fn arm_thumb_round_trip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        self_inverse_round_trip(BcjArch::ArmThumb, &data);
    }

    #[test]
    fn ppc_round_trip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        self_inverse_round_trip(BcjArch::PowerPc, &data);
    }

    #[test]
    fn ia64_round_trip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        self_inverse_round_trip(BcjArch::Ia64, &data);
    }

    #[test]
    fn sparc_round_trip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        self_inverse_round_trip(BcjArch::Sparc, &data);
    }

    #[test]
    fn empty_input_round_trip() {
        for arch in [
            BcjArch::X86,
            BcjArch::Arm,
            BcjArch::ArmThumb,
            BcjArch::PowerPc,
            BcjArch::Ia64,
            BcjArch::Sparc,
        ] {
            self_inverse_round_trip(arch, &[]);
        }
    }

    #[test]
    fn from_arch_props_empty() {
        let coder = BcjCoder::from_arch_props(BcjArch::X86, &[]).unwrap();
        assert_eq!(coder.start_pos, 0);
        assert_eq!(coder.properties(), vec![]);
    }

    #[test]
    fn from_arch_props_four_bytes() {
        let coder = BcjCoder::from_arch_props(BcjArch::X86, &[0x00, 0x10, 0x00, 0x00]).unwrap();
        assert_eq!(coder.start_pos, 0x1000);
        assert_eq!(coder.properties(), vec![0x00, 0x10, 0x00, 0x00]);
    }

    #[test]
    fn from_arch_props_invalid_length_is_error() {
        let result = BcjCoder::from_arch_props(BcjArch::X86, &[0x01, 0x02]);
        assert!(result.is_err());
    }
}
