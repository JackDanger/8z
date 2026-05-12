//! Layer 6: multi-threaded pipeline tests.
//!
//! All tests here are placeholders — threading is not yet implemented.
//! Un-ignore and fill in once the parallel decode/encode path lands.

/// Verify that decoding multiple archives concurrently on separate threads
/// produces correct output (no shared-state data races).
#[test]
#[ignore = "threading not yet implemented; un-ignore when parallel decode lands"]
fn concurrent_decode_produces_correct_output() {
    todo!()
}

/// Verify that encoding multiple archives concurrently on separate threads
/// produces archives that 7zz can extract.
#[test]
#[ignore = "threading not yet implemented; un-ignore when parallel encode lands"]
fn concurrent_encode_produces_valid_archives() {
    todo!()
}

/// Verify that the archive builder is `Send` and can be used from multiple
/// threads without synchronization issues.
#[test]
#[ignore = "threading not yet implemented; un-ignore when ArchiveBuilder is Send + Sync"]
fn archive_builder_is_send() {
    todo!()
}
