//! End-header parsing: the metadata block pointed to by the signature header.
//!
//! The end-header begins with a single `Header` (0x01) or `EncodedHeader`
//! (0x17) tag byte. For Phase C we only support the uncompressed form (0x01);
//! the encoded (compressed) form returns [`SevenZippyError::not_yet_implemented`].
//!
//! Structure (7zFormat.txt §5.3):
//!
//! ```text
//! Header {
//!   MainStreamsInfo (0x04)? {
//!     PackInfo (0x06) { ... }
//!     UnpackInfo (0x07) { ... }
//!     SubStreamsInfo (0x08) { ... }
//!     End (0x00)
//!   }
//!   FilesInfo (0x05) { ... }
//!   End (0x00)
//! }
//! ```

use std::string::String;

use crate::container::folders::{parse_folder, Folder};
use crate::container::properties::{
    read_bit_vector, read_bytes, read_u32_le, read_u64_le, read_u8, read_uint64, PropertyId,
};
use crate::container::streams::{parse_pack_info, PackedStreams, UnpackedStreams};
use crate::error::{SevenZippyError, SevenZippyResult};

// ── FileEntry ─────────────────────────────────────────────────────────────────

/// Metadata for a single file stored in the archive.
#[derive(Clone, Debug, Default)]
pub struct FileEntry {
    /// File name (UTF-8, decoded from the UTF-16LE stored in the archive).
    pub name: String,
    /// Uncompressed file size, if recorded.
    pub size: Option<u64>,
    /// CRC32 of the uncompressed data, if recorded.
    pub crc: Option<u32>,
    /// Modification timestamp as a Windows FILETIME
    /// (100-nanosecond ticks since 1601-01-01 00:00:00 UTC).
    pub mtime: Option<u64>,
    /// Whether this entry represents an empty file.
    pub is_empty: bool,
    /// Whether this is an anti-item (used for incremental archive updates).
    pub is_anti: bool,
    /// Windows file attributes, if recorded.
    pub attributes: Option<u32>,
}

// ── Header ────────────────────────────────────────────────────────────────────

/// Parsed end-header — the complete metadata block of a `.7z` archive.
#[derive(Clone, Debug)]
pub struct Header {
    /// Main-stream info: folder layout + packed-stream sizes + sub-stream info.
    pub main_streams: Option<UnpackedStreams>,
    /// Raw packed-stream geometry (duplicates `main_streams.folders` info but
    /// sometimes parsed separately).
    pub packed_streams: Option<PackedStreams>,
    /// One entry per logical file stored in the archive.
    pub files: Vec<FileEntry>,
}

// ── Top-level header parser ───────────────────────────────────────────────────

/// Parse the end-header block.
///
/// The cursor must point at the first byte of the header block (the
/// `Header` / `EncodedHeader` tag byte). The block is the slice that was
/// located and CRC-validated by [`crate::container::Archive::parse`].
///
/// Returns [`SevenZippyError::not_yet_implemented`] if the header is encoded
/// (compressed — tag 0x17). This is intentional for Phase C.
pub(crate) fn parse(input: &[u8]) -> SevenZippyResult<Header> {
    let mut cursor = input;

    let first_tag = read_u8(&mut cursor)?;
    match first_tag {
        0x01 => {} // plain Header — proceed
        0x17 => {
            // EncodedHeader: the metadata itself is compressed inside a folder.
            // Phase C only supports uncompressed headers. This case is reached
            // when 7zz uses compression for the metadata block, which it
            // normally does for large archives. Copy-only archives always use
            // the plain Header.
            return Err(SevenZippyError::not_yet_implemented("encoded header"));
        }
        other => {
            return Err(SevenZippyError::invalid_header(format!(
                "expected Header (0x01) or EncodedHeader (0x17) tag, got {other:#04x}"
            )));
        }
    }

    let mut main_streams: Option<UnpackedStreams> = None;
    let mut packed_streams: Option<PackedStreams> = None;
    let mut files: Vec<FileEntry> = Vec::new();

    loop {
        let tag_byte = read_u8(&mut cursor)?;
        match PropertyId::from_u8(tag_byte)? {
            PropertyId::End => break,
            PropertyId::MainStreamsInfo => {
                let (ps, us) = parse_main_streams_info(&mut cursor)?;
                packed_streams = Some(ps);
                main_streams = Some(us);
            }
            PropertyId::FilesInfo => {
                files = parse_files_info(&mut cursor)?;
            }
            PropertyId::ArchiveProperties => {
                // Skip: just read property blocks until End
                skip_archive_properties(&mut cursor)?;
            }
            other => {
                return Err(SevenZippyError::invalid_header(format!(
                    "unexpected top-level header property {other:?}"
                )));
            }
        }
    }

    Ok(Header {
        main_streams,
        packed_streams,
        files,
    })
}

