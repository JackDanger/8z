//! Folder and bond parsing for 7z container metadata.
//!
//! A *folder* in 7z terminology is a self-contained decompression unit: it
//! holds one or more coders chained together, with bonds describing how the
//! coders' streams connect to each other and to the packed data.

use crate::container::coders::{parse_coder, Coder};
use crate::container::properties::read_uint64;
use crate::error::EightZResult;

// ── Bond ─────────────────────────────────────────────────────────────────────

/// A binding between an output stream of one coder and an input stream of
/// another coder within the same folder.
///
/// `in_index` is the index of the input stream slot being fed; `out_index`
/// identifies which coder output stream feeds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bond {
    /// Index into the folder's flattened input-stream list.
    pub in_index: u64,
    /// Index into the folder's flattened output-stream list.
    pub out_index: u64,
}

// ── UnpackSize ────────────────────────────────────────────────────────────────

/// Convenience alias: per-output-stream unpack sizes for one folder.
///
/// There is one entry per coder output stream (usually `num_coders` for simple
/// single-output coders). Filled in by [`parse_unpack_info`] separately from
/// [`parse_folder`] because the spec stores them in a sibling `CodersUnpackSize`
/// block.
pub type UnpackSize = u64;

// ── Folder ───────────────────────────────────────────────────────────────────

/// A self-contained decompression unit containing one or more coders.
#[derive(Clone, Debug)]
pub struct Folder {
    /// Coder chain for this folder, in order (first = outermost).
    pub coders: Vec<Coder>,
    /// Bonds between coder streams (num_total_out - num_folders = num_bonds).
    /// For a simple single-coder folder this is empty.
    pub bonds: Vec<Bond>,
    /// Which packed-stream indices feed into this folder (one per unbound
    /// input stream).
    pub packed_stream_indices: Vec<u64>,
    /// Unpack sizes: one per coder output stream. Filled by
    /// [`parse_unpack_info`]; empty until then.
    pub unpack_sizes: Vec<UnpackSize>,
    /// CRC32 of the final unpacked output, if present. Filled by
    /// [`parse_unpack_info`]; `None` until then.
    pub unpack_crc: Option<u32>,
}

// ── Folder parser ─────────────────────────────────────────────────────────────

/// Parse a single folder from the cursor.
///
/// This reads:
/// 1. `NumCoders` (UINT64)
/// 2. Each coder (flag byte + method ID + optional streams/properties)
/// 3. Bind pairs: `(num_out_total - 1)` pairs of `(InIndex, OutIndex)` UINT64
/// 4. Pack-stream indices: `(num_in_total - num_bonds)` UINT64 values
///    (usually just one, `0`)
///
/// `unpack_sizes` and `unpack_crc` are left empty/`None` — the caller
/// (`parse_unpack_info`) fills them in.
pub(crate) fn parse_folder(input: &mut &[u8]) -> EightZResult<Folder> {
    let num_coders = read_uint64(input)?;
    let mut coders = Vec::with_capacity(num_coders as usize);
    let mut num_in_total: u64 = 0;
    let mut num_out_total: u64 = 0;

    for _ in 0..num_coders {
        let coder = parse_coder(input)?;
        num_in_total += coder.num_in_streams;
        num_out_total += coder.num_out_streams;
        coders.push(coder);
    }

    // Number of bind pairs = num_out_total - 1 (one output is the "final" output)
    let num_bonds = num_out_total.saturating_sub(1);
    let mut bonds = Vec::with_capacity(num_bonds as usize);
    for _ in 0..num_bonds {
        let in_index = read_uint64(input)?;
        let out_index = read_uint64(input)?;
        bonds.push(Bond {
            in_index,
            out_index,
        });
    }

    // Number of packed-stream indices = num_in_total - num_bonds
    // Per spec: PackedIndices are only stored explicitly when NumPackedStreams > 1.
    // When there is exactly one packed stream, its index is implicitly 0 (sequential).
    let num_pack_streams = num_in_total.saturating_sub(num_bonds);
    let packed_stream_indices: Vec<u64> = if num_pack_streams > 1 {
        let mut indices = Vec::with_capacity(num_pack_streams as usize);
        for _ in 0..num_pack_streams {
            indices.push(read_uint64(input)?);
        }
        indices
    } else {
        // Implicit: exactly one packed stream at sequential index 0
        vec![0]
    };

    Ok(Folder {
        coders,
        bonds,
        packed_stream_indices,
        unpack_sizes: Vec::new(),
        unpack_crc: None,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::coders::MethodId;

    #[test]
    fn parse_single_copy_coder_folder() {
        // NumCoders=1, Coder: flag=0x01 id=0x00 (Copy)
        // No bonds (num_out_total - 1 = 0), 1 pack stream index = 0x00
        let bytes: &[u8] = &[0x01, 0x01, 0x00, 0x00];
        let mut cursor = bytes;
        let folder = parse_folder(&mut cursor).expect("should parse");
        assert_eq!(folder.coders.len(), 1);
        assert_eq!(folder.coders[0].method_id, MethodId::copy());
        assert!(folder.bonds.is_empty());
        assert_eq!(folder.packed_stream_indices, vec![0]);
        assert!(folder.unpack_sizes.is_empty());
        assert!(folder.unpack_crc.is_none());
    }

    #[test]
    fn parse_two_coder_folder() {
        // NumCoders=2 (LZMA2 + BCJ), each simple (1 in, 1 out)
        // num_out_total=2, num_bonds=1, bond: in=0 out=1
        // num_in_total=2, num_pack_indices = 2 - 1 = 1; index=0
        //
        // Encoding:
        //   NumCoders=2 (0x02)
        //   Coder0: flag=0x01 id=0x21 (LZMA2, simple)
        //   Coder1: flag=0x04 id=0x03,0x03,0x01,0x03 (BCJ x86, simple)
        //   Bond: in_index=0x00 out_index=0x01
        //   PackStreamIndex: 0x00
        let bytes: &[u8] = &[
            0x02, // NumCoders
            0x01, 0x21, // Coder0: flag, id=LZMA2
            0x04, 0x03, 0x03, 0x01, 0x03, // Coder1: flag (id_size=4), id=BCJ
            0x00, 0x01, // Bond: in=0, out=1
            0x00, // PackStreamIndex=0
        ];
        let mut cursor = bytes;
        let folder = parse_folder(&mut cursor).expect("should parse two-coder folder");
        assert_eq!(folder.coders.len(), 2);
        assert_eq!(folder.coders[0].method_id, MethodId::lzma2());
        assert_eq!(folder.coders[1].method_id, MethodId::bcj());
        assert_eq!(folder.bonds.len(), 1);
        assert_eq!(
            folder.bonds[0],
            Bond {
                in_index: 0,
                out_index: 1
            }
        );
        assert_eq!(folder.packed_stream_indices, vec![0]);
    }
}
