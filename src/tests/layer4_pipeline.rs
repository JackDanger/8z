//! Layer 4: full folder pipeline round-trip tests.
//!
//! These tests write a 7z archive with `ArchiveBuilder`, read it back with
//! `Archive::parse`, and assert byte-for-byte equality of the extracted
//! content — exercising the complete encode → container → decode path with
//! no external tools.
//!
//! Copy-coder rows are **live** and must pass. Codec-specific rows are
//! `#[ignore]`d until the corresponding crate is wired up.

use super::fixtures;
use crate::{assert_slices_eq, Archive, ArchiveBuilder};

// ── Copy coder (live) ─────────────────────────────────────────────────────────

/// Round-trip 64 KiB of seeded random bytes through the Copy pipeline.
///
/// Migrated from `copy_round_trip::round_trip_copy_64k`.
#[test]
fn round_trip_copy_64k() {
    let input = fixtures::random(0x00C0_FFEE_FACE, 65_536);
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("payload.bin", input.clone());
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 1);
    assert_eq!(archive.file_name(0), Some("payload.bin"));

    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);
}

/// Round-trip an empty file — edge case for the Copy coder.
///
/// Migrated from `copy_round_trip::round_trip_copy_empty`.
#[test]
fn round_trip_copy_empty() {
    let input: Vec<u8> = vec![];
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("empty.bin", input.clone());
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    let extracted = archive.reader().extract(0).unwrap();
    assert_eq!(extracted, input);
}

/// Round-trip an archive containing three files.
///
/// Migrated from `copy_round_trip::round_trip_copy_three_files`.
#[test]
fn round_trip_copy_three_files() {
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("a.bin", b"first".to_vec());
    b.add_copy_file("b.bin", fixtures::random(1, 1024));
    b.add_copy_file("c.bin", fixtures::random(2, 8192));
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 3);
    assert_eq!(archive.reader().extract(0).unwrap(), b"first");
}

/// Round-trip 1 MiB of zeros (maximally compressible for future coders).
#[test]
fn round_trip_copy_1mib_zeros() {
    let input = fixtures::zeros(1024 * 1024);
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("zeros.bin", input.clone());
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);
}

/// Round-trip a sequential-byte pattern.
#[test]
fn round_trip_copy_sequential() {
    let input = fixtures::sequential(32_768);
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("sequential.bin", input.clone());
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);
}

// ── LZMA pipeline (live via lazippy wrapper) ─────────────────────────────────

/// Round-trip 64 KiB of seeded random bytes through the LZMA pipeline.
#[cfg(feature = "lzma")]
#[test]
fn round_trip_lzma_64k() {
    use crate::pipeline::lzma::LzmaCoder;
    let input = fixtures::random(0xDEAD_BEEF, 65_536);
    let mut b = ArchiveBuilder::new();
    b.add_file("payload.bin", input.clone(), Box::new(LzmaCoder::new()));
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 1);

    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);
}

/// Verify LZMA compresses 1 MiB of zeros to significantly less than 1 MiB.
#[cfg(feature = "lzma")]
#[test]
fn round_trip_lzma_zeros_compresses_well() {
    use crate::pipeline::lzma::LzmaCoder;
    let input = fixtures::zeros(1024 * 1024);
    let mut b = ArchiveBuilder::new();
    b.add_file("zeros.bin", input.clone(), Box::new(LzmaCoder::new()));
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);

    // The archive should be much smaller than the original 1 MiB input
    // (LZMA compresses zeros to well under 1% of input size).
    assert!(
        archive_bytes.len() < input.len() / 10,
        "LZMA should compress 1 MiB of zeros heavily; archive is {} bytes",
        archive_bytes.len()
    );
}

// ── Other codecs (ignored until respective crates are wired) ─────────────────

#[test]
#[ignore = "LZMA2 not yet implemented; un-ignore when lazippier is wired in dispatch.rs"]
fn round_trip_lzma2_64k() {
    todo!()
}

/// Round-trip 64 KiB of seeded random bytes through the BZip2 pipeline.
#[cfg(feature = "bzip2")]
#[test]
fn round_trip_bzip2_64k() {
    use crate::pipeline::bzip2::Bzip2Coder;
    let input = fixtures::random(0xB21_B21B, 65_536);
    let mut b = ArchiveBuilder::new();
    b.add_file("payload.bin", input.clone(), Box::new(Bzip2Coder));
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 1);

    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);
}

/// Round-trip 64 KiB of seeded random bytes through the Deflate pipeline.
#[cfg(feature = "deflate")]
#[test]
fn round_trip_deflate_64k() {
    use crate::pipeline::deflate::DeflateCoder;
    let input = fixtures::random(0x0DEF_1A7E, 65_536);
    let mut b = ArchiveBuilder::new();
    b.add_file("payload.bin", input.clone(), Box::new(DeflateCoder));
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 1);

    let extracted = archive.reader().extract(0).unwrap();
    assert_slices_eq!(extracted, input);
}

#[test]
#[ignore = "PPMd not yet implemented; un-ignore when pippyzippy is wired in dispatch.rs"]
fn round_trip_ppmd_64k() {
    todo!()
}