// ── MainStreamsInfo ───────────────────────────────────────────────────────────

fn parse_main_streams_info(
    input: &mut &[u8],
) -> SevenZippyResult<(PackedStreams, UnpackedStreams)> {
    let mut opt_pack: Option<PackedStreams> = None;
    let mut opt_unpack: Option<UnpackedStreams> = None;

    loop {
        let tag_byte = read_u8(input)?;
        match PropertyId::from_u8(tag_byte)? {
            PropertyId::End => break,
            PropertyId::PackInfo => {
                opt_pack = Some(parse_pack_info(input)?);
            }
            PropertyId::UnpackInfo => {
                opt_unpack = Some(parse_unpack_info(input)?);
            }
            PropertyId::SubStreamsInfo => {
                // Sub-stream info refines the unpack view (multiple files per
                // folder). We parse it into the already-built UnpackedStreams.
                let us = opt_unpack.take().ok_or_else(|| {
                    SevenZippyError::invalid_header("SubStreamsInfo encountered before UnpackInfo")
                })?;
                opt_unpack = Some(parse_sub_streams_info(input, us)?);
            }
            other => {
                return Err(SevenZippyError::invalid_header(format!(
                    "unexpected property {other:?} in MainStreamsInfo"
                )));
            }
        }
    }

    let pack = opt_pack
        .ok_or_else(|| SevenZippyError::invalid_header("MainStreamsInfo missing PackInfo"))?;
    let unpack = opt_unpack
        .ok_or_else(|| SevenZippyError::invalid_header("MainStreamsInfo missing UnpackInfo"))?;

    Ok((pack, unpack))
}

// ── UnpackInfo ────────────────────────────────────────────────────────────────

fn parse_unpack_info(input: &mut &[u8]) -> SevenZippyResult<UnpackedStreams> {
    // Expect Folder (0x0B) tag
    let tag = read_u8(input)?;
    if tag != PropertyId::Folder as u8 {
        return Err(SevenZippyError::invalid_header(format!(
            "expected Folder (0x0B) in UnpackInfo, got {tag:#04x}"
        )));
    }

    let num_folders = read_uint64(input)? as usize;
    let external = read_u8(input)?;

    let mut folders: Vec<Folder> = Vec::with_capacity(num_folders);
    if external == 0 {
        for _ in 0..num_folders {
            folders.push(parse_folder(input)?);
        }
    } else {
        // External folder data (rarely used) — not supported in Phase C
        let _data_stream_index = read_uint64(input)?;
        return Err(SevenZippyError::not_yet_implemented(
            "external folder data in UnpackInfo",
        ));
    }

    // Now read sibling properties until End
    loop {
        let tag_byte = read_u8(input)?;
        match PropertyId::from_u8(tag_byte)? {
            PropertyId::End => break,
            PropertyId::CodersUnpackSize => {
                // One UINT64 per coder output stream, per folder.
                for folder in folders.iter_mut() {
                    let num_out: usize = folder
                        .coders
                        .iter()
                        .map(|c| c.num_out_streams as usize)
                        .sum();
                    folder.unpack_sizes = Vec::with_capacity(num_out);
                    for _ in 0..num_out {
                        folder.unpack_sizes.push(read_uint64(input)?);
                    }
                }
            }
            PropertyId::Crc => {
                // Folder-level CRCs (one per folder)
                let all_defined = read_u8(input)?;
                let defined: Vec<bool> = if all_defined != 0 {
                    vec![true; num_folders]
                } else {
                    read_bit_vector(input, num_folders)?
                };
                for (i, &def) in defined.iter().enumerate() {
                    if def {
                        folders[i].unpack_crc = Some(read_u32_le(input)?);
                    }
                }
            }
            other => {
                return Err(SevenZippyError::invalid_header(format!(
                    "unexpected property {other:?} in UnpackInfo"
                )));
            }
        }
    }

    Ok(UnpackedStreams {
        num_unpack_streams: vec![1; num_folders],
        unpack_stream_sizes: Vec::new(),
        unpack_stream_crcs: Vec::new(),
        folders,
    })
}

