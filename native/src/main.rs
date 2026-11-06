#![allow(clippy::all, warnings, unused)]
fn main() -> std::process::ExitCode {
    match sawbyte_play::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sawbyte-play: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
