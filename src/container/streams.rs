//! Packed-stream and unpacked-stream accounting.
//!
//! `PackedStreams` describes the byte ranges of compressed data within the
//! archive. `UnpackedStreams` groups those into folders (decompression units)
//! and exposes per-file boundaries via sub-stream information.

use crate::container::folders::Folder;
use crate::container::properties::{
    read_bit_vector, read_u32_le, read_u8, read_uint64, PropertyId,
};
use crate::error::{EightZError, EightZResult};

// ── PackedStreams ─────────────────────────────────────────────────────────────

/// Describes the layout of packed (compressed) data in the archive body.
///
/// Corresponds to the `PackInfo` block (PropertyId 0x06).
#[derive(Clone, Debug)]
pub struct PackedStreams {
    /// Byte offset of the packed data relative to the end of the signature
    /// header (i.e. relative to byte 32 of the file).
    pub pack_pos: u64,
    /// Byte length of each packed stream.
    pub pack_sizes: Vec<u64>,
    /// Optional CRC32 per packed stream (`None` if not stored in the archive).
    pub pack_crcs: Vec<Option<u32>>,
}

/// Parse a `PackInfo` block.
///
/// The cursor must be positioned immediately after the `PackInfo` (0x06) tag
/// byte. Reading continues until the `End` (0x00) tag is consumed.
pub(crate) fn parse_pack_info(input: &mut &[u8]) -> EightZResult<PackedStreams> {
    let pack_pos = read_uint64(input)?;
    let num_pack = read_uint64(input)? as usize;

    let mut pack_sizes = vec![0u64; num_pack];
    let mut pack_crcs: Vec<Option<u32>> = vec![None; num_pack];

    loop {
        let tag_byte = read_u8(input)?;
        match PropertyId::from_u8(tag_byte)? {
            PropertyId::End => break,
            PropertyId::Size => {
                for s in pack_sizes.iter_mut() {
                    *s = read_uint64(input)?;
                }
            }
            PropertyId::Crc => {
                let all_defined = read_u8(input)?;
                let defined: Vec<bool> = if all_defined != 0 {
                    vec![true; num_pack]
                } else {
                    read_bit_vector(input, num_pack)?
                };
                for (i, &def) in defined.iter().enumerate() {
                    if def {
                        pack_crcs[i] = Some(read_u32_le(input)?);
                    }
                }
            }
            other => {
                return Err(EightZError::invalid_header(format!(
                    "unexpected property {other:?} in PackInfo"
                )));
            }
        }
    }

    Ok(PackedStreams {
        pack_pos,
        pack_sizes,
        pack_crcs,
    })
}

// ── UnpackedStreams ───────────────────────────────────────────────────────────

/// The unpacked-stream view: one or more folders, each folder covering one or
/// more sub-streams (files).
///
/// For Phase C we only support 1 file per folder (the common case for
/// Copy-only archives). The full sub-stream info is parsed but sub-stream
/// sizes beyond the first are preserved for future use.
#[derive(Clone, Debug)]
pub struct UnpackedStreams {
    /// All folders in this archive.
    pub folders: Vec<Folder>,
    /// Number of sub-streams (files) per folder.
    pub num_unpack_streams: Vec<u64>,
    /// Sub-stream sizes (one entry per file, flattened across all folders).
    /// The last sub-stream of each folder has its size inferred from the
    /// folder's total unpack size minus the sum of the others.
    pub unpack_stream_sizes: Vec<u64>,
    /// CRC32 per sub-stream, flattened.
    pub unpack_stream_crcs: Vec<Option<u32>>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pack_info_simple() {
        // PackInfo for 1 pack stream of size 19, no CRC
        // pack_pos=0 num_pack=1 Size(0x09) 19 End(0x00)
        let bytes: Vec<u8> = vec![
            0x00, // pack_pos = 0
            0x01, // num_pack = 1
            0x09, // Size tag
            0x13, // size[0] = 19
            0x00, // End
        ];
        let mut cursor: &[u8] = &bytes;
        let ps = parse_pack_info(&mut cursor).expect("should parse");
        assert_eq!(ps.pack_pos, 0);
        assert_eq!(ps.pack_sizes, vec![19]);
        assert_eq!(ps.pack_crcs, vec![None]);
    }

    #[test]
    fn parse_pack_info_with_crc() {
        // PackInfo with 1 stream, size 10, CRC 0xDEADBEEF
        let crc_bytes: [u8; 4] = 0xDEAD_BEEF_u32.to_le_bytes();
        let mut bytes: Vec<u8> = vec![
            0x00, // pack_pos = 0
            0x01, // num_pack = 1
            0x09, // Size
            0x0A, // size=10
            0x0A, // Crc tag
            0x01, // all_defined = true
        ];
        bytes.extend_from_slice(&crc_bytes);
        bytes.push(0x00); // End
        let mut cursor: &[u8] = &bytes;
        let ps = parse_pack_info(&mut cursor).expect("should parse");
        assert_eq!(ps.pack_sizes, vec![10]);
        assert_eq!(ps.pack_crcs, vec![Some(0xDEAD_BEEF)]);
    }
}
