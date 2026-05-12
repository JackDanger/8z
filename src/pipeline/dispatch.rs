//! Method-ID → `Box<dyn Coder>` dispatch.
//!
//! This is the single place where codec sub-crates plug in. Currently only the
//! in-tree Copy coder is wired. All others return `EightZError::MissingCoder` (or
//! `EightZError::UnsupportedMethod` for completely unknown method IDs).

use crate::container::MethodId;
use crate::error::{EightZError, EightZResult};
use crate::pipeline::{Coder, CopyCoder};

/// Return a `Box<dyn Coder>` for the given 7z method ID.
///
/// # Errors
///
/// - [`EightZError::MissingCoder`] — the codec is known but its feature flag is
///   disabled for this build.
/// - [`EightZError::UnsupportedMethod`] — the method ID is not recognised at all.
pub fn coder_for(method_id: &MethodId) -> EightZResult<Box<dyn Coder>> {
    let m = method_id.0.as_slice();
    match m {
        [0x00] => Ok(Box::new(CopyCoder)),

        // ── codec stubs — feature-gated ─────────────────────────────────────
        [0x03, 0x01, 0x01] => {
            #[cfg(feature = "lzma")]
            {
                Err(EightZError::not_yet_implemented(
                    "lazippy LZMA coder integration",
                ))
            }
            #[cfg(not(feature = "lzma"))]
            {
                Err(EightZError::missing_coder("LZMA"))
            }
        }
        [0x21] => Err(EightZError::missing_coder("LZMA2")),
        [0x04, 0x01, 0x08] => Err(EightZError::missing_coder("Deflate")),
        [0x04, 0x01, 0x09] => Err(EightZError::missing_coder("Deflate64")),
        [0x04, 0x02, 0x02] => Err(EightZError::missing_coder("BZip2")),
        [0x03, 0x04, 0x01] => Err(EightZError::missing_coder("PPMd")),
        [0x03, 0x03, ..] => Err(EightZError::missing_coder("BCJ family")),
        [0x03] => Err(EightZError::missing_coder("Delta")),
        [0x06, 0xF1, 0x07, 0x01] => Err(EightZError::missing_coder("AES+SHA-256")),

        _ => Err(EightZError::unsupported_method(method_id.0.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_coder_dispatches() {
        let coder = coder_for(&MethodId::copy()).unwrap();
        let data = b"test data";
        let encoded = coder.encode(data).unwrap();
        let decoded = coder.decode(&encoded, data.len() as u64).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn lzma2_is_missing() {
        let result = coder_for(&MethodId::lzma2());
        assert!(matches!(result, Err(EightZError::MissingCoder { .. })));
    }

    #[test]
    fn unknown_method_is_unsupported() {
        let unknown = MethodId(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let result = coder_for(&unknown);
        assert!(matches!(result, Err(EightZError::UnsupportedMethod { .. })));
    }
}
