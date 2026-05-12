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

// ── LZMA pipeline (ignored until lazippy is wired) ───────────────────────────

#[test]
#[ignore = "LZMA not yet implemented in lazippy; un-ignore after LazippyCoder is wired in dispatch.rs"]
fn round_trip_lzma_64k() {
    // When un-ignored: use an ArchiveBuilder variant that selects LZMA coder.
    todo!()
}

#[test]
#[ignore = "LZMA not yet implemented; un-ignore with round_trip_lzma_64k"]
fn round_trip_lzma_zeros_compresses_well() {
    // Assert that LZMA output on zeros is significantly smaller than input.
    todo!()
}

// ── Other codecs (ignored until respective crates are wired) ─────────────────

#[test]
#[ignore = "LZMA2 not yet implemented; un-ignore when lazippier is wired in dispatch.rs"]
fn round_trip_lzma2_64k() {
    todo!()
}

#[test]
#[ignore = "BZip2 not yet implemented; un-ignore when bzippy2 is wired in dispatch.rs"]
fn round_trip_bzip2_64k() {
    todo!()
}

#[test]
#[ignore = "Deflate not yet implemented; un-ignore when gzippy lib API is wired in dispatch.rs"]
fn round_trip_deflate_64k() {
    todo!()
}

#[test]
#[ignore = "PPMd not yet implemented; un-ignore when pippyzippy is wired in dispatch.rs"]
fn round_trip_ppmd_64k() {
    todo!()
}
