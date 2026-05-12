//! 7zz-compatible argument parsing for the 8z CLI.
//!
//! Phase C: only `--version` and `--help` are handled. Everything else
//! returns [`EightZError::NotYetImplemented`].

use std::process::ExitCode;

use crate::error::EightZError;

const VERSION_STRING: &str = concat!("8z ", env!("CARGO_PKG_VERSION"));

const HELP_STRING: &str = "\
Usage: 8z [COMMAND] [OPTIONS] [ARCHIVE] [FILES...]

Pure-Rust 7z archive implementation.

Commands (not yet implemented — coming in a later phase):
  a   Add files to archive
  x   Extract files with full paths
  e   Extract files to current directory
  l   List archive contents
  t   Test archive integrity
  d   Delete files from archive

Options:
  --version   Print version and exit
  --help      Print this help and exit

Examples:
  8z a archive.7z file.txt     Create archive (NYI)
  8z x archive.7z              Extract archive (NYI)
  un8z archive.7z              Alias for: 8z x archive.7z (NYI)
";

/// Primary entry point — invoked when argv[0] is `8z` or `8za`.
pub fn run() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    // argv[0] is the binary name; real args start at index 1.
    let flags: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();

    match flags.as_slice() {
        ["--version"] | ["-V"] => {
            println!("{VERSION_STRING}");
            ExitCode::SUCCESS
        }
        ["--help"] | ["-h"] => {
            print!("{HELP_STRING}");
            ExitCode::SUCCESS
        }
        _ => {
            let err = EightZError::not_yet_implemented("CLI subcommands");
            eprintln!("8z: {err}");
            eprintln!("Run `8z --help` for usage.");
            ExitCode::FAILURE
        }
    }
}

/// Extract entry point — invoked when argv[0] is `un8z` or `8zcat`.
pub fn run_extract() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let flags: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();

    match flags.as_slice() {
        ["--version"] | ["-V"] => {
            println!("{VERSION_STRING}");
            ExitCode::SUCCESS
        }
        ["--help"] | ["-h"] => {
            print!("{HELP_STRING}");
            ExitCode::SUCCESS
        }
        _ => {
            let err = EightZError::not_yet_implemented("extract subcommand");
            eprintln!("un8z: {err}");
            eprintln!("Run `8z --help` for usage.");
            ExitCode::FAILURE
        }
    }
}
