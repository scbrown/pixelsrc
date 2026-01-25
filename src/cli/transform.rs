//! Transform command implementation

use std::path::Path;
use std::process::ExitCode;

use super::EXIT_ERROR;

/// Transform sprites (mirror, rotate, tile, etc.)
///
/// Applies transforms to sprite grids and outputs new source files.
#[allow(clippy::too_many_arguments)]
#[allow(unused_variables)]
pub fn run_transform(
    input: &Path,
    mirror: Option<&str>,
    rotate: Option<u16>,
    tile: Option<&str>,
    pad: Option<u32>,
    outline: Option<Option<String>>,
    outline_width: u32,
    crop: Option<&str>,
    shift: Option<&str>,
    shadow: Option<&str>,
    shadow_token: Option<&str>,
    sprite_name: Option<&str>,
    output: &Path,
    stdin: bool,
    allow_large: bool,
) -> ExitCode {
    eprintln!("Error: The 'transform' command is deprecated.");
    eprintln!("Grid-based transforms are no longer supported.");
    eprintln!("Use structured regions format for sprite definitions.");
    ExitCode::from(EXIT_ERROR)
}
