//! Layer 2: method-ID dispatch tests.
//!
//! Verifies that `pipeline::coder_for(method_id)` returns the right variant
//! for each method ID class:
//! - Recognised + enabled  → `Ok(Box<dyn Coder>)`
//! - Recognised + disabled → `Err(EightZError::MissingCoder { .. })`
//! - Unrecognised           → `Err(EightZError::UnsupportedMethod { .. })`
//!
//! These tests exercise the dispatch layer in isolation, without running any
//! actual compression/decompression.

use crate::container::MethodId;
use crate::error::EightZError;
use crate::pipeline::coder_for;

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

// ── LZMA: feature-gated — currently returns NotYetImplemented when `lzma` feature is on ──

/// When built with `--features lzma` (the default), the `lzma` feature is
/// enabled but lazippy's integration is a stub that returns
/// `EightZError::NotYetImplemented`.
///
/// When built without the `lzma` feature, dispatch returns
/// `EightZError::MissingCoder { name: "LZMA" }`.
#[test]
fn lzma_coder_is_not_yet_ready() {
    let result = coder_for(&MethodId::lzma());
    match result {
        Err(EightZError::NotYetImplemented(_)) => {
            // lzma feature is enabled but integration not wired — expected.
        }
        Err(EightZError::MissingCoder { name: "LZMA" }) => {
            // lzma feature is disabled — also expected.
        }
        Ok(_) => panic!("LZMA dispatch should not succeed yet"),
        Err(e) => panic!("unexpected error for LZMA dispatch: {e}"),
    }
}

// ── LZMA2 / BZip2 / Deflate / PPMd: MissingCoder ────────────────────────────

fn assert_missing_coder(method: &MethodId, name: &str) {
    let result = coder_for(method);
    match result {
        Err(EightZError::MissingCoder { .. }) => {}
        Ok(_) => panic!("{name} dispatch returned Ok — expected MissingCoder"),
        Err(e) => panic!("expected MissingCoder for {name}, got: {e}"),
    }
}

fn assert_unsupported(method: &MethodId, label: &str) {
    let result = coder_for(method);
    match result {
        Err(EightZError::UnsupportedMethod { .. }) => {}
        Ok(_) => panic!("{label} dispatch returned Ok — expected UnsupportedMethod"),
        Err(e) => panic!("expected UnsupportedMethod for {label}, got: {e}"),
    }
}

#[test]
fn lzma2_is_missing() {
    assert_missing_coder(&MethodId::lzma2(), "LZMA2");
}

#[test]
fn bzip2_is_missing() {
    assert_missing_coder(&MethodId::bzip2(), "BZip2");
}

#[test]
fn deflate_is_missing() {
    assert_missing_coder(&MethodId::deflate(), "Deflate");
}

#[test]
fn deflate64_is_missing() {
    assert_missing_coder(&MethodId::deflate64(), "Deflate64");
}

#[test]
fn ppmd_is_missing() {
    assert_missing_coder(&MethodId::ppmd(), "PPMd");
}

#[test]
fn bcj_is_missing() {
    assert_missing_coder(&MethodId::bcj(), "BCJ");
}

#[test]
fn delta_is_missing() {
    assert_missing_coder(&MethodId::delta(), "Delta");
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
