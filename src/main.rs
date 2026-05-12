use std::process::ExitCode;

fn main() -> ExitCode {
    let argv0 = std::env::args().next().unwrap_or_default();
    let basename = std::path::Path::new(&argv0)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "8z".to_string());

    match basename.as_str() {
        "un8z" | "8zcat" => eightz::cli::run_extract(),
        _ => eightz::cli::run(),
    }
}
