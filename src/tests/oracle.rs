//! Hermetic wrapper around the `7zz` (Igor Pavlov's 7-Zip) CLI.
//! Every coder's round-trip oracle test calls into this module.
//!
//! # Design
//!
//! Two public functions cover all oracle use-cases:
//! - [`seven_zip_compress`] — produce a reference `.7z` archive from raw bytes.
//! - [`seven_zip_decompress`] — extract the first entry from a `.7z` archive.
//!
//! The [`require_7zz!`] macro at the top of any oracle test short-circuits on
//! machines where `7zz` isn't installed (allowed locally; CI always installs it).

use std::path::PathBuf;
use std::process::{Command, Stdio};

// ── CoderSpec ─────────────────────────────────────────────────────────────────

/// Coder specification for [`seven_zip_compress`].
///
/// Determines the `-m0=...` argument passed to `7zz a`.
///
/// Variants for codecs not yet implemented are included so future tests can
/// add oracle round-trips without changing the enum.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CoderSpec {
    /// Store-only (no compression). `-m0=copy`
    Copy,
    /// LZMA at the given compression level with an optional dictionary size.
    /// `-m0=lzma:d=<dict_size>` or `-m0=lzma` depending on options.
    Lzma {
        level: u32,
        /// Dictionary size in bytes. `None` lets 7zz choose the default.
        dict_size: Option<u32>,
    },
    /// LZMA2 at the given compression level. `-m0=lzma2`
    Lzma2 { level: u32 },
    /// BZip2. `-m0=bzip2`
    Bzip2 { level: u32 },
    /// Deflate. `-m0=deflate`
    Deflate { level: u32 },
    /// PPMd. `-m0=ppmd`
    Ppmd { level: u32, order: Option<u32> },
}

impl CoderSpec {
    /// Return the `-m0=...` value to pass to `7zz a`.
    pub fn m_arg(&self) -> String {
        match self {
            CoderSpec::Copy => "copy".to_string(),
            CoderSpec::Lzma {
                level: _,
                dict_size: Some(d),
            } => format!("lzma:d={d}"),
            CoderSpec::Lzma { .. } => "lzma".to_string(),
            CoderSpec::Lzma2 { level: _ } => "lzma2".to_string(),
            CoderSpec::Bzip2 { level: _ } => "bzip2".to_string(),
            CoderSpec::Deflate { level: _ } => "deflate".to_string(),
            CoderSpec::Ppmd {
                level: _,
                order: Some(o),
            } => format!("ppmd:o={o}"),
            CoderSpec::Ppmd { .. } => "ppmd".to_string(),
        }
    }

    /// Return the `-mx=...` value (compression level) to pass to `7zz a`.
    fn mx_arg(&self) -> u32 {
        match self {
            CoderSpec::Copy => 0,
            CoderSpec::Lzma { level, .. } => *level,
            CoderSpec::Lzma2 { level } => *level,
            CoderSpec::Bzip2 { level } => *level,
            CoderSpec::Deflate { level } => *level,
            CoderSpec::Ppmd { level, .. } => *level,
        }
    }
}

// ── which ─────────────────────────────────────────────────────────────────────

/// Locate a binary by searching `$PATH`.
///
/// Returns the full path if found, or an `Err` if not. We avoid adding a
/// `which` crate dependency by walking `$PATH` ourselves.
fn which(bin: &str) -> std::io::Result<PathBuf> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("{bin} not found in PATH"),
    ))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return the path to `7zz` (or `7z` as a fallback) if installed; `None` otherwise.
///
/// Callers that want to skip a test when `7zz` is absent should use the
/// [`require_7zz!`] macro rather than calling this directly.
pub fn seven_zz_path() -> Option<PathBuf> {
    which("7zz").or_else(|_| which("7z")).ok()
}

/// Skip the calling test if `7zz` is not installed, printing a message.
///
/// CI must have `7zz`; this macro allows the suite to pass on bare developer
/// machines that haven't installed it.
///
/// This macro is only valid inside `src/tests/` submodules where
/// `super::oracle::seven_zz_path` is in scope.
///
/// # Example
///
/// ```rust,ignore
/// #[test]
/// fn my_oracle_test() {
///     require_7zz!();
///     // ... rest of the test
/// }
/// ```
macro_rules! require_7zz {
    () => {
        if super::oracle::seven_zz_path().is_none() {
            eprintln!("[skip] 7zz not installed — oracle test skipped");
            return;
        }
    };
}

