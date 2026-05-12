//! Deterministic test data generators for the 8z test suite.
//!
//! All generators are pure functions with no external dependencies.
//! The same `seed` and `len` always produce the same bytes — safe to
//! hard-code expected values in tests and reproduce them on any machine.
//!
//! # PRNG
//!
//! The internal PRNG is a 64-bit xorshift (Marsaglia 2003). It is deliberately
//! simple and **not** cryptographically secure. The seed 0 is advanced once
//! before the first output so that `random(0, n)` still produces non-trivial
//! data.

// ── PRNG ──────────────────────────────────────────────────────────────────────

/// One step of a 64-bit xorshift PRNG (Marsaglia 2003).
#[inline]
fn xorshift64(state: &mut u64) -> u64 {
    // Avoid the all-zeros fixed point.
    if *state == 0 {
        *state = 0x1234_5678_9ABC_DEF0;
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

// ── Generators ────────────────────────────────────────────────────────────────

/// `len` pseudo-random bytes seeded with `seed`.
///
/// Deterministic: `random(seed, len) == random(seed, len)` always.
/// Returns an empty `Vec` when `len == 0`.
pub fn random(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed;
    (0..len)
        .map(|_| (xorshift64(&mut state) & 0xFF) as u8)
        .collect()
}

/// `len` zero bytes — maximally compressible.
pub fn zeros(len: usize) -> Vec<u8> {
    vec![0u8; len]
}

/// Repeating `0, 1, 2, …, 255, 0, 1, …` pattern.
///
/// Highly compressible; good for testing run-length and dictionary coders.
pub fn sequential(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 256) as u8).collect()
}

/// Half pseudo-random bytes followed by half zeros — mixed entropy.
///
/// Useful for exercising code paths that handle both compressible and
/// incompressible regions in the same input.
pub fn mixed(seed: u64, len: usize) -> Vec<u8> {
    let half = len / 2;
    let remainder = len - half;
    let mut out = random(seed, half);
    out.extend(zeros(remainder));
    out
}

/// A deterministic ASCII paragraph of `len` bytes built by repeating a fixed
/// English sentence.
///
/// The text is realistic enough to compress well with dictionary coders (LZMA,
/// Deflate, PPMd) without relying on any runtime data.
pub fn ascii_paragraph(len: usize) -> Vec<u8> {
    const SENTENCE: &[u8] = b"The quick brown fox jumps over the lazy dog. \
          Pack my box with five dozen liquor jugs. \
          How vexingly quick daft zebras jump! \
          The five boxing wizards jump quickly. ";

    if len == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        let remaining = len - out.len();
        out.extend_from_slice(&SENTENCE[..remaining.min(SENTENCE.len())]);
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_len_zero_is_empty() {
        assert!(random(42, 0).is_empty());
    }

    #[test]
    fn random_correct_length() {
        assert_eq!(random(1, 100).len(), 100);
        assert_eq!(random(999, 65536).len(), 65536);
    }

    #[test]
    fn random_is_deterministic() {
        assert_eq!(random(0xDEAD_BEEF, 256), random(0xDEAD_BEEF, 256));
    }

    #[test]
    fn random_different_seeds_differ() {
        // Extremely unlikely to collide for 256 bytes.
        assert_ne!(random(1, 256), random(2, 256));
    }

    #[test]
    fn zeros_correct_length() {
        assert_eq!(zeros(0).len(), 0);
        assert_eq!(zeros(1024).len(), 1024);
        assert!(zeros(100).iter().all(|&b| b == 0));
    }

    #[test]
    fn sequential_correct_length() {
        assert_eq!(sequential(0).len(), 0);
        assert_eq!(sequential(512).len(), 512);
    }

    #[test]
    fn sequential_wraps_at_256() {
        let v = sequential(512);
        assert_eq!(v[0], 0);
        assert_eq!(v[255], 255);
        assert_eq!(v[256], 0); // wraps
        assert_eq!(v[511], 255);
    }

    #[test]
    fn mixed_correct_length() {
        assert_eq!(mixed(0, 0).len(), 0);
        assert_eq!(mixed(1, 1000).len(), 1000);
    }

    #[test]
    fn mixed_second_half_is_zeros() {
        let len = 100;
        let v = mixed(42, len);
        // Second half (indices 50..100) must all be zero.
        assert!(v[50..].iter().all(|&b| b == 0));
    }

    #[test]
    fn ascii_paragraph_len_zero_is_empty() {
        assert!(ascii_paragraph(0).is_empty());
    }

    #[test]
    fn ascii_paragraph_correct_length() {
        assert_eq!(ascii_paragraph(10).len(), 10);
        assert_eq!(ascii_paragraph(1024).len(), 1024);
    }

    #[test]
    fn ascii_paragraph_is_ascii() {
        let v = ascii_paragraph(200);
        assert!(v.iter().all(|&b| b.is_ascii()), "all bytes should be ASCII");
    }

    #[test]
    fn ascii_paragraph_is_deterministic() {
        assert_eq!(ascii_paragraph(500), ascii_paragraph(500));
    }
}
