//! Layer 2: method-ID dispatch tests.
//!
//! Verifies that `pipeline::coder_for(method_id)` returns the right variant
//! for each method ID class:
//! - Recognised + enabled  → `Ok(Box<dyn Coder>)`
//! - Recognised + disabled → `Err(SevenZippyError::MissingCoder { .. })`
//! - Unrecognised           → `Err(SevenZippyError::UnsupportedMethod { .. })`
//!
//! These tests exercise the dispatch layer in isolation, without running any
//! actual compression/decompression.

use crate::container::MethodId;
use crate::error::SevenZippyError;
use crate::pipeline::coder_for_method as coder_for;

// ── Copy coder (always enabled, in-tree) ─────────────────────────────────────

#[test]
fn copy_coder_dispatches_ok() {
    let coder = coder_for(&MethodId::copy()).expect("Copy coder must always be available");
    // Verify the method_id round-trips through the trait.
    assert_eq!(coder.method_id(), MethodId::copy());
}

#[test]
fn copy_coder_is_identity_on_small_input() {
    let coder = coder_for(&MethodId::copy()).unwrap();
    let input = b"dispatch test";
    let encoded = coder.encode(input).unwrap();
    let decoded = coder.decode(&encoded, input.len() as u64).unwrap();
    assert_eq!(decoded, input);
}

// ── LZMA: feature-gated — wired via lazippy ──────────────────────────────────

/// When built with `--features lzma` (the default), dispatch returns a live
/// `LzmaCoder` backed by lazippy.
///
/// When built without the `lzma` feature, dispatch returns
/// `SevenZippyError::MissingCoder { name: "LZMA" }`.
#[test]
fn lzma_coder_dispatches() {
    let result = coder_for(&MethodId::lzma());
    #[cfg(feature = "lzma")]
    {
        // LZMA is wired — must succeed.
        let coder = result.expect("LZMA coder must be available when lzma feature is enabled");
        assert_eq!(coder.method_id(), MethodId::lzma());
    }
    #[cfg(not(feature = "lzma"))]
    {
        // Feature disabled — must return MissingCoder.
        assert!(
            matches!(result, Err(SevenZippyError::MissingCoder { .. })),
            "expected MissingCoder when lzma feature is disabled, got {result:?}"
        );
    }
}

// ── LZMA2 / BZip2 / Deflate / PPMd: MissingCoder ────────────────────────────

fn assert_missing_coder(method: &MethodId, name: &str) {
    let result = coder_for(method);
    match result {
        Err(SevenZippyError::MissingCoder { .. }) => {}
        Ok(_) => panic!("{name} dispatch returned Ok — expected MissingCoder"),
        Err(e) => panic!("expected MissingCoder for {name}, got: {e}"),
    }
}

fn assert_unsupported(method: &MethodId, label: &str) {
    let result = coder_for(method);
    match result {
        Err(SevenZippyError::UnsupportedMethod { .. }) => {}
        Ok(_) => panic!("{label} dispatch returned Ok — expected UnsupportedMethod"),
        Err(e) => panic!("expected UnsupportedMethod for {label}, got: {e}"),
    }
}

#[test]
fn lzma2_is_missing() {
    assert_missing_coder(&MethodId::lzma2(), "LZMA2");
}

/// BZip2 dispatches to Bzip2Coder when the `bzip2` feature is enabled,
/// or returns MissingCoder when the feature is disabled.
#[test]
fn bzip2_dispatches_or_is_missing() {
    let result = coder_for(&MethodId::bzip2());
    #[cfg(feature = "bzip2")]
    {
        let coder = result.expect("BZip2 coder must be available when bzip2 feature is enabled");
        assert_eq!(coder.method_id(), MethodId::bzip2());
    }
    #[cfg(not(feature = "bzip2"))]
    {
        assert!(
            matches!(result, Err(SevenZippyError::MissingCoder { .. })),
            "expected MissingCoder when bzip2 feature is disabled, got {result:?}"
        );
    }
}

