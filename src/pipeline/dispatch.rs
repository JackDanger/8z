//! Method-ID → `Box<dyn Coder>` dispatch.
//!
//! This is the single place where codec sub-crates plug in. Currently the
//! Copy and LZMA coders are wired. All others return
//! `SevenZippyError::MissingCoder` (or `SevenZippyError::UnsupportedMethod`
//! for completely unknown method IDs).

use crate::container::{Coder as CoderMeta, MethodId};
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::{Coder, CopyCoder};

/// Return a `Box<dyn Coder>` for the given 7z coder metadata.
///
/// The full `CoderMeta` is taken (not just the `MethodId`) so that coders with
/// codec-specific properties (e.g. LZMA's 5-byte props) can read them.
///
/// # Errors
///
/// - [`SevenZippyError::MissingCoder`] — the codec is known but its feature flag is
///   disabled for this build.
/// - [`SevenZippyError::UnsupportedMethod`] — the method ID is not recognised at all.
pub fn coder_for(coder_meta: &CoderMeta) -> SevenZippyResult<Box<dyn Coder>> {
    let m = coder_meta.method_id.0.as_slice();
    match m {
        [0x00] => Ok(Box::new(CopyCoder)),

        // ── LZMA — feature-gated ────────────────────────────────────────────
        [0x03, 0x01, 0x01] => {
            #[cfg(feature = "lzma")]
            {
                use crate::pipeline::lzma::LzmaCoder;
                Ok(Box::new(LzmaCoder::with_props(
                    coder_meta.properties.clone(),
                )))
            }
            #[cfg(not(feature = "lzma"))]
            {
                Err(SevenZippyError::missing_coder("LZMA"))
            }
        }

        // ── Deflate — feature-gated ─────────────────────────────────────────
        [0x04, 0x01, 0x08] => {
            #[cfg(feature = "deflate")]
            {
                use crate::pipeline::deflate::DeflateCoder;
                Ok(Box::new(DeflateCoder))
            }
            #[cfg(not(feature = "deflate"))]
            {
                Err(SevenZippyError::missing_coder("Deflate"))
            }
        }

        // ── Deflate64 — feature-gated (decode-only) ─────────────────────────────
        //    64 KiB sliding-window variant; no good Rust encoder exists.
        [0x04, 0x01, 0x09] => {
            #[cfg(feature = "deflate64")]
            {
                use crate::pipeline::deflate64::Deflate64Coder;
                Ok(Box::new(Deflate64Coder))
            }
            #[cfg(not(feature = "deflate64"))]
            {
                Err(SevenZippyError::missing_coder("Deflate64"))
            }
        }

        // ── BZip2 — feature-gated ───────────────────────────────────────────
        [0x04, 0x02, 0x02] => {
            #[cfg(feature = "bzip2")]
            {
                use crate::pipeline::bzip2::Bzip2Coder;
                Ok(Box::new(Bzip2Coder))
            }
            #[cfg(not(feature = "bzip2"))]
            {
                Err(SevenZippyError::missing_coder("BZip2"))
            }
        }

        // ── Delta filter — feature-gated ────────────────────────────────────
        [0x03] => {
            #[cfg(feature = "delta")]
            {
                use crate::pipeline::delta::DeltaCoder;
                DeltaCoder::from_props(&coder_meta.properties)
                    .map(|c| Box::new(c) as Box<dyn Coder>)
            }
            #[cfg(not(feature = "delta"))]
            {
                Err(SevenZippyError::missing_coder("Delta"))
            }
        }

        // ── PPMd — feature-gated ────────────────────────────────────────────
        [0x03, 0x04, 0x01] => {
            #[cfg(feature = "ppmd")]
            {
                use crate::pipeline::ppmd::PpmdCoder;
                PpmdCoder::from_props(&coder_meta.properties).map(|c| Box::new(c) as Box<dyn Coder>)
            }
            #[cfg(not(feature = "ppmd"))]
            {
                Err(SevenZippyError::missing_coder("PPMd"))
            }
        }

        // ── BCJ family filters — feature-gated ─────────────────────────────
        #[cfg(feature = "bcj")]
        [0x03, 0x03, 0x01, 0x03] => {
            use crate::pipeline::bcj::{BcjArch, BcjCoder};
            BcjCoder::from_arch_props(BcjArch::X86, &coder_meta.properties)
                .map(|c| Box::new(c) as Box<dyn Coder>)
        }
        #[cfg(feature = "bcj")]
        [0x03, 0x03, 0x02, 0x05] => {
            use crate::pipeline::bcj::{BcjArch, BcjCoder};
            BcjCoder::from_arch_props(BcjArch::PowerPc, &coder_meta.properties)
                .map(|c| Box::new(c) as Box<dyn Coder>)
        }
        #[cfg(feature = "bcj")]
        [0x03, 0x03, 0x04, 0x01] => {
            use crate::pipeline::bcj::{BcjArch, BcjCoder};
            BcjCoder::from_arch_props(BcjArch::Ia64, &coder_meta.properties)
                .map(|c| Box::new(c) as Box<dyn Coder>)
        }
        #[cfg(feature = "bcj")]
        [0x03, 0x03, 0x05, 0x01] => {
            use crate::pipeline::bcj::{BcjArch, BcjCoder};
            BcjCoder::from_arch_props(BcjArch::Arm, &coder_meta.properties)
                .map(|c| Box::new(c) as Box<dyn Coder>)
        }
        #[cfg(feature = "bcj")]
        [0x03, 0x03, 0x07, 0x01] => {
            use crate::pipeline::bcj::{BcjArch, BcjCoder};
            BcjCoder::from_arch_props(BcjArch::ArmThumb, &coder_meta.properties)
                .map(|c| Box::new(c) as Box<dyn Coder>)
        }
        #[cfg(feature = "bcj")]
        [0x03, 0x03, 0x08, 0x05] => {
            use crate::pipeline::bcj::{BcjArch, BcjCoder};
            BcjCoder::from_arch_props(BcjArch::Sparc, &coder_meta.properties)
                .map(|c| Box::new(c) as Box<dyn Coder>)
        }
        // BCJ fallback when feature is disabled, or unknown BCJ variant
        [0x03, 0x03, ..] => Err(SevenZippyError::missing_coder("BCJ family")),

        // ── LZMA2 — feature-gated ───────────────────────────────────────────
        [0x21] => {
            #[cfg(feature = "lzma2")]
            {
                use crate::pipeline::lzma2::Lzma2Coder;
                Lzma2Coder::with_props(coder_meta.properties.clone())
                    .map(|c| Box::new(c) as Box<dyn Coder>)
            }
            #[cfg(not(feature = "lzma2"))]
            {
                Err(SevenZippyError::missing_coder("LZMA2"))
            }
        }
        [0x06, 0xF1, 0x07, 0x01] => Err(SevenZippyError::missing_coder("AES+SHA-256")),

        _ => Err(SevenZippyError::unsupported_method(
            coder_meta.method_id.0.clone(),
        )),
    }
}

