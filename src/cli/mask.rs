//! CLI dispatch for the `pxl mask` command.
//!
//! Handles read-only sprite state queries: token grid extraction,
//! coordinate queries, bounding boxes, and JSON output.

use std::path::Path;
use std::process::ExitCode;

use crate::mask::{BoundsResult, MaskPipeline, QueryResult, TokenGrid};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Normalize a token argument: strip surrounding braces if present.
/// Users type `--query "{eye}"` but the grid stores bare names like `"eye"`.
fn normalize_token(token: &str) -> &str {
    token.strip_prefix('{').and_then(|s| s.strip_suffix('}')).unwrap_or(token)
}

/// Execute the mask command.
pub fn run_mask(
    input: &Path,
    sprite: Option<&str>,
    json: bool,
    query: Option<&str>,
    bounds: Option<&str>,
) -> ExitCode {
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

    // Dispatch to the appropriate operation
    if let Some(token) = query {
        let token = normalize_token(token);
        let result = grid.query(token);
        if json {
            print_query_json(&result);
        } else {
            print_query_text(&result);
        }
    } else if let Some(token) = bounds {
        let token = normalize_token(token);
        let result = grid.bounds(token);
        if json {
            print_bounds_json(&result);
        } else {
            print_bounds_text(&result);
        }
    } else {
        // Default: dump the full token grid
        if json {
            print_grid_json(&grid, sprite_name);
        } else {
            print_grid_text(&grid, sprite_name);
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

// --- Query output ---

fn print_query_text(result: &QueryResult) {
    let count = result.coords.len();
    println!("{{{}}}: {} pixel{}", result.token, count, if count == 1 { "" } else { "s" });
    for (x, y) in &result.coords {
        println!("  ({}, {})", x, y);
    }
}

fn print_query_json(result: &QueryResult) {
    let coords: Vec<[u32; 2]> = result.coords.iter().map(|&(x, y)| [x, y]).collect();
    let output = serde_json::json!({
        "token": format!("{{{}}}", result.token),
        "count": result.coords.len(),
        "coords": coords,
    });
    println!("{}", serde_json::to_string(&output).unwrap());
}

// --- Bounds output ---

fn print_bounds_text(result: &BoundsResult) {
    match result.bounds {
        Some([x, y, w, h]) => {
            println!(
                "{{{}}}: bounding box [{}, {}, {}, {}]  (x={}, y={}, w={}, h={})",
                result.token, x, y, w, h, x, y, w, h
            );
            println!("  {} pixels", result.pixel_count);
        }
        None => {
            println!("{{{}}}: not found", result.token);
        }
    }
}

fn print_bounds_json(result: &BoundsResult) {
    let output = match result.bounds {
        Some(b) => serde_json::json!({
            "token": format!("{{{}}}", result.token),
            "bounds": b,
            "pixel_count": result.pixel_count,
        }),
        None => serde_json::json!({
            "token": format!("{{{}}}", result.token),
            "bounds": null,
            "pixel_count": 0,
        }),
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}

// --- Grid output (default, unchanged from M1) ---

fn print_grid_json(grid: &TokenGrid, sprite_name: &str) {
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

fn print_grid_text(grid: &TokenGrid, sprite_name: &str) {
    println!("Token grid for \"{}\" ({}x{}):", sprite_name, grid.width, grid.height);

    for row in &grid.grid {
        let line = row.iter().map(|t| format!("{{{}}}", t)).collect::<Vec<_>>().join("");
        println!("  {}", line);
    }
}
