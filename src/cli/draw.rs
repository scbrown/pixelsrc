//! CLI dispatch for the `pxl draw` command.
//!
//! Handles argument parsing dispatch and orchestrates the draw pipeline.

use std::path::Path;
use std::process::ExitCode;

use crate::draw::{DrawError, DrawOp, DrawPipeline};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Parse a `--set` argument: `x,y="{token}"` or `x,y={token}`.
///
/// The token may optionally be wrapped in braces: `5,10="{eye}"` or `5,10={eye}`.
fn parse_set_arg(arg: &str) -> Result<DrawOp, String> {
    let (coords, token_part) = arg
        .split_once('=')
        .ok_or_else(|| format!("invalid --set format '{}', expected x,y={{token}}", arg))?;

    let (x, y) = parse_coords(coords)?;
    let token = parse_token(token_part)?;

    Ok(DrawOp::Set { x, y, token })
}

/// Parse an `--erase` argument: `x,y`.
fn parse_erase_arg(arg: &str) -> Result<DrawOp, String> {
    let (x, y) = parse_coords(arg)?;
    Ok(DrawOp::Erase { x, y })
}

/// Parse a `--rect` argument: `x,y,w,h="{token}"` or `x,y,w,h={token}`.
fn parse_rect_arg(arg: &str) -> Result<DrawOp, String> {
    let (coords, token_part) = arg
        .split_once('=')
        .ok_or_else(|| format!("invalid --rect format '{}', expected x,y,w,h={{token}}", arg))?;

    let parts: Vec<&str> = coords.split(',').collect();
    if parts.len() != 4 {
        return Err(format!(
            "invalid --rect coordinates '{}', expected x,y,w,h (4 values)",
            coords
        ));
    }

    let x: usize = parts[0].trim().parse().map_err(|_| format!("invalid x '{}'", parts[0].trim()))?;
    let y: usize = parts[1].trim().parse().map_err(|_| format!("invalid y '{}'", parts[1].trim()))?;
    let w: usize = parts[2].trim().parse().map_err(|_| format!("invalid width '{}'", parts[2].trim()))?;
    let h: usize = parts[3].trim().parse().map_err(|_| format!("invalid height '{}'", parts[3].trim()))?;

    let token = parse_token(token_part)?;

    Ok(DrawOp::Rect { x, y, w, h, token })
}

/// Parse `x,y` coordinate string.
fn parse_coords(s: &str) -> Result<(usize, usize), String> {
    let (x_str, y_str) = s
        .split_once(',')
        .ok_or_else(|| format!("invalid coordinates '{}', expected x,y", s))?;

    let x: usize = x_str
        .trim()
        .parse()
        .map_err(|_| format!("invalid x coordinate '{}'", x_str.trim()))?;
    let y: usize = y_str
        .trim()
        .parse()
        .map_err(|_| format!("invalid y coordinate '{}'", y_str.trim()))?;

    Ok((x, y))
}

/// Parse a token from `"{token}"` or `{token}` format, returning just the name.
fn parse_token(s: &str) -> Result<String, String> {
    let s = s.trim();
    // Strip surrounding quotes if present
    let s = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s);
    // Strip braces if present
    let s = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')).unwrap_or(s);

    if s.is_empty() {
        return Err("empty token".to_string());
    }
    Ok(s.to_string())
}

/// Parse a `--line` argument: `x1,y1,x2,y2="{token}"` or `x1,y1,x2,y2={token}`.
fn parse_line_arg(arg: &str) -> Result<DrawOp, String> {
    let (coords, token_part) = arg
        .split_once('=')
        .ok_or_else(|| format!("invalid --line format '{}', expected x1,y1,x2,y2={{token}}", arg))?;

    let parts: Vec<&str> = coords.split(',').collect();
    if parts.len() != 4 {
        return Err(format!(
            "invalid --line coordinates '{}', expected x1,y1,x2,y2 (4 values)",
            coords
        ));
    }

    let x0: usize = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("invalid x1 coordinate '{}'", parts[0].trim()))?;
    let y0: usize = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("invalid y1 coordinate '{}'", parts[1].trim()))?;
    let x1: usize = parts[2]
        .trim()
        .parse()
        .map_err(|_| format!("invalid x2 coordinate '{}'", parts[2].trim()))?;
    let y1: usize = parts[3]
        .trim()
        .parse()
        .map_err(|_| format!("invalid y2 coordinate '{}'", parts[3].trim()))?;

    let token = parse_token(token_part)?;

    Ok(DrawOp::Line { x0, y0, x1, y1, token })
}