// ── SubStreamsInfo ────────────────────────────────────────────────────────────

fn parse_sub_streams_info(
    input: &mut &[u8],
    mut us: UnpackedStreams,
) -> SevenZippyResult<UnpackedStreams> {
    let num_folders = us.folders.len();
    // Defaults: 1 sub-stream per folder
    us.num_unpack_streams = vec![1; num_folders];

    loop {
        let tag_byte = read_u8(input)?;
        match PropertyId::from_u8(tag_byte)? {
            PropertyId::End => break,
            PropertyId::NumUnpackStream => {
                for n in us.num_unpack_streams.iter_mut() {
                    *n = read_uint64(input)?;
                }
            }
            PropertyId::Size => {
                // Sizes for sub-streams that are not the last in their folder
                // (the last one's size is inferred).
                us.unpack_stream_sizes.clear();
                for (fi, &n_sub) in us.num_unpack_streams.iter().enumerate() {
                    // The last sub-stream size in each folder is inferred from
                    // the folder's total unpack size.
                    for si in 0..n_sub {
                        if si < n_sub - 1 {
                            us.unpack_stream_sizes.push(read_uint64(input)?);
                        } else {
                            // Infer: folder_unpack - sum_of_earlier
                            let folder_total =
                                us.folders[fi].unpack_sizes.last().copied().unwrap_or(0);
                            let already: u64 = us
                                .unpack_stream_sizes
                                .iter()
                                .rev()
                                .take((n_sub - 1) as usize)
                                .sum();
                            us.unpack_stream_sizes
                                .push(folder_total.saturating_sub(already));
                        }
                    }
                }
            }
            PropertyId::Crc => {
                // Sub-stream CRCs
                let total_sub: usize = us.num_unpack_streams.iter().map(|&n| n as usize).sum();
                let all_defined = read_u8(input)?;
                let defined: Vec<bool> = if all_defined != 0 {
                    vec![true; total_sub]
                } else {
                    read_bit_vector(input, total_sub)?
                };
                us.unpack_stream_crcs = vec![None; total_sub];
                for (i, &def) in defined.iter().enumerate() {
                    if def {
                        us.unpack_stream_crcs[i] = Some(read_u32_le(input)?);
                    }
                }
            }
            other => {
                return Err(SevenZippyError::invalid_header(format!(
                    "unexpected property {other:?} in SubStreamsInfo"
                )));
            }
        }
    }

    Ok(us)
}

// ── FilesInfo ─────────────────────────────────────────────────────────────────

