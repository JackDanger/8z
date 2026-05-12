//! Phase C closing test: full read + write round-trip via the in-tree Copy coder.

use crate::{Archive, ArchiveBuilder};

fn seeded_random(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state & 0xFF) as u8
        })
        .collect()
}

#[test]
fn round_trip_copy_64k() {
    let input = seeded_random(0x00C0_FFEE_FACE, 65_536);
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("payload.bin", input.clone());
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 1);
    assert_eq!(archive.file_name(0), Some("payload.bin"));

    let extracted = archive.reader().extract(0).unwrap();
    assert_eq!(extracted, input);
}

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

#[test]
fn round_trip_copy_three_files() {
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("a.bin", b"first".to_vec());
    b.add_copy_file("b.bin", seeded_random(1, 1024));
    b.add_copy_file("c.bin", seeded_random(2, 8192));
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    assert_eq!(archive.file_count(), 3);
    assert_eq!(archive.reader().extract(0).unwrap(), b"first");
}
