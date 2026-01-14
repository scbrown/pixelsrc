//! TTP (Text To Pixel) - Command-line tool for rendering pixel art from JSONL definitions

use std::process::ExitCode;

use pxl::cli;

fn main() -> ExitCode {
    cli::run()
}