fn parse_files_info(input: &mut &[u8]) -> SevenZippyResult<Vec<FileEntry>> {
    let num_files = read_uint64(input)? as usize;
    let mut entries: Vec<FileEntry> = (0..num_files).map(|_| FileEntry::default()).collect();

    loop {
        let prop_byte = read_u8(input)?;
        if prop_byte == 0x00 {
            break;
        }

        let prop = PropertyId::from_u8(prop_byte)?;
        // Each property block has a size field (number of bytes that follow).
        let prop_size = read_uint64(input)? as usize;
        // We read the property data into a fixed-size sub-slice so that parsing
        // bugs in one property can't corrupt later ones.
        let prop_data = read_bytes(input, prop_size)?;
        let mut cur: &[u8] = prop_data;

        match prop {
            PropertyId::Name => {
                // Format per spec:
                //   BYTE External
                //   if External != 0: UINT64 DataIndex (not supported)
                //   []
                //   for(Files): wchar_t Name[len]; wchar_t 0;
                //   []
                let external = read_u8(&mut cur)?;
                if external != 0 {
                    return Err(SevenZippyError::not_yet_implemented(
                        "external name data in FilesInfo",
                    ));
                }
                // Remaining bytes are UTF-16LE null-terminated names
                let name_data = cur; // all remaining bytes
                if !name_data.len().is_multiple_of(2) {
                    return Err(SevenZippyError::invalid_header(
                        "Name property byte count is not even (not valid UTF-16LE)",
                    ));
                }
                let u16s: Vec<u16> = name_data
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let all_names = String::from_utf16(&u16s).map_err(|_| {
                    SevenZippyError::invalid_header("invalid UTF-16LE in Name property")
                })?;
                let names: Vec<&str> = all_names.split('\x00').filter(|s| !s.is_empty()).collect();
                for (i, name) in names.iter().take(num_files).enumerate() {
                    entries[i].name = name.to_string();
                }
            }

            PropertyId::EmptyStream => {
                let bv = read_bit_vector(&mut cur, num_files)?;
                for (i, &flag) in bv.iter().enumerate() {
                    entries[i].is_empty = flag;
                }
            }

            PropertyId::EmptyFile => {
                // Only meaningful for files flagged IsEmpty; skip for Phase C.
                // (The bit vector is relative to empty-stream count, not total files.)
                let _ = cur; // skip
            }

            PropertyId::Anti => {
                let bv = read_bit_vector(&mut cur, num_files)?;
                for (i, &flag) in bv.iter().enumerate() {
                    entries[i].is_anti = flag;
                }
            }

            PropertyId::MTime => {
                // Format per spec:
                //   BYTE AllAreDefined
                //   if AllAreDefined == 0: for(NumFiles) BIT TimeDefined
                //   BYTE External
                //   if External != 0: UINT64 DataIndex
                //   []
                //   for(Defined) REAL_UINT64 Time
                //   []
                let all_defined = read_u8(&mut cur)?;
                let defined: Vec<bool> = if all_defined != 0 {
                    vec![true; num_files]
                } else {
                    read_bit_vector(&mut cur, num_files)?
                };
                let external = read_u8(&mut cur)?;
                if external != 0 {
                    return Err(SevenZippyError::not_yet_implemented(
                        "external MTime data in FilesInfo",
                    ));
                }
                for (i, &def) in defined.iter().enumerate() {
                    if def {
                        entries[i].mtime = Some(read_u64_le(&mut cur)?);
                    }
                }
            }

            PropertyId::CTime | PropertyId::ATime => {
                // Same layout as MTime; discard values (not exposed in Phase C).
                let all_defined = read_u8(&mut cur)?;
                let defined: Vec<bool> = if all_defined != 0 {
                    vec![true; num_files]
                } else {
                    read_bit_vector(&mut cur, num_files)?
                };
                let external = read_u8(&mut cur)?;
                if external != 0 {
                    return Err(SevenZippyError::not_yet_implemented(
                        "external CTime/ATime data in FilesInfo",
                    ));
                }
                for &def in &defined {
                    if def {
                        let _ts = read_u64_le(&mut cur)?; // discard
                    }
                }
            }

            PropertyId::Attributes => {
                // Format per spec:
                //   BYTE AllAreDefined
                //   if AllAreDefined == 0: for(NumFiles) BIT AttributesAreDefined
                //   BYTE External
                //   if External != 0: UINT64 DataIndex
                //   []
                //   for(Defined) UINT32 Attributes
                //   []
                let all_defined = read_u8(&mut cur)?;
                let defined: Vec<bool> = if all_defined != 0 {
                    vec![true; num_files]
                } else {
                    read_bit_vector(&mut cur, num_files)?
                };
                let external = read_u8(&mut cur)?;
                if external != 0 {
                    return Err(SevenZippyError::not_yet_implemented(
                        "external Attributes data in FilesInfo",
                    ));
                }
                for (i, &def) in defined.iter().enumerate() {
                    if def {
                        entries[i].attributes = Some(read_u32_le(&mut cur)?);
                    }
                }
            }

            PropertyId::Dummy | PropertyId::StartPos | PropertyId::Comment => {
                // Skip: Dummy is a padding property; StartPos is for sparse files
                // (not needed for Phase C); Comment is informational.
                let _ = cur;
            }

            other => {
                // Forward-compatible: skip unrecognised property using the size field.
                let _ = cur;
                let _ = other;
            }
        }
    }

    Ok(entries)
}

// ── ArchiveProperties ────────────────────────────────────────────────────────

fn skip_archive_properties(input: &mut &[u8]) -> SevenZippyResult<()> {
    loop {
        let prop = read_u8(input)?;
        if prop == PropertyId::End as u8 {
            break;
        }
        let size = read_uint64(input)? as usize;
        let _ = read_bytes(input, size)?;
    }
    Ok(())
}
