//! Copy coder — the simplest possible codec: identity (memcpy).

use crate::container::MethodId;
use crate::error::{SevenZippyError, SevenZippyResult};
use crate::pipeline::Coder;

/// The Copy coder performs no compression: packed bytes == unpacked bytes.
///
/// This is the only codec implemented in-tree in `7zippy` itself. All other codecs
/// live in sibling crates (lazippy, gzippy, etc.) and plug in via the `Coder` trait.
pub struct CopyCoder;

impl Coder for CopyCoder {
    fn decode(&self, packed: &[u8], unpacked_size: u64) -> SevenZippyResult<Vec<u8>> {
        if packed.len() as u64 != unpacked_size {
            return Err(SevenZippyError::truncated(format!(
                "Copy coder: packed size {} != unpacked size {}",
                packed.len(),
                unpacked_size
            )));
        }
        Ok(packed.to_vec())
    }

    fn encode(&self, unpacked: &[u8]) -> SevenZippyResult<Vec<u8>> {
        Ok(unpacked.to_vec())
    }

    fn method_id(&self) -> MethodId {
        MethodId::copy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_empty() {
        let coder = CopyCoder;
        let packed: &[u8] = &[];
        let decoded = coder.decode(packed, 0).unwrap();
        assert_eq!(decoded, packed);
        let encoded = coder.encode(packed).unwrap();
        assert_eq!(encoded, packed);
    }

    #[test]
    fn copy_single_byte() {
        let coder = CopyCoder;
        let data: &[u8] = &[0x42];
        let encoded = coder.encode(data).unwrap();
        assert_eq!(encoded, data);
        let decoded = coder.decode(&encoded, 1).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn copy_1kib() {
        let coder = CopyCoder;
        let data: Vec<u8> = (0..1024_u16).map(|i| (i & 0xFF) as u8).collect();
        let encoded = coder.encode(&data).unwrap();
        assert_eq!(encoded, data);
        let decoded = coder.decode(&encoded, data.len() as u64).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn copy_64kib() {
        let coder = CopyCoder;
        let data: Vec<u8> = (0..65536_u32).map(|i| (i & 0xFF) as u8).collect();
        let encoded = coder.encode(&data).unwrap();
        assert_eq!(encoded, data);
        let decoded = coder.decode(&encoded, data.len() as u64).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn copy_size_mismatch_is_error() {
        let coder = CopyCoder;
        let packed: &[u8] = &[0x01, 0x02, 0x03];
        let result = coder.decode(packed, 5); // claims 5 bytes but only 3 present
        assert!(result.is_err());
    }
}
