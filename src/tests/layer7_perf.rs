//! Layer 7: performance bound tests (informational).
//!
//! These tests assert that basic operations complete within generous wall-time
//! bounds. They are marked `#[ignore]` by default because:
//! - They are informational, not correctness checks.
//! - Wall-time assertions are flaky in CI under load.
//! - Proper benchmarking belongs in `benches/` with Criterion.
//!
//! Un-ignore locally (or in a dedicated perf job) to catch gross regressions
//! before they reach `cargo bench`.

use super::fixtures;
use crate::Archive;
use crate::ArchiveBuilder;
use std::time::Instant;

/// Copy round-trip on 1 MiB must complete in under 1 second on any
/// modern machine. This is a sanity bound, not a throughput target.
#[test]
#[ignore = "perf bounds are informational only; run locally or in a dedicated perf job"]
fn copy_round_trip_1mib_under_1s() {
    let input = fixtures::random(0x1111_2222, 1024 * 1024);

    let start = Instant::now();

    let mut b = ArchiveBuilder::new();
    b.add_copy_file("payload.bin", input.clone());
    let archive_bytes = b.build().unwrap();

    let archive = Archive::parse(&archive_bytes).unwrap();
    let extracted = archive.reader().extract(0).unwrap();

    let elapsed = start.elapsed();

    assert_eq!(
        extracted.len(),
        input.len(),
        "round-trip must preserve length"
    );
    assert!(
        elapsed.as_secs() < 1,
        "Copy round-trip on 1 MiB took {elapsed:?}; expected < 1 second"
    );
}

/// Parsing a 64 KiB Copy-coder archive header must complete in under 10 ms.
#[test]
#[ignore = "perf bounds are informational only; run locally or in a dedicated perf job"]
fn parse_header_64k_under_10ms() {
    // Build the archive first (outside the timed section).
    let input = fixtures::random(0x3333_4444, 65_536);
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("payload.bin", input);
    let archive_bytes = b.build().unwrap();

    let start = Instant::now();
    for _ in 0..1000 {
        let _ = Archive::parse(&archive_bytes).unwrap();
    }
    let elapsed = start.elapsed();
    let per_parse = elapsed / 1000;

    assert!(
        per_parse.as_millis() < 10,
        "Archive::parse averaged {per_parse:?}; expected < 10 ms per call"
    );
}
