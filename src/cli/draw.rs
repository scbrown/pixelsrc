//! CLI dispatch for the `pxl draw` command.
//!
//! Handles argument parsing dispatch and orchestrates the draw pipeline.

use std::path::Path;
use std::process::ExitCode;

use crate::draw::{DrawError, DrawPipeline};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the draw command.
pub fn run_draw(
    input: &Path,
    sprite: Option<&str>,
    output: Option<&Path>,
    dry_run: bool,
) -> ExitCode {
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
    let pipeline: DrawPipeline = match DrawPipeline::load(input, Some(sprite_name)) {
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