/// Deflate dispatches to DeflateCoder when the `deflate` feature is enabled,
/// or returns MissingCoder when the feature is disabled.
#[test]
fn deflate_dispatches_or_is_missing() {
    let result = coder_for(&MethodId::deflate());
    #[cfg(feature = "deflate")]
    {
        let coder =
            result.expect("Deflate coder must be available when deflate feature is enabled");
        assert_eq!(coder.method_id(), MethodId::deflate());
    }
    #[cfg(not(feature = "deflate"))]
    {
        assert!(
            matches!(result, Err(SevenZippyError::MissingCoder { .. })),
            "expected MissingCoder when deflate feature is disabled, got {result:?}"
        );
    }
}

#[test]
fn deflate64_is_missing() {
    assert_missing_coder(&MethodId::deflate64(), "Deflate64");
}

/// When built with `--features ppmd` (the default), dispatch returns a live
/// `PpmdCoder`. When built without, returns `MissingCoder`.
#[test]
fn ppmd_dispatches_or_is_missing() {
    use crate::container::Coder as CoderMeta;
    // Supply valid 5-byte PPMd7 properties (order=6, mem_size=16MiB) so
    // PpmdCoder::from_props doesn't fail with a length error.
    let props = {
        let order: u8 = 6;
        let mem_size: u32 = 16 * 1024 * 1024;
        let mut v = vec![order];
        v.extend_from_slice(&mem_size.to_le_bytes());
        v
    };
    let meta = CoderMeta {
        method_id: MethodId::ppmd(),
        num_in_streams: 1,
        num_out_streams: 1,
        properties: props,
    };
    let result = crate::pipeline::coder_for(&meta);
    #[cfg(feature = "ppmd")]
    {
        let coder = result.expect("PPMd coder must be available when ppmd feature is enabled");
        assert_eq!(coder.method_id(), MethodId::ppmd());
    }
    #[cfg(not(feature = "ppmd"))]
    {
        assert!(matches!(result, Err(SevenZippyError::MissingCoder { .. })));
    }
}

/// BCJ x86 dispatches to BcjCoder when the `bcj` feature is enabled,
/// or returns MissingCoder when the feature is disabled.
#[test]
fn bcj_dispatches_or_is_missing() {
    let result = coder_for(&MethodId::bcj());
    #[cfg(feature = "bcj")]
    {
        let coder = result.expect("BCJ coder must be available when bcj feature is enabled");
        assert_eq!(coder.method_id(), MethodId::bcj());
    }
    #[cfg(not(feature = "bcj"))]
    {
        assert!(
            matches!(result, Err(SevenZippyError::MissingCoder { .. })),
            "expected MissingCoder when bcj feature is disabled, got {result:?}"
        );
    }
}

/// BCJ2 is not yet implemented regardless of feature flags.
#[test]
fn bcj2_is_missing() {
    assert_missing_coder(&MethodId::bcj2(), "BCJ2");
}

/// When built with `--features delta` (the default), dispatch returns a live
/// `DeltaCoder`. When built without, returns `MissingCoder`.
#[test]
fn delta_dispatches_or_is_missing() {
    let result = coder_for(&MethodId::delta());
    #[cfg(feature = "delta")]
    {
        let coder = result.expect("Delta coder must be available when delta feature is enabled");
        assert_eq!(coder.method_id(), MethodId::delta());
    }
    #[cfg(not(feature = "delta"))]
    {
        assert!(matches!(result, Err(SevenZippyError::MissingCoder { .. })));
    }
}

// ── Completely unknown method ID: UnsupportedMethod ──────────────────────────

#[test]
fn unknown_method_id_is_unsupported() {
    assert_unsupported(&MethodId(vec![0xAA, 0xBB]), "0xAA 0xBB");
}

#[test]
fn random_long_method_id_is_unsupported() {
    assert_unsupported(
        &MethodId(vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02]),
        "0xDEADBEEF...",
    );
}
