use std::process::ExitCode;

fn main() -> ExitCode {
    match kat::validate_grammar_registry_at_manifest_dir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
