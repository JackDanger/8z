//! Layer 3: per-coder sub-crate smoke tests.
//!
//! Each test in this module delegates to a sibling crate's own test suite
//! by spawning `cargo test -p <crate> --lib` as a subprocess and asserting
//! exit 0. This provides a quick cross-package sanity check from 7zippy's
//! perspective: "do lazippy's own tests still pass?"
//!
//! Tests are skipped if `cargo` is not found on `$PATH`.
//!
//! # Future additions
//!
//! As more sibling crates (lazippier, pippyzippy, …) land, add a test row
//! here modelled on `lazippy_lib_tests_pass`.

use std::process::Command;

/// Find `cargo` on `$PATH`, returning `None` if not found.
fn find_cargo() -> Option<std::path::PathBuf> {
    // Use CARGO env var if set (common in test environments).
    if let Some(cargo) = std::env::var_os("CARGO") {
        return Some(std::path::PathBuf::from(cargo));
    }
    // Walk $PATH manually.
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join("cargo");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Verify that `lazippy`'s own lib tests pass from 7zippy's perspective.
///
/// This test is skipped (`return` early) if:
/// - `cargo` is not on PATH, or
/// - `lazippy` is not present at the expected `../lazippy` path relative to
///   the workspace root.
///
/// When lazippy is present, we run its tests with `--no-default-features`
/// so we don't require its optional dependencies.
#[test]
fn lazippy_lib_tests_pass() {
    let cargo = match find_cargo() {
        Some(c) => c,
        None => {
            eprintln!("[skip] cargo not found — per-coder smoke test skipped");
            return;
        }
    };

    // Resolve the lazippy path relative to the 7zippy workspace root.
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lazippy_path = workspace_root.join("../lazippy");
    if !lazippy_path.exists() {
        eprintln!(
            "[skip] lazippy not found at {} — per-coder smoke test skipped",
            lazippy_path.display()
        );
        return;
    }

    let output = Command::new(&cargo)
        .arg("test")
        .arg("--lib")
        .arg("--no-default-features")
        .arg("--manifest-path")
        .arg(lazippy_path.join("Cargo.toml"))
        .output()
        .expect("failed to spawn cargo test for lazippy");

    assert!(
        output.status.success(),
        "lazippy lib tests failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

// ── Ignored stubs: sibling crates not yet created ────────────────────────────

#[test]
#[ignore = "lazippier repo not yet created; un-ignore when JackDanger/lazippier lands"]
fn lazippier_lib_tests_pass() {
    // Identical pattern to lazippy_lib_tests_pass.
    todo!("add lazippier smoke test once the repo exists")
}

#[test]
#[ignore = "pippyzippy repo not yet created; un-ignore when JackDanger/pippyzippy lands"]
fn pippyzippy_lib_tests_pass() {
    todo!()
}

#[test]
#[ignore = "bzippy2 repo not yet created; un-ignore when JackDanger/bzippy2 lands"]
fn bzippy2_lib_tests_pass() {
    todo!()
}

#[test]
#[ignore = "gzippy library API not yet landed; un-ignore after the feat/library-api PR merges"]
fn gzippy_lib_tests_pass() {
    todo!()
}
