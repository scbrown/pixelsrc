//! CLI dispatch for the `pxl scaffold` command.
//!
//! Handles argument parsing and dispatches to scaffold generators.

use clap::Subcommand;
use std::path::Path;
use std::process::ExitCode;

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Output format for scaffold commands.
#[derive(Clone, Debug, Default, clap::ValueEnum)]
pub enum ScaffoldFormat {
    /// Pixelsrc .pxl format (default)
    #[default]
    Pxl,
    /// JSON Lines format
    Jsonl,
}

#[derive(Subcommand)]
pub enum ScaffoldAction {
    /// Generate an empty sprite with palette and grid
    ///
    /// Examples:
    ///   pxl scaffold sprite --name hero --size 16x16
    ///   pxl scaffold sprite --name hero --size 16x16 --palette medieval
    ///   pxl scaffold sprite --name hero --size 16x16 --tokens "skin,hair,eye"
    Sprite {
        /// Sprite name
        #[arg(long)]
        name: String,

        /// Sprite size as WxH (e.g., "16x16", "32x64")
        #[arg(long, default_value = "16x16")]
        size: String,

        /// Built-in palette name to use
        #[arg(long)]
        palette: Option<String>,

        /// Comma-separated token names for auto-generated palette
        #[arg(long)]
        tokens: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output format
        #[arg(long, default_value = "pxl", value_enum)]
        format: ScaffoldFormat,
    },

    /// Generate a tiled composition with placeholder tile sprites
    ///
    /// Examples:
    ///   pxl scaffold composition --name level --size 128x128 --cell-size 32x32
    ///   pxl scaffold composition --name level --size 128x128 --cell-size 32x32 --palette nature
    Composition {
        /// Composition name
        #[arg(long)]
        name: String,

        /// Total size as WxH (e.g., "128x128")
        #[arg(long)]
        size: String,

        /// Tile cell size as WxH (e.g., "32x32")
        #[arg(long)]
        cell_size: String,

        /// Built-in palette name to use
        #[arg(long)]
        palette: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output format
        #[arg(long, default_value = "pxl", value_enum)]
        format: ScaffoldFormat,
    },

    /// Generate a palette from preset or color list
    ///
    /// Examples:
    ///   pxl scaffold palette --name warm --preset forest
    ///   pxl scaffold palette --name custom --colors "#FF0000,#00FF00,#0000FF"
    Palette {
        /// Palette name
        #[arg(long)]
        name: String,

        /// Built-in preset name (forest, medieval, synthwave, ocean)
        #[arg(long)]
        preset: Option<String>,

        /// Comma-separated hex colors (e.g., "#FF0000,#00FF00")
        #[arg(long)]
        colors: Option<String>,

        /// Prefix for auto-generated token names (default: "c")
        #[arg(long, default_value = "c")]
        token_prefix: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output format
        #[arg(long, default_value = "pxl", value_enum)]
        format: ScaffoldFormat,
    },
}

/// Parse a "WxH" size string into (width, height).
fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid size '{}', expected WxH (e.g., \"16x16\")",
            s
        ));
    }
    let w: u32 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("invalid width '{}'", parts[0].trim()))?;
    let h: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("invalid height '{}'", parts[1].trim()))?;
    if w == 0 || h == 0 {
        return Err(format!("size dimensions must be > 0, got {}x{}", w, h));
    }
    Ok((w, h))
}

/// Write scaffold output to file or stdout.
fn write_output(content: &str, output: Option<&Path>) -> ExitCode {
    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(path, content) {
                eprintln!("Error writing to '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
            eprintln!("Wrote: {}", path.display());
            ExitCode::from(EXIT_SUCCESS)
        }
        None => {
            print!("{}", content);
            ExitCode::from(EXIT_SUCCESS)
        }
    }
}

/// Execute the scaffold command.
pub fn run_scaffold(action: ScaffoldAction) -> ExitCode {
    match action {
        ScaffoldAction::Sprite {
            name,
            size,
            palette,
            tokens,
            output,
            format: _,
        } => {
            let (w, h) = match parse_size(&size) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return ExitCode::from(EXIT_INVALID_ARGS);
                }
            };

            let token_list: Vec<String> = tokens
                .as_deref()
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let content = crate::scaffold::generate_sprite(
                &name,
                w,
                h,
                palette.as_deref(),
                &token_list,
            );

            write_output(&content, output.as_deref())
        }

        ScaffoldAction::Composition {
            name,
            size,
            cell_size,
            palette,
            output,
            format: _,
        } => {
            let (sw, sh) = match parse_size(&size) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return ExitCode::from(EXIT_INVALID_ARGS);
                }
            };

            let (cw, ch) = match parse_size(&cell_size) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return ExitCode::from(EXIT_INVALID_ARGS);
                }
            };

            if sw % cw != 0 || sh % ch != 0 {
                eprintln!(
                    "Error: size {}x{} is not evenly divisible by cell_size {}x{}",
                    sw, sh, cw, ch
                );
                return ExitCode::from(EXIT_INVALID_ARGS);
            }

            let content = crate::scaffold::generate_composition(
                &name,
                sw,
                sh,
                cw,
                ch,
                palette.as_deref(),
            );

            match content {
                Ok(c) => write_output(&c, output.as_deref()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::from(EXIT_ERROR)
                }
            }
        }

        ScaffoldAction::Palette {
            name,
            preset,
            colors,
            token_prefix,
            output,
            format: _,
        } => {
            let content = crate::scaffold::generate_palette_scaffold(
                &name,
                preset.as_deref(),
                colors.as_deref(),
                &token_prefix,
            );

            match content {
                Ok(c) => write_output(&c, output.as_deref()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::from(EXIT_ERROR)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_valid() {
        assert_eq!(parse_size("16x16"), Ok((16, 16)));
        assert_eq!(parse_size("32x64"), Ok((32, 64)));
        assert_eq!(parse_size("1x1"), Ok((1, 1)));
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("16").is_err());
        assert!(parse_size("16x").is_err());
        assert!(parse_size("x16").is_err());
        assert!(parse_size("0x16").is_err());
        assert!(parse_size("16x0").is_err());
        assert!(parse_size("axb").is_err());
    }
}
