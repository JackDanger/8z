//! Write-side oracle: our 8z-written archive must be readable by the real 7zz.
//! These tests are skipped if 7zz isn't on PATH (CI installs it).

use crate::ArchiveBuilder;
use std::process::{Command, Stdio};

fn has_7zz() -> bool {
    Command::new("7zz")
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn seven_zip_can_extract_our_copy_archive() {
    if !has_7zz() {
        eprintln!("skipping: 7zz not installed");
        return;
    }

    let payload = b"Hello, from 8z's own writer!".to_vec();
    let mut b = ArchiveBuilder::new();
    b.add_copy_file("greeting.txt", payload.clone());
    let archive_bytes = b.build().unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let archive_path = tmp.path().join("ours.7z");
    std::fs::write(&archive_path, &archive_bytes).unwrap();

    // Extract via 7zz into a subdirectory
    let extract_dir = tmp.path().join("out");
    std::fs::create_dir_all(&extract_dir).unwrap();
    let output = Command::new("7zz")
        .arg("x")
        .arg(format!("-o{}", extract_dir.display()))
        .arg("-y")
        .arg(&archive_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "7zz failed to extract our archive:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let extracted = std::fs::read(extract_dir.join("greeting.txt")).unwrap();
    assert_eq!(extracted, payload);
}
