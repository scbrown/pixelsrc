//! CLI integration tests for the `pxl mask` command.
//!
//! Tests the full command-line interface using real v2 example files
//! (region-based format). Verifies text and JSON output for all mask
//! operations: query, bounds, sample, neighbors, region, count, list.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the pxl binary.
fn pxl_binary() -> PathBuf {
    let release = Path::new("target/release/pxl");
    if release.exists() {
        return release.to_path_buf();
    }
    let debug = Path::new("target/debug/pxl");
    if debug.exists() {
        return debug.to_path_buf();
    }
    panic!("pxl binary not found. Run 'cargo build' first.");
}

/// Run pxl mask with the given arguments and return (stdout, stderr, success).
fn run_mask(args: &[&str]) -> (String, String, bool) {
    let output =
        Command::new(pxl_binary()).arg("mask").args(args).output().expect("Failed to execute pxl");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

// ============================================================================
// --list operation (file-level, no --sprite required)
// ============================================================================

#[test]
fn test_mask_list_hero() {
    let (stdout, _, ok) = run_mask(&["examples/hero.pxl", "--list"]);
    assert!(ok, "mask --list should succeed");
    assert!(stdout.contains("hero_idle"), "should list hero_idle sprite");
    assert!(stdout.contains("hero_breathe"), "should list hero_breathe animation");
}

#[test]
fn test_mask_list_hero_json() {
    let (stdout, _, ok) = run_mask(&["examples/hero.pxl", "--list", "--json"]);
    assert!(ok, "mask --list --json should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let sprites = json["sprites"].as_array().expect("sprites array");
    assert!(sprites.iter().any(|s| s["name"] == "hero_idle"), "should contain hero_idle");
    assert_eq!(sprites[0]["format"], "regions", "hero uses region format");
    let animations = json["animations"].as_array().expect("animations array");
    assert!(!animations.is_empty(), "should have animations");
}

#[test]
fn test_mask_list_coin() {
    let (stdout, _, ok) = run_mask(&["examples/coin.pxl", "--list", "--json"]);
    assert!(ok, "mask --list coin should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let sprites = json["sprites"].as_array().unwrap();
    assert!(sprites.iter().any(|s| s["name"] == "coin"));
    // Coin has region_count
    let coin = sprites.iter().find(|s| s["name"] == "coin").unwrap();
    assert!(coin["region_count"].as_u64().unwrap() > 0, "coin should have regions");
}

#[test]
fn test_mask_list_heart() {
    let (stdout, _, ok) = run_mask(&["examples/heart.pxl", "--list"]);
    assert!(ok);
    assert!(stdout.contains("heart"), "should list heart sprite");
    assert!(stdout.contains("heart_beat"), "should list heart_beat animation");
}

// ============================================================================
// --query operation
// ============================================================================

#[test]
fn test_mask_query_hero_eye() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--query", "{eye}"]);
    assert!(ok, "query should succeed");
    assert!(stdout.contains("{eye}"), "output should mention {{eye}}");
    assert!(stdout.contains("2 pixel"), "hero has 2 eye pixels");
}

#[test]
fn test_mask_query_hero_eye_json() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--query", "{eye}", "--json"]);
    assert!(ok, "query --json should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["token"], "{eye}");
    assert_eq!(json["count"], 2, "hero has 2 eye pixels");
    let coords = json["coords"].as_array().unwrap();
    assert_eq!(coords.len(), 2);
    // Eyes are at (6,6) and (9,6)
    assert_eq!(coords[0], serde_json::json!([6, 6]));
    assert_eq!(coords[1], serde_json::json!([9, 6]));
}

#[test]
fn test_mask_query_bare_token_name() {
    // bare name (no braces) should also work
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--query", "eye", "--json"]);
    assert!(ok, "bare token name should work");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["count"], 2);
}

#[test]
fn test_mask_query_not_found() {
    let (stdout, _, ok) = run_mask(&[
        "examples/hero.pxl",
        "--sprite",
        "hero_idle",
        "--query",
        "nonexistent",
        "--json",
    ]);
    assert!(ok, "query for missing token should still succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["count"], 0);
    assert_eq!(json["coords"].as_array().unwrap().len(), 0);
}

// ============================================================================
// --bounds operation
// ============================================================================

#[test]
fn test_mask_bounds_hero_shirt() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--bounds", "{shirt}", "--json"]);
    assert!(ok, "bounds should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["token"], "{shirt}");
    assert!(json["bounds"].is_array(), "should have bounds array");
    assert!(json["pixel_count"].as_u64().unwrap() > 0, "shirt should have pixels");
}

#[test]
fn test_mask_bounds_hero_eye() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--bounds", "{eye}", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let bounds = json["bounds"].as_array().unwrap();
    // Eyes are at x=6,y=6 and x=9,y=6 → bbox [6, 6, 4, 1]
    assert_eq!(bounds[0], 6, "min x");
    assert_eq!(bounds[1], 6, "min y");
    assert_eq!(bounds[2], 4, "width (9-6+1)");
    assert_eq!(bounds[3], 1, "height");
    assert_eq!(json["pixel_count"], 2);
}

#[test]
fn test_mask_bounds_not_found() {
    let (stdout, _, ok) = run_mask(&[
        "examples/hero.pxl",
        "--sprite",
        "hero_idle",
        "--bounds",
        "nonexistent",
        "--json",
    ]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(json["bounds"].is_null(), "missing token has null bounds");
    assert_eq!(json["pixel_count"], 0);
}

#[test]
fn test_mask_bounds_coin_gold() {
    let (stdout, _, ok) =
        run_mask(&["examples/coin.pxl", "--sprite", "coin", "--bounds", "{gold}", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(json["pixel_count"].as_u64().unwrap() > 10, "gold covers most of the coin");
}

// ============================================================================
// --sample operation
// ============================================================================

#[test]
fn test_mask_sample_hero() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--sample", "6,6", "--json"]);
    assert!(ok, "sample should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["x"], 6);
    assert_eq!(json["y"], 6);
    // (6,6) is where the left eye is
    assert_eq!(json["token"], "{eye}");
}

#[test]
fn test_mask_sample_transparent() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--sample", "0,0", "--json"]);
    assert!(ok, "sample at empty corner should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    // Top-left corner of hero should be transparent or outline
    assert!(json["token"].is_string());
}

#[test]
fn test_mask_sample_out_of_bounds() {
    let (_, stderr, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--sample", "100,100"]);
    assert!(!ok, "sample out of bounds should fail");
    assert!(stderr.contains("out of bounds"), "should report out of bounds error");
}

#[test]
fn test_mask_sample_text_format() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--sample", "6,6"]);
    assert!(ok);
    // Text output format: "(6, 6): {eye}"
    assert!(stdout.contains("(6, 6)"), "text format shows coordinates");
    assert!(stdout.contains("{eye}"), "text format shows token in braces");
}

// ============================================================================
// --neighbors operation
// ============================================================================

#[test]
fn test_mask_neighbors_hero_eye() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--neighbors", "6,6", "--json"]);
    assert!(ok, "neighbors should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["token"], "{eye}");
    let neighbors = &json["neighbors"];
    // Eye at (6,6) is surrounded by skin/hair
    assert!(neighbors["up"].is_string(), "should have up neighbor");
    assert!(neighbors["down"].is_string(), "should have down neighbor");
    assert!(neighbors["left"].is_string(), "should have left neighbor");
    assert!(neighbors["right"].is_string(), "should have right neighbor");
}

#[test]
fn test_mask_neighbors_text_format() {
    let (stdout, _, ok) =
        run_mask(&["examples/coin.pxl", "--sprite", "coin", "--neighbors", "3,3"]);
    assert!(ok);
    // Text output should have main coordinate and directional neighbors
    assert!(stdout.contains("(3, 3)"), "should show center coordinate");
    assert!(stdout.contains("up"), "should show up neighbor");
    assert!(stdout.contains("down"), "should show down neighbor");
}

#[test]
fn test_mask_neighbors_out_of_bounds() {
    let (_, stderr, ok) =
        run_mask(&["examples/coin.pxl", "--sprite", "coin", "--neighbors", "99,99"]);
    assert!(!ok, "neighbors out of bounds should fail");
    assert!(stderr.contains("out of bounds"));
}

// ============================================================================
// --region operation
// ============================================================================

#[test]
fn test_mask_region_hero_face() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--region", "5,5,6,4", "--json"]);
    assert!(ok, "region should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["x"], 5);
    assert_eq!(json["y"], 5);
    assert_eq!(json["width"], 6);
    assert_eq!(json["height"], 4);
    assert_eq!(json["clamped"], false);
    let grid = json["grid"].as_array().unwrap();
    assert_eq!(grid.len(), 4, "4 rows");
    assert_eq!(grid[0].as_array().unwrap().len(), 6, "6 columns");
}

#[test]
fn test_mask_region_clamped() {
    let (stdout, stderr, ok) = run_mask(&[
        "examples/hero.pxl",
        "--sprite",
        "hero_idle",
        "--region",
        "10,10,20,20",
        "--json",
    ]);
    assert!(ok, "clamped region should succeed");
    assert!(stderr.contains("clamped"), "should warn about clamping");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["clamped"], true);
}

#[test]
fn test_mask_region_text_format() {
    let (stdout, _, ok) =
        run_mask(&["examples/heart.pxl", "--sprite", "heart", "--region", "0,0,7,6"]);
    assert!(ok);
    // Text output should contain brace-wrapped tokens
    assert!(stdout.contains("Region"), "text format starts with 'Region'");
    assert!(stdout.contains("{"), "text format wraps tokens in braces");
}

// ============================================================================
// --count operation
// ============================================================================

#[test]
fn test_mask_count_hero() {
    let (stdout, _, ok) =
        run_mask(&["examples/hero.pxl", "--sprite", "hero_idle", "--count", "--json"]);
    assert!(ok, "count should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let total = json["total"].as_u64().unwrap();
    assert_eq!(total, 256, "hero is 16x16 = 256 pixels");
    let tokens = json["tokens"].as_object().unwrap();
    assert!(tokens.contains_key("{_}"), "should have transparent token");
    assert!(tokens.contains_key("{outline}"), "should have outline token");
}

#[test]
fn test_mask_count_coin() {
    let (stdout, _, ok) = run_mask(&["examples/coin.pxl", "--sprite", "coin", "--count", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["total"], 64, "coin is 8x8 = 64 pixels");
    let tokens = json["tokens"].as_object().unwrap();
    assert!(tokens.contains_key("{gold}"), "should have gold token");
}

#[test]
fn test_mask_count_text_format() {
    let (stdout, _, ok) = run_mask(&["examples/heart.pxl", "--sprite", "heart", "--count"]);
    assert!(ok);
    assert!(stdout.contains("Token counts"), "text format has header");
    assert!(stdout.contains("{r}"), "should list main token");
    assert!(stdout.contains("%"), "should show percentages");
}

#[test]
fn test_mask_count_token_sum_equals_total() {
    let (stdout, _, ok) = run_mask(&["examples/coin.pxl", "--sprite", "coin", "--count", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let total = json["total"].as_u64().unwrap();
    let tokens = json["tokens"].as_object().unwrap();
    let sum: u64 = tokens.values().map(|v| v.as_u64().unwrap()).sum();
    assert_eq!(sum, total, "token counts should sum to total");
}

// ============================================================================
// Default grid dump (no operation flag)
// ============================================================================

#[test]
fn test_mask_default_grid_json() {
    let (stdout, _, ok) = run_mask(&["examples/coin.pxl", "--sprite", "coin", "--json"]);
    assert!(ok, "default grid dump should succeed");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["sprite"], "coin");
    assert_eq!(json["width"], 8);
    assert_eq!(json["height"], 8);
    let grid = json["grid"].as_array().unwrap();
    assert_eq!(grid.len(), 8, "8 rows");
    assert_eq!(grid[0].as_array().unwrap().len(), 8, "8 columns");
}

#[test]
fn test_mask_default_grid_text() {
    let (stdout, _, ok) = run_mask(&["examples/heart.pxl", "--sprite", "heart"]);
    assert!(ok);
    assert!(stdout.contains("Token grid for"), "text format has header");
    assert!(stdout.contains("heart"), "shows sprite name");
    assert!(stdout.contains("7x6"), "shows dimensions");
}

// ============================================================================
// Error handling
// ============================================================================

#[test]
fn test_mask_missing_sprite_flag() {
    let (_, stderr, ok) = run_mask(&["examples/hero.pxl", "--query", "{eye}"]);
    assert!(!ok, "should fail without --sprite");
    assert!(
        stderr.contains("--sprite") || stderr.contains("Available sprites"),
        "should mention --sprite requirement"
    );
}

#[test]
fn test_mask_sprite_not_found() {
    let (_, stderr, ok) = run_mask(&["examples/hero.pxl", "--sprite", "nonexistent", "--count"]);
    assert!(!ok, "should fail for unknown sprite");
    assert!(stderr.contains("not found"), "should report sprite not found");
    assert!(stderr.contains("hero_idle"), "should suggest available sprites");
}

#[test]
fn test_mask_file_not_found() {
    let (_, stderr, ok) = run_mask(&["nonexistent.pxl", "--list"]);
    assert!(!ok, "should fail for missing file");
    assert!(!stderr.is_empty(), "should have error message");
}

#[test]
fn test_mask_invalid_sample_coords() {
    let (_, stderr, ok) = run_mask(&["examples/coin.pxl", "--sprite", "coin", "--sample", "abc"]);
    assert!(!ok, "invalid coords should fail");
    assert!(!stderr.is_empty());
}

#[test]
fn test_mask_invalid_region_args() {
    let (_, stderr, ok) = run_mask(&["examples/coin.pxl", "--sprite", "coin", "--region", "1,2,3"]);
    assert!(!ok, "region with 3 values should fail");
    assert!(stderr.contains("x,y,w,h"), "should explain expected format");
}

// ============================================================================
// Cross-example verification (multiple example files)
// ============================================================================

#[test]
fn test_mask_all_examples_listable() {
    // Every example .pxl file should be listable
    for example in &["examples/hero.pxl", "examples/coin.pxl", "examples/heart.pxl"] {
        let (_, _, ok) = run_mask(&[example, "--list"]);
        assert!(ok, "{} should be listable", example);
    }
}

#[test]
fn test_mask_coin_sample_center() {
    // Center of coin (3,3) should be gold or one of its tokens
    let (stdout, _, ok) =
        run_mask(&["examples/coin.pxl", "--sprite", "coin", "--sample", "3,3", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let token = json["token"].as_str().unwrap();
    // Center should be gold (main body)
    assert_eq!(token, "{gold}", "center of coin should be gold");
}

#[test]
fn test_mask_heart_query_main_token() {
    let (stdout, _, ok) =
        run_mask(&["examples/heart.pxl", "--sprite", "heart", "--query", "{r}", "--json"]);
    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(json["count"].as_u64().unwrap() > 10, "heart should have many red pixels");
}