/// Return a `Box<dyn Coder>` for the given 7z method ID with empty properties.
///
/// Convenience wrapper for callers that only have a `MethodId` (e.g. tests).
pub fn coder_for_method(method_id: &MethodId) -> SevenZippyResult<Box<dyn Coder>> {
    let meta = CoderMeta {
        method_id: method_id.clone(),
        num_in_streams: 1,
        num_out_streams: 1,
        properties: Vec::new(),
    };
    coder_for(&meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_coder_dispatches() {
        let coder = coder_for_method(&MethodId::copy()).unwrap();
        let data = b"test data";
        let encoded = coder.encode(data).unwrap();
        let decoded = coder.decode(&encoded, data.len() as u64).unwrap();
        assert_eq!(decoded, data);
    }

    #[cfg(not(feature = "lzma2"))]
    #[test]
    fn lzma2_is_missing_without_feature() {
        let result = coder_for_method(&MethodId::lzma2());
        assert!(matches!(result, Err(SevenZippyError::MissingCoder { .. })));
    }

    #[test]
    fn unknown_method_is_unsupported() {
        let unknown = MethodId(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let result = coder_for_method(&unknown);
        assert!(matches!(
            result,
            Err(SevenZippyError::UnsupportedMethod { .. })
        ));
    }

    #[cfg(feature = "lzma")]
    #[test]
    fn lzma_coder_dispatches() {
        // Standard LZMA props: props_byte=0x5D, dict_size=1MiB
        let props = vec![0x5D, 0x00, 0x00, 0x10, 0x00];
        let meta = CoderMeta {
            method_id: MethodId::lzma(),
            num_in_streams: 1,
            num_out_streams: 1,
            properties: props,
        };
        let coder = coder_for(&meta).unwrap();
        let input = b"hello LZMA world";
        let encoded = coder.encode(input).unwrap();
        let decoded = coder.decode(&encoded, input.len() as u64).unwrap();
        assert_eq!(decoded, input);
    }
}