/// Collect all draw operations from CLI args.
fn collect_ops(set_args: &[String], erase_args: &[String], rect_args: &[String], line_args: &[String]) -> Result<Vec<DrawOp>, String> {
    let mut ops = Vec::new();
    for arg in set_args {
        ops.push(parse_set_arg(arg)?);
    }
    for arg in erase_args {
        ops.push(parse_erase_arg(arg)?);
    }
    for arg in rect_args {
        ops.push(parse_rect_arg(arg)?);
    }
    for arg in line_args {
        ops.push(parse_line_arg(arg)?);
    }
    Ok(ops)
}

/// Execute the draw command.
pub fn run_draw(
    input: &Path,
    sprite: Option<&str>,
    set_args: &[String],
    erase_args: &[String],
    rect_args: &[String],
    line_args: &[String],
    output: Option<&Path>,
    dry_run: bool,
) -> ExitCode {
    // Parse operations first (fail fast on bad args)
    let ops = match collect_ops(set_args, erase_args, rect_args, line_args) {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    // Sprite name is required
    let sprite_name = match sprite {
        Some(name) => name,
        None => {
            // If no sprite specified, list available sprites
            match DrawPipeline::load(input, None) {
                Ok(pipeline) => {
                    let names: Vec<&str> = pipeline.sprite_names();
                    if names.is_empty() {
                        eprintln!("Error: No sprites found in '{}'", input.display());
                    } else {
                        eprintln!("Error: --sprite is required. Available sprites:");
                        for name in names {
                            eprintln!("  {}", name);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    // Load the file and find the sprite
    let mut pipeline: DrawPipeline = match DrawPipeline::load(input, Some(sprite_name)) {
        Ok(p) => p,
        Err(e) => {
            match &e {
                DrawError::SpriteNotFound(name) => {
                    eprintln!("Error: sprite '{}' not found in '{}'", name, input.display());
                    // Try to list available sprites
                    if let Ok(p) = DrawPipeline::load(input, None) {
                        let names: Vec<&str> = p.sprite_names();
                        if !names.is_empty() {
                            eprintln!("Available sprites:");
                            for n in names {
                                eprintln!("  {}", n);
                            }
                        }
                    }
                }
                _ => eprintln!("Error: {}", e),
            }
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Apply draw operations (if any)
    if !ops.is_empty() {
        if let Err(e) = pipeline.apply_ops(&ops) {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    }

    if dry_run {
        // Dry run: serialize and print diff
        match pipeline.serialize() {
            Ok(result) => {
                for warning in &result.warnings {
                    eprintln!("Warning: {}", warning);
                }

                let original = match std::fs::read_to_string(input) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading '{}': {}", input.display(), e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                };

                if original == result.content {
                    println!("No changes.");
                } else {
                    // Show a simple diff
                    let target_display = output
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| input.display().to_string());
                    println!("--- {}", input.display());
                    println!("+++ {} (after draw)", target_display);
                    print_simple_diff(&original, &result.content);
                }
                ExitCode::from(EXIT_SUCCESS)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::from(EXIT_ERROR)
            }
        }
    } else {
        // Write mode
        let target = output.unwrap_or(input);
        match pipeline.write_to(target) {
            Ok(result) => {
                for warning in &result.warnings {
                    eprintln!("Warning: {}", warning);
                }
                eprintln!("Wrote: {}", target.display());
                ExitCode::from(EXIT_SUCCESS)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::from(EXIT_ERROR)
            }
        }
    }
}

/// Print a simple line-by-line diff between two strings.
fn print_simple_diff(original: &str, modified: &str) {
    let orig_lines: Vec<&str> = original.lines().collect();
    let mod_lines: Vec<&str> = modified.lines().collect();

    let max_len = orig_lines.len().max(mod_lines.len());
    for i in 0..max_len {
        let orig = orig_lines.get(i).copied().unwrap_or("");
        let modi = mod_lines.get(i).copied().unwrap_or("");
        if orig != modi {
            if !orig.is_empty() {
                println!("-{}", orig);
            }
            if !modi.is_empty() {
                println!("+{}", modi);
            }
        }
    }
}