// Make require_7zz! available to all submodules of src/tests/ without
// needing an explicit import.
pub(super) use require_7zz;

/// Compress `input` bytes with the reference `7zz` tool and return the raw
/// `.7z` archive bytes.
///
/// Internally: writes `input` to a tempdir as `payload.bin`, runs
/// `7zz a -t7z -m0=<spec.m_arg()> -mx=<level> archive.7z payload.bin`,
/// reads back `archive.7z`.
///
/// # Panics
///
/// Panics if `7zz` is not installed or if the compression command fails.
/// Use [`require_7zz!`] at the top of any test that calls this function.
pub fn seven_zip_compress(input: &[u8], spec: &CoderSpec) -> Vec<u8> {
    let sevenzip = seven_zz_path()
        .expect("7zz not found on PATH — use require_7zz!() at the top of this test");

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    let payload_path = tmp.path().join("payload.bin");
    let archive_path = tmp.path().join("archive.7z");

    std::fs::write(&payload_path, input).expect("failed to write payload.bin");

    let mx = format!("-mx={}", spec.mx_arg());
    let m0 = format!("-m0={}", spec.m_arg());

    let output = Command::new(&sevenzip)
        .arg("a")
        .arg("-t7z")
        .arg(&m0)
        .arg(&mx)
        .arg(&archive_path)
        .arg(&payload_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn 7zz");

    assert!(
        output.status.success(),
        "7zz compress failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    std::fs::read(&archive_path).expect("failed to read archive.7z after 7zz")
}

/// Decompress a `.7z` archive with the reference `7zz` tool.
///
/// Returns the raw bytes of the first extracted entry (`payload.bin`).
///
/// Internally: writes `archive` to a tempdir as `archive.7z`, runs
/// `7zz x -o<outdir> -y archive.7z`, reads back `outdir/payload.bin`.
///
/// # Panics
///
/// Panics if `7zz` is not installed or if the extraction command fails.
/// Use [`require_7zz!`] at the top of any test that calls this function.
pub fn seven_zip_decompress(archive: &[u8]) -> Vec<u8> {
    let sevenzip = seven_zz_path()
        .expect("7zz not found on PATH — use require_7zz!() at the top of this test");

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    let archive_path = tmp.path().join("archive.7z");
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).expect("failed to create extraction dir");

    std::fs::write(&archive_path, archive).expect("failed to write archive.7z");

    let output = Command::new(&sevenzip)
        .arg("x")
        .arg(format!("-o{}", out_dir.display()))
        .arg("-y")
        .arg(&archive_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn 7zz");

    assert!(
        output.status.success(),
        "7zz decompress failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // Collect all extracted files; return the first one found.
    let first = find_first_file(&out_dir).expect("7zz succeeded but no files were extracted");
    std::fs::read(&first)
        .unwrap_or_else(|e| panic!("failed to read extracted file {}: {e}", first.display()))
}

/// Walk `dir` and return the path to the first regular file found (DFS order).
fn find_first_file(dir: &std::path::Path) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_file() {
            return Some(path);
        } else if path.is_dir() {
            if let Some(f) = find_first_file(&path) {
                return Some(f);
            }
        }
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_zz_path_does_not_panic() {
        // Just verify the function runs without panicking.
        // The value may be None on machines without 7zz installed.
        let _ = seven_zz_path();
    }

    #[test]
    fn coder_spec_m_arg_copy() {
        assert_eq!(CoderSpec::Copy.m_arg(), "copy");
    }

    #[test]
    fn coder_spec_m_arg_lzma_with_dict() {
        let spec = CoderSpec::Lzma {
            level: 5,
            dict_size: Some(1 << 20),
        };
        assert_eq!(spec.m_arg(), "lzma:d=1048576");
    }

    #[test]
    fn coder_spec_m_arg_lzma_no_dict() {
        let spec = CoderSpec::Lzma {
            level: 5,
            dict_size: None,
        };
        assert_eq!(spec.m_arg(), "lzma");
    }

    #[test]
    fn coder_spec_m_arg_ppmd_with_order() {
        let spec = CoderSpec::Ppmd {
            level: 6,
            order: Some(8),
        };
        assert_eq!(spec.m_arg(), "ppmd:o=8");
    }
}
