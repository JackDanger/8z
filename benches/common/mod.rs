//! Shared helpers for bench targets. Bench targets can't import `src/tests/`
//! (cfg(test)-only), so we duplicate the generators here.

#[allow(dead_code)]
pub fn random(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state & 0xFF) as u8
        })
        .collect()
}

#[allow(dead_code)]
pub fn zeros(len: usize) -> Vec<u8> {
    vec![0u8; len]
}

#[allow(dead_code)]
pub fn sequential(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 256) as u8).collect()
}

/// Load a fixture from corpora/fixtures/archives/ relative to CARGO_MANIFEST_DIR.
#[allow(dead_code)]
pub fn load_fixture(name: &str) -> Vec<u8> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("corpora/fixtures/archives")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path.display(), e))
}
