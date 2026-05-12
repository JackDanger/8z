//! AES-256-CBC multi-coder folder decoder.
//!
//! 7z AES archives use a 2-coder folder topology:
//!
//! ```text
//! Coders (in order):
//!   [0] AES   (1 in, 1 out)  — decrypts the packed stream
//!   [1] LZMA2 (1 in, 1 out)  — decompresses the decrypted stream
//!
//! Bond:
//!   Bond { in_index: 1, out_index: 0 }
//!     → AES output (stream index 0) feeds LZMA2's input (stream index 1)
//!
//! Packed stream:
//!   packed[0] → AES input
//!             → AES decrypts → LZMA2 input
//!                            → LZMA2 decompresses → final output
//! ```
//!
//! The inner compressor is typically LZMA2 (the 7zz default), but may be
//! any single-stream coder. This Phase 1 implementation handles only
//! AES+LZMA2.
//!
//! ## Password API
//!
//! AES decryption requires a user-supplied password. The password is passed
//! separately from the packed data and is not stored in the archive.

use crate::error::{SevenZippyError, SevenZippyResult};

/// Is this a canonical AES+LZMA2 folder?
///
/// Returns `true` if the folder has:
/// - Exactly 2 coders: AES (`[0x06,0xF1,0x07,0x01]`) then LZMA2 (`[0x21]`)
/// - Exactly 1 bond: in_index=1, out_index=0
/// - Exactly 1 packed stream index
pub fn is_aes_lzma2_folder(folder: &crate::container::Folder) -> bool {
    use crate::container::MethodId;
    if folder.coders.len() != 2 {
        return false;
    }
    if folder.bonds.len() != 1 {
        return false;
    }
    if folder.packed_stream_indices.len() != 1 {
        return false;
    }
    let aes_id = MethodId(vec![0x06, 0xF1, 0x07, 0x01]);
    let lzma2_id = MethodId(vec![0x21]);
    folder.coders[0].method_id == aes_id && folder.coders[1].method_id == lzma2_id
}

/// Is this any folder encrypted with AES (regardless of the inner coder)?
///
/// Returns `true` if the first coder's method ID is `[0x06,0xF1,0x07,0x01]`.
/// Used by `decode_folder` to route to `decode_aes_folder` and to give the
/// caller a useful error if no password was provided.
pub fn has_aes_coder(folder: &crate::container::Folder) -> bool {
    use crate::container::MethodId;
    let aes_id = MethodId(vec![0x06, 0xF1, 0x07, 0x01]);
    folder.coders.first().is_some_and(|c| c.method_id == aes_id)
}

/// Decode an AES+LZMA2 folder from a single packed stream byte slice.
///
/// # Arguments
///
/// - `folder`: the folder metadata (coders + bonds + unpack_sizes)
/// - `packed`: the single packed (encrypted) byte stream
/// - `password`: the archive password (UTF-8)
///
/// # Errors
///
/// - `NotYetImplemented` if the folder is not the AES+LZMA2 topology
/// - `InvalidArgument` if the password is empty and the archive is encrypted
/// - Propagates decrypt errors from lockzippy and decompress errors from lazippier
#[cfg(feature = "aes")]
pub fn decode_aes_folder(
    folder: &crate::container::Folder,
    packed: &[u8],
    password: &str,
) -> SevenZippyResult<Vec<u8>> {
    if !is_aes_lzma2_folder(folder) {
        return Err(SevenZippyError::not_yet_implemented(
            "AES folder with non-LZMA2 inner coder",
        ));
    }

    // Step 1: AES-256-CBC decrypt.
    // folder.coders[0] = AES; its properties contain NumCyclesPower + IV.
    let aes_coder = &folder.coders[0];
    let decrypted = lockzippy::decrypt::decrypt_7z(packed, &aes_coder.properties, password)
        .map_err(|e| SevenZippyError::Coder(Box::new(e)))?;

    // Step 2: LZMA2 decompress.
    // folder.coders[1] = LZMA2; its properties contain the props byte.
    // unpack_sizes[1] is the LZMA2 output size (= final uncompressed size).
    let lzma2_coder = &folder.coders[1];
    let unpack_size = folder.unpack_sizes.last().copied().unwrap_or(0);

    use crate::pipeline::lzma2::Lzma2Coder;
    use crate::pipeline::Coder;
    let lzma2 = Lzma2Coder::with_props(lzma2_coder.properties.clone())?;
    lzma2.decode(&decrypted, unpack_size)
}
