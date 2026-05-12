//! Delta filter coder — in-tree implementation (no sibling crate needed).
//!
//! 7z's Delta filter (method ID `[0x03]`) is a simple byte-differencing
//! pre-conditioner. It improves compression of data with regularly-spaced
//! channels (e.g. multi-channel audio or interleaved sample data).
//!
//! # Properties
//!
//! One property byte: `distance - 1` (0 → distance 1, 1 → distance 2, …).
//! A distance of 1 is the most common: each byte is replaced by its difference
//! from the preceding byte.
//!
//! # Encoding
//!
//! For each position `i`, with a circular buffer of length `distance`:
//! ```text
//! out[i] = (in[i] - buf[i % distance]) & 0xFF
//! buf[i % distance] = in[i]
//! ```
//!
//! # Decoding
//!
//! ```text
//! out[i] = (packed[i] + buf[i % distance]) & 0xFF
//! buf[i % distance] = out[i]
//! ```
//!
//! Both operations wrap modulo 256.

use crate::container::MethodId;
use crate::error::SevenZippyResult;
use crate::pipeline::Coder;

/// Maximum supported distance value (matches 7z's own limit: 256 channels).
const MAX_DISTANCE: usize = 256;

/// Delta filter coder backed by an in-tree pure-Rust implementation.
pub struct DeltaCoder {
    /// Byte difference distance: 1 = subtract/add adjacent bytes.
    /// Stored as-is; `properties()` returns `(distance - 1)` as one byte.
    distance: usize,
}

impl DeltaCoder {
    /// Create a Delta coder with the given byte distance.
    ///
    /// `distance` must be in `1..=256`. Values outside this range are clamped.
    pub fn new(distance: usize) -> Self {
        let distance = distance.clamp(1, MAX_DISTANCE);
        Self { distance }
    }

    /// Create a Delta coder from the 1-byte properties blob stored in an archive.
    ///
    /// The property byte is `distance - 1`, so 0x00 → distance 1 (byte delta).
    pub fn from_props(props: &[u8]) -> SevenZippyResult<Self> {
        let byte = props.first().copied().unwrap_or(0);
        let distance = byte as usize + 1;
        Ok(Self { distance })
    }

    /// Apply the Delta encoding transformation.
    fn apply_encode(&self, input: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; self.distance];
        let mut out = Vec::with_capacity(input.len());
        for (i, &b) in input.iter().enumerate() {
            let channel = i % self.distance;
            let delta = b.wrapping_sub(buf[channel]);
            out.push(delta);
            buf[channel] = b;
        }
        out
    }

    /// Apply the Delta decoding transformation.
    fn apply_decode(&self, packed: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; self.distance];
        let mut out = Vec::with_capacity(packed.len());
        for (i, &b) in packed.iter().enumerate() {
            let channel = i % self.distance;
            let restored = b.wrapping_add(buf[channel]);
            out.push(restored);
            buf[channel] = restored;
        }
        out
    }
}

impl Coder for DeltaCoder {
    fn decode(&self, packed: &[u8], _unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        Ok(self.apply_decode(packed))
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        Ok(self.apply_encode(unpacked))
    }

    fn method_id(&self) -> MethodId {
        MethodId::delta()
    }

    fn properties(&self) -> Vec<u8> {
        // The 7z spec stores `distance - 1` as a single property byte.
        vec![(self.distance - 1) as u8]
    }
}

impl Default for DeltaCoder {
    fn default() -> Self {
        Self::new(1)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_round_trip_distance_1() {
        let coder = DeltaCoder::new(1);
        let input = b"Hello, Delta filter!";
        let encoded = coder.encode(input).unwrap();
        let decoded = coder.decode(&encoded, input.len() as u64).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn identity_round_trip_distance_4() {
        let coder = DeltaCoder::new(4);
        let input: Vec<u8> = (0u8..=255).cycle().take(1024).collect();
        let encoded = coder.encode(&input).unwrap();
        let decoded = coder.decode(&encoded, input.len() as u64).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn encode_known_values_distance_1() {
        let coder = DeltaCoder::new(1);
        // Input: [1, 3, 6, 10, 15, 21] → differences: [1, 2, 3, 4, 5, 6]
        let input = [1u8, 3, 6, 10, 15, 21];
        let encoded = coder.encode(&input).unwrap();
        assert_eq!(encoded, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn decode_known_values_distance_1() {
        let coder = DeltaCoder::new(1);
        // Packed: [1, 2, 3, 4, 5, 6] → cumulative sums: [1, 3, 6, 10, 15, 21]
        let packed = [1u8, 2, 3, 4, 5, 6];
        let decoded = coder.decode(&packed, packed.len() as u64).unwrap();
        assert_eq!(decoded, vec![1, 3, 6, 10, 15, 21]);
    }

    #[test]
    fn wrap_around_is_modulo_256() {
        let coder = DeltaCoder::new(1);
        // 5 - 250 = -245 → wraps to 11 (mod 256)
        let input = [250u8, 5];
        let encoded = coder.encode(&input).unwrap();
        assert_eq!(encoded[0], 250); // first byte unchanged (prev=0)
        assert_eq!(encoded[1], 5u8.wrapping_sub(250)); // = 11
        let decoded = coder.decode(&encoded, input.len() as u64).unwrap();
        assert_eq!(decoded, input.to_vec());
    }

    #[test]
    fn from_props_distance_1() {
        let coder = DeltaCoder::from_props(&[0x00]).unwrap();
        assert_eq!(coder.distance, 1);
        assert_eq!(coder.properties(), vec![0x00]);
    }

    #[test]
    fn from_props_distance_3() {
        let coder = DeltaCoder::from_props(&[0x02]).unwrap();
        assert_eq!(coder.distance, 3);
        assert_eq!(coder.properties(), vec![0x02]);
    }

    #[test]
    fn empty_input_round_trips() {
        let coder = DeltaCoder::new(1);
        let encoded = coder.encode(&[]).unwrap();
        assert!(encoded.is_empty());
        let decoded = coder.decode(&[], 0).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn large_random_round_trip() {
        let coder = DeltaCoder::new(2);
        let input: Vec<u8> = (0u16..=65535).map(|x| (x * 7 + 13) as u8).collect();
        let encoded = coder.encode(&input).unwrap();
        let decoded = coder.decode(&encoded, input.len() as u64).unwrap();
        assert_eq!(decoded, input);
    }
}
