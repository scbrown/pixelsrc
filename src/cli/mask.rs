//! CLI dispatch for the `pxl mask` command.
//!
//! Handles read-only sprite state queries: token grid extraction and JSON output.

use std::path::Path;
use std::process::ExitCode;

use crate::mask::{MaskPipeline, TokenGrid};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the mask command.
pub fn run_mask(input: &Path, sprite: Option<&str>, json: bool) -> ExitCode {
    // Sprite name is required for grid extraction
    let sprite_name = match sprite {
        Some(name) => name,
        None => {
            // List available sprites
            match MaskPipeline::load(input, None) {
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

    // Load file and find sprite
    let pipeline: MaskPipeline = match MaskPipeline::load(input, Some(sprite_name)) {
        Ok(p) => p,
        Err(e) => {
            match &e {
                crate::draw::DrawError::SpriteNotFound(name) => {
                    eprintln!("Error: sprite '{}' not found in '{}'", name, input.display());
                    if let Ok(p) = MaskPipeline::load(input, None) {
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

    let sprite = pipeline.sprite().unwrap();

    // Extract token grid
    let grid = match TokenGrid::from_sprite(sprite) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Output
    if json {
        print_json(&grid, sprite_name);
    } else {
        print_text(&grid, sprite_name);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Print token grid as JSON.
fn print_json(grid: &TokenGrid, sprite_name: &str) {
    let grid_json: Vec<Vec<&str>> = grid
        .grid
        .iter()
        .map(|row: &Vec<String>| -> Vec<&str> { row.iter().map(|s| s.as_str()).collect() })
        .collect();

    let output = serde_json::json!({
        "sprite": sprite_name,
        "width": grid.width,
        "height": grid.height,
        "grid": grid_json,
    });

    println!("{}", serde_json::to_string(&output).unwrap());
}

/// Print token grid as human-readable text.
fn print_text(grid: &TokenGrid, sprite_name: &str) {
    println!("Token grid for \"{}\" ({}x{}):", sprite_name, grid.width, grid.height);

    for row in &grid.grid {
        let line = row.iter().map(|t| format!("{{{}}}", t)).collect::<Vec<_>>().join("");
        println!("  {}", line);
    }
}
