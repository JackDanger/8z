//! CRC-32 helpers used for 7z header validation.

/// Compute the CRC-32 of `bytes` using the standard IEEE polynomial (same as
/// `crc32fast`).
pub(crate) fn crc32(bytes: &[u8]) -> u32 {
    let mut h = crc32fast::Hasher::new();
    h.update(bytes);
    h.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector() {
        // CRC32 of empty is 0x00000000
        assert_eq!(crc32(b""), 0x0000_0000);
        // CRC32 of b"123456789" is the well-known check value
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
