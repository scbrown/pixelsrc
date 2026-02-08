//! CLI dispatch for the `pxl mask` command.
//!
//! Handles read-only sprite state queries: token grid extraction,
//! coordinate queries, bounding boxes, and JSON output.

use std::path::Path;
use std::process::ExitCode;

use crate::mask::{BoundsResult, MaskPipeline, QueryResult, TokenGrid};
use crate::models::{PaletteRef, TtpObject};

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
    sample: Option<&str>,
    neighbors: Option<&str>,
    json: bool,
    query: Option<&str>,
    bounds: Option<&str>,
    list: bool,
) -> ExitCode {
    // --list is a file-level query that doesn't require --sprite
    if list {
        return run_list(input, json);
    }

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
    } else if let Some(coord_str) = sample {
        return run_sample(&grid, coord_str, json);
    } else if let Some(coord_str) = neighbors {
        return run_neighbors(&grid, coord_str, json);
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

// --- Coordinate parsing ---

/// Parse "x,y" coordinate string.
fn parse_coords(s: &str) -> Result<(u32, u32), String> {
    let (x_str, y_str) =
        s.split_once(',').ok_or_else(|| format!("invalid coordinates '{}', expected x,y", s))?;

    let x: u32 =
        x_str.trim().parse().map_err(|_| format!("invalid x coordinate '{}'", x_str.trim()))?;
    let y: u32 =
        y_str.trim().parse().map_err(|_| format!("invalid y coordinate '{}'", y_str.trim()))?;

    Ok((x, y))
}

// --- Sample output ---

/// Execute --sample operation.
fn run_sample(grid: &TokenGrid, coord_str: &str, json: bool) -> ExitCode {
    let (x, y) = match parse_coords(coord_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    match grid.sample(x, y) {
        Ok(token) => {
            if json {
                let output = serde_json::json!({
                    "x": x,
                    "y": y,
                    "token": format!("{{{}}}", token),
                });
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                println!("({}, {}): {{{}}}", x, y, token);
            }
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(EXIT_ERROR)
        }
    }
}

// --- Neighbors output ---

/// Execute --neighbors operation.
fn run_neighbors(grid: &TokenGrid, coord_str: &str, json: bool) -> ExitCode {
    let (x, y) = match parse_coords(coord_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    match grid.neighbors(x, y) {
        Ok(result) => {
            if json {
                let mut neighbors = serde_json::Map::new();
                if let Some(ref t) = result.up {
                    neighbors
                        .insert("up".to_string(), serde_json::Value::String(format!("{{{}}}", t)));
                }
                if let Some(ref t) = result.down {
                    neighbors.insert(
                        "down".to_string(),
                        serde_json::Value::String(format!("{{{}}}", t)),
                    );
                }
                if let Some(ref t) = result.left {
                    neighbors.insert(
                        "left".to_string(),
                        serde_json::Value::String(format!("{{{}}}", t)),
                    );
                }
                if let Some(ref t) = result.right {
                    neighbors.insert(
                        "right".to_string(),
                        serde_json::Value::String(format!("{{{}}}", t)),
                    );
                }

                let output = serde_json::json!({
                    "x": x,
                    "y": y,
                    "token": format!("{{{}}}", result.token),
                    "neighbors": serde_json::Value::Object(neighbors),
                });
                println!("{}", serde_json::to_string(&output).unwrap());
            } else {
                println!("({}, {}): {{{}}}", x, y, result.token);
                if let Some(ref t) = result.up {
                    println!("  up    ({}, {}): {{{}}}", x, y.wrapping_sub(1), t);
                }
                if let Some(ref t) = result.down {
                    println!("  down  ({}, {}): {{{}}}", x, y + 1, t);
                }
                if let Some(ref t) = result.left {
                    println!("  left  ({}, {}): {{{}}}", x.wrapping_sub(1), y, t);
                }
                if let Some(ref t) = result.right {
                    println!("  right ({}, {}): {{{}}}", x + 1, y, t);
                }
            }
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(EXIT_ERROR)
        }
    }
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

// --- List output (DT-M5) ---

/// Execute --list operation: enumerate all sprites, compositions, animations.
fn run_list(input: &Path, json: bool) -> ExitCode {
    let pipeline = match MaskPipeline::load(input, None) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    let objects = pipeline.objects();

    let mut sprites: Vec<SpriteInfo> = Vec::new();
    let mut compositions: Vec<CompositionInfo> = Vec::new();
    let mut animations: Vec<AnimationInfo> = Vec::new();

    for obj in objects {
        match obj {
            TtpObject::Sprite(s) => {
                let size = s.size;
                let palette_name = match &s.palette {
                    PaletteRef::Named(n) if !n.is_empty() => Some(n.clone()),
                    PaletteRef::Inline(_) => Some("(inline)".to_string()),
                    _ => None,
                };
                let format = if s.regions.is_some() {
                    "regions"
                } else if s.source.is_some() {
                    "source"
                } else {
                    "grid"
                };
                let region_count = s.regions.as_ref().map(|r| r.len());
                sprites.push(SpriteInfo {
                    name: s.name.clone(),
                    size,
                    palette: palette_name,
                    format: format.to_string(),
                    region_count,
                });
            }
            TtpObject::Composition(c) => {
                compositions.push(CompositionInfo {
                    name: c.name.clone(),
                    size: c.size,
                    layer_count: c.layers.len(),
                });
            }
            TtpObject::Animation(a) => {
                let frame_count = if a.is_frame_based() {
                    a.frames.len()
                } else if a.is_css_keyframes() {
                    a.css_keyframes().map(|kf| kf.len()).unwrap_or(0)
                } else {
                    0
                };
                let duration_ms = a.duration_ms();
                let total_ms =
                    if a.is_frame_based() { duration_ms * frame_count as u32 } else { duration_ms };
                animations.push(AnimationInfo {
                    name: a.name.clone(),
                    frame_count,
                    duration_ms: total_ms,
                    format: if a.is_css_keyframes() {
                        "keyframes".to_string()
                    } else {
                        "frames".to_string()
                    },
                });
            }
            _ => {}
        }
    }

    if json {
        print_list_json(&sprites, &compositions, &animations);
    } else {
        print_list_text(input, &sprites, &compositions, &animations);
    }

    ExitCode::from(EXIT_SUCCESS)
}

struct SpriteInfo {
    name: String,
    size: Option<[u32; 2]>,
    palette: Option<String>,
    format: String,
    region_count: Option<usize>,
}

struct CompositionInfo {
    name: String,
    size: Option<[u32; 2]>,
    layer_count: usize,
}

struct AnimationInfo {
    name: String,
    frame_count: usize,
    duration_ms: u32,
    format: String,
}

fn print_list_text(
    input: &Path,
    sprites: &[SpriteInfo],
    compositions: &[CompositionInfo],
    animations: &[AnimationInfo],
) {
    let filename = input.file_name().map(|f| f.to_string_lossy()).unwrap_or_default();

    if !sprites.is_empty() {
        println!("Sprites in {}:", filename);
        for s in sprites {
            let size_str = match s.size {
                Some([w, h]) => format!("{}x{}", w, h),
                None => "?x?".to_string(),
            };
            let palette_str = match &s.palette {
                Some(p) => format!("  palette: {}", p),
                None => String::new(),
            };
            let region_str = match s.region_count {
                Some(n) => format!("  {} regions", n),
                None => String::new(),
            };
            println!(
                "  {:<16}{:<8}{}  ({}){}",
                s.name, size_str, palette_str, s.format, region_str
            );
        }
    }

    if !compositions.is_empty() {
        if !sprites.is_empty() {
            println!();
        }
        println!("Compositions:");
        for c in compositions {
            let size_str = match c.size {
                Some([w, h]) => format!("{}x{}", w, h),
                None => "?x?".to_string(),
            };
            println!("  {:<16}{}  {} layers", c.name, size_str, c.layer_count);
        }
    }

    if !animations.is_empty() {
        if !sprites.is_empty() || !compositions.is_empty() {
            println!();
        }
        println!("Animations:");
        for a in animations {
            println!("  {:<16}{} {}, {}ms", a.name, a.frame_count, a.format, a.duration_ms);
        }
    }

    if sprites.is_empty() && compositions.is_empty() && animations.is_empty() {
        println!("No sprites, compositions, or animations found in {}", filename);
    }
}

fn print_list_json(
    sprites: &[SpriteInfo],
    compositions: &[CompositionInfo],
    animations: &[AnimationInfo],
) {
    let sprites_json: Vec<serde_json::Value> = sprites
        .iter()
        .map(|s| {
            let mut obj = serde_json::json!({
                "name": s.name,
                "format": s.format,
            });
            if let Some([w, h]) = s.size {
                obj["size"] = serde_json::json!([w, h]);
            }
            if let Some(ref p) = s.palette {
                obj["palette"] = serde_json::json!(p);
            }
            if let Some(n) = s.region_count {
                obj["region_count"] = serde_json::json!(n);
            }
            obj
        })
        .collect();

    let compositions_json: Vec<serde_json::Value> = compositions
        .iter()
        .map(|c| {
            let mut obj = serde_json::json!({
                "name": c.name,
                "layer_count": c.layer_count,
            });
            if let Some([w, h]) = c.size {
                obj["size"] = serde_json::json!([w, h]);
            }
            obj
        })
        .collect();

    let animations_json: Vec<serde_json::Value> = animations
        .iter()
        .map(|a| {
            serde_json::json!({
                "name": a.name,
                "frame_count": a.frame_count,
                "duration_ms": a.duration_ms,
                "format": a.format,
            })
        })
        .collect();

    let output = serde_json::json!({
        "sprites": sprites_json,
        "compositions": compositions_json,
        "animations": animations_json,
    });
    println!("{}", serde_json::to_string(&output).unwrap());
}
