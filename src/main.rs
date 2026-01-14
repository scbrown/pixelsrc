//! Pixelsrc - Command-line tool for rendering pixel art from JSONL definitions

use std::process::ExitCode;

use pixelsrc::cli;

fn main() -> ExitCode {
    cli::run()
}
