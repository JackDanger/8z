//! Coder pipeline dispatch: given a `Folder` and packed bytes, produce unpacked bytes.
//!
//! The `pipeline` module is the bridge between the container parser (`container::`)
//! and the codec implementations (in-tree Copy + sibling crates). `dispatch.rs` is
//! the single place where each codec plugs in.

#[cfg(feature = "bcj")]
pub mod bcj;
#[cfg(feature = "bzip2")]
pub mod bzip2;
mod coder_trait;
mod copy;
#[cfg(feature = "deflate")]
pub mod deflate;
#[cfg(feature = "delta")]
pub mod delta;
mod dispatch;
#[cfg(feature = "lzma")]
pub mod lzma;
#[cfg(feature = "ppmd")]
pub mod ppmd;

pub use coder_trait::Coder;
pub use copy::CopyCoder;
pub use dispatch::{coder_for, coder_for_method};

use crate::container::crc::crc32;
use crate::container::{Coder as CoderMeta, Folder, MethodId};
use crate::error::{SevenZippyError, SevenZippyResult};

/// Decode one folder's packed bytes through its coder pipeline, producing
/// the final unpacked stream.
///
/// For Phase C: only single-coder folders (no bonds) are supported.
pub fn decode_folder(folder: &Folder, packed: &[u8]) -> SevenZippyResult<Vec<u8>> {
    if folder.coders.len() != 1 || !folder.bonds.is_empty() {
        return Err(SevenZippyError::not_yet_implemented(
            "multi-coder folder pipeline",
        ));
    }
    let coder_meta = &folder.coders[0];
    let coder = coder_for(coder_meta)?;
    let unpack_size = folder
        .unpack_sizes
        .first()
        .copied()
        .unwrap_or(packed.len() as u64);
    coder.decode(packed, unpack_size)
}

/// Encode a single byte slice into a single-coder folder using `coder`.
///
/// Returns `(packed_bytes, folder_metadata)`.
pub fn encode_single_coder_folder(
    coder: &dyn Coder,
    unpacked: &[u8],
) -> SevenZippyResult<(Vec<u8>, Folder)> {
    let packed = coder.encode(unpacked)?;
    let folder = Folder {
        coders: vec![CoderMeta {
            method_id: coder.method_id(),
            num_in_streams: 1,
            num_out_streams: 1,
            properties: coder.properties(),
        }],
        bonds: vec![],
        packed_stream_indices: vec![0],
        unpack_sizes: vec![unpacked.len() as u64],
        unpack_crc: Some(crc32(unpacked)),
    };
    Ok((packed, folder))
}

/// Convenience: encode using the in-tree Copy coder.
pub fn encode_copy_folder(unpacked: &[u8]) -> SevenZippyResult<(Vec<u8>, Folder)> {
    encode_single_coder_folder(&CopyCoder, unpacked)
}

/// Return the method ID for the Copy coder.
pub fn copy_method_id() -> MethodId {
    MethodId::copy()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_copy_folder_identity() {
        let data = b"hello world";
        let (packed, folder) = encode_copy_folder(data).unwrap();
        let unpacked = decode_folder(&folder, &packed).unwrap();
        assert_eq!(unpacked, data);
    }

    #[test]
    fn encode_copy_folder_sets_crc() {
        let data = b"test";
        let (_, folder) = encode_copy_folder(data).unwrap();
        assert!(folder.unpack_crc.is_some());
    }

    #[test]
    fn multi_coder_folder_is_not_yet_implemented() {
        use crate::container::{Bond, Coder as CoderMeta};
        let folder = Folder {
            coders: vec![
                CoderMeta {
                    method_id: MethodId::copy(),
                    num_in_streams: 1,
                    num_out_streams: 1,
                    properties: vec![],
                },
                CoderMeta {
                    method_id: MethodId::copy(),
                    num_in_streams: 1,
                    num_out_streams: 1,
                    properties: vec![],
                },
            ],
            bonds: vec![Bond {
                in_index: 0,
                out_index: 1,
            }],
            packed_stream_indices: vec![0],
            unpack_sizes: vec![5],
            unpack_crc: None,
        };
        let result = decode_folder(&folder, b"hello");
        assert!(matches!(result, Err(SevenZippyError::NotYetImplemented(_))));
    }
}
