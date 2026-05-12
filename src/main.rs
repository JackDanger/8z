use std::process::ExitCode;

fn main() -> ExitCode {
    let argv0 = std::env::args().next().unwrap_or_default();
    let basename = std::path::Path::new(&argv0)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "7zippy".to_string());

    match basename.as_str() {
        "un7zippy" | "7zippycat" => sevenzippy::cli::run_extract(),
        _ => sevenzippy::cli::run(),
    }
}
