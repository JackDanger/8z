//! 7zz-compatible argument parsing for the 7zippy CLI.
//!
//! Phase C: only `--version` and `--help` are handled. Everything else
//! returns [`SevenZippyError::NotYetImplemented`].

use std::process::ExitCode;

use crate::error::SevenZippyError;

const NAME: &str = "7zippy";
const VERSION_STRING: &str = concat!("7zippy ", env!("CARGO_PKG_VERSION"));

const HELP_STRING: &str = "\
Usage: 7zippy [COMMAND] [OPTIONS] [ARCHIVE] [FILES...]

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
  7zippy a archive.7z file.txt     Create archive (NYI)
  7zippy x archive.7z              Extract archive (NYI)
  un7zippy archive.7z              Alias for: 7zippy x archive.7z (NYI)
";

/// Primary entry point — invoked when argv\[0\] is `7zippy` or `7zippya`.
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
            let err = SevenZippyError::not_yet_implemented("CLI subcommands");
            eprintln!("{NAME}: {err}");
            eprintln!("Run `{NAME} --help` for usage.");
            ExitCode::FAILURE
        }
    }
}

/// Extract entry point — invoked when argv\[0\] is `un7zippy` or `7zippycat`.
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
            let err = SevenZippyError::not_yet_implemented("extract subcommand");
            eprintln!("un7zippy: {err}");
            eprintln!("Run `{NAME} --help` for usage.");
            ExitCode::FAILURE
        }
    }
}
