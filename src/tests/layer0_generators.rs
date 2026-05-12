//! Layer 0: sanity tests for the [`fixtures`](super::fixtures) module.
//!
//! These tests verify that the deterministic data generators behave
//! correctly before any codec logic runs. They are intentionally trivial —
//! if these fail, something is wrong with the test infrastructure itself.

use super::fixtures;

// ── Length / non-panic on zero ────────────────────────────────────────────────

#[test]
fn random_zero_len_is_empty() {
    assert!(fixtures::random(0, 0).is_empty());
    assert!(fixtures::random(u64::MAX, 0).is_empty());
}

#[test]
fn zeros_zero_len_is_empty() {
    assert!(fixtures::zeros(0).is_empty());
}

#[test]
fn sequential_zero_len_is_empty() {
    assert!(fixtures::sequential(0).is_empty());
}

#[test]
fn mixed_zero_len_is_empty() {
    assert!(fixtures::mixed(0, 0).is_empty());
}

#[test]
fn ascii_paragraph_zero_len_is_empty() {
    assert!(fixtures::ascii_paragraph(0).is_empty());
}

// ── Determinism ───────────────────────────────────────────────────────────────

#[test]
fn random_is_reproducible() {
    assert_eq!(fixtures::random(42, 1024), fixtures::random(42, 1024));
}

#[test]
fn sequential_is_reproducible() {
    assert_eq!(fixtures::sequential(512), fixtures::sequential(512));
}

#[test]
fn mixed_is_reproducible() {
    assert_eq!(fixtures::mixed(7, 256), fixtures::mixed(7, 256));
}

#[test]
fn ascii_paragraph_is_reproducible() {
    assert_eq!(
        fixtures::ascii_paragraph(300),
        fixtures::ascii_paragraph(300)
    );
}

// ── Content sanity ────────────────────────────────────────────────────────────

#[test]
fn zeros_all_zero() {
    assert!(fixtures::zeros(1000).iter().all(|&b| b == 0));
}

#[test]
fn sequential_modulo_256() {
    let v = fixtures::sequential(512);
    for (i, &b) in v.iter().enumerate() {
        assert_eq!(
            b,
            (i % 256) as u8,
            "byte at index {i} should be {}",
            i % 256
        );
    }
}

#[test]
fn ascii_paragraph_all_ascii() {
    let v = fixtures::ascii_paragraph(1000);
    for (i, &b) in v.iter().enumerate() {
        assert!(b.is_ascii(), "byte at index {i} is not ASCII: 0x{b:02X}");
    }
}

#[test]
fn mixed_second_half_zeros() {
    let len = 200;
    let v = fixtures::mixed(99, len);
    assert_eq!(v.len(), len);
    // The second half (from half..len) should all be zero.
    let half = len / 2;
    assert!(
        v[half..].iter().all(|&b| b == 0),
        "second half of mixed() should be zeros"
    );
}

// ── Different seeds produce different output ──────────────────────────────────

#[test]
fn random_different_seeds_differ() {
    // 256 bytes — collision probability is negligible.
    assert_ne!(
        fixtures::random(1, 256),
        fixtures::random(2, 256),
        "different seeds must produce different random bytes"
    );
}
