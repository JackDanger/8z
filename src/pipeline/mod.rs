//! Coder pipeline dispatch: given a `Folder` and packed bytes, produce unpacked bytes.
//!
//! The `pipeline` module is the bridge between the container parser (`container::`)
//! and the codec implementations (in-tree Copy + sibling crates). `dispatch.rs` is
//! the single place where each codec plugs in.

#[cfg(feature = "aes")]
pub mod aes_folder;
#[cfg(feature = "bcj")]
pub mod bcj;
#[cfg(feature = "bcj2")]
pub mod bcj2_folder;
#[cfg(feature = "bzip2")]
pub mod bzip2;
mod coder_trait;
mod copy;
#[cfg(feature = "deflate")]
pub mod deflate;
#[cfg(feature = "deflate64")]
pub mod deflate64;
#[cfg(feature = "delta")]
pub mod delta;
mod dispatch;
#[cfg(feature = "lzma")]
pub mod lzma;
#[cfg(feature = "lzma2")]
pub mod lzma2;
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
/// `packed_streams` contains the packed byte slices for this folder, one per
/// unbound input stream (as determined by `folder.packed_stream_indices`).
/// For single-coder folders, exactly one slice is expected. For BCJ2 folders,
/// exactly four slices are expected.
///
/// For AES-encrypted folders, use [`decode_folder_with_password`] instead;
/// this function returns `SevenZippyError::EncryptedContent` for such folders.
pub fn decode_folder(folder: &Folder, packed_streams: &[&[u8]]) -> SevenZippyResult<Vec<u8>> {
    // ── AES-encrypted folder ─────────────────────────────────────────────────
    #[cfg(feature = "aes")]
    if aes_folder::has_aes_coder(folder) {
        return Err(SevenZippyError::encrypted_content(
            "archive is AES-encrypted; use extract_with_password() and supply a password",
        ));
    }

    // ── BCJ2 multi-coder folder ──────────────────────────────────────────────
    #[cfg(feature = "bcj2")]
    if bcj2_folder::is_bcj2_lzma_folder(folder) {
        return bcj2_folder::decode_bcj2_folder(folder, packed_streams);
    }

    // ── Single-coder folder (all other codecs) ───────────────────────────────
    if folder.coders.len() != 1 || !folder.bonds.is_empty() {
        return Err(SevenZippyError::not_yet_implemented(
            "multi-coder folder pipeline",
        ));
    }
    let packed = packed_streams.first().copied().unwrap_or(&[]);
    let coder_meta = &folder.coders[0];
    let coder = coder_for(coder_meta)?;
    let unpack_size = folder
        .unpack_sizes
        .first()
        .copied()
        .unwrap_or(packed.len() as u64);
    coder.decode(packed, unpack_size)
}

/// Decode an AES-encrypted folder using the supplied password.
///
/// For non-AES folders, delegates to [`decode_folder`].
#[cfg(feature = "aes")]
pub fn decode_folder_with_password(
    folder: &Folder,
    packed_streams: &[&[u8]],
    password: &str,
) -> SevenZippyResult<Vec<u8>> {
    if aes_folder::has_aes_coder(folder) {
        let packed = packed_streams.first().copied().unwrap_or(&[]);
        aes_folder::decode_aes_folder(folder, packed, password)
    } else {
        decode_folder(folder, packed_streams)
    }
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
        let unpacked = decode_folder(&folder, &[packed.as_slice()]).unwrap();
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
        let result = decode_folder(&folder, &[b"hello" as &[u8]]);
        assert!(matches!(result, Err(SevenZippyError::NotYetImplemented(_))));
    }
}
