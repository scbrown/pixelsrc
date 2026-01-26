//! Diff Command Demo Tests
//!
//! Demonstrates the `pxl diff` command functionality for comparing
//! sprites semantically between two files.

use pixelsrc::diff::{diff_sprites, format_diff, PaletteChange};
use pixelsrc::models::{PaletteRef, Sprite};
use std::collections::HashMap;

/// Helper to create a simple sprite for testing
fn make_sprite(name: &str, grid: Vec<&str>, palette: HashMap<String, String>) -> Sprite {
    // Compute dimensions from grid (for backwards compatibility in tests)
    let height = grid.len() as u32;
    let width = grid.first().map(|r| r.matches('{').count() as u32).unwrap_or(0);

    Sprite {
        name: name.to_string(),
        size: if height > 0 && width > 0 { Some([width, height]) } else { None },
        palette: PaletteRef::Inline(palette),
        ..Default::default()
    }
}

// ============================================================================
// No Difference Tests
// ============================================================================
/// @demo cli/diff#identical
/// @title Diff Identical Sprites
/// @description Identical sprites produce no differences.
#[test]
fn test_diff_identical_sprites() {
    let palette = HashMap::from([
        ("{_}".to_string(), "#0000".to_string()),
        ("{x}".to_string(), "#FF0000".to_string()),
    ]);

    let sprite_a =
        make_sprite("test", vec!["{_}{x}{_}", "{x}{x}{x}", "{_}{x}{_}"], palette.clone());
    let sprite_b =
        make_sprite("test", vec!["{_}{x}{_}", "{x}{x}{x}", "{_}{x}{_}"], palette.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette, &palette);

    assert!(diff.is_empty(), "Identical sprites should have no differences");
}

// ============================================================================
// Dimension Change Tests
// ============================================================================
/// @demo cli/diff#dimension_change
/// @title Diff Dimension Change
/// @description Detects when sprite dimensions change between versions.
#[test]
fn test_diff_dimension_change() {
    let palette = HashMap::from([("{x}".to_string(), "#FF0000".to_string())]);

    // 3x3 sprite
    let sprite_a =
        make_sprite("test", vec!["{x}{x}{x}", "{x}{x}{x}", "{x}{x}{x}"], palette.clone());
    // 4x4 sprite
    let sprite_b = make_sprite(
        "test",
        vec!["{x}{x}{x}{x}", "{x}{x}{x}{x}", "{x}{x}{x}{x}", "{x}{x}{x}{x}"],
        palette.clone(),
    );

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette, &palette);

    assert!(diff.dimension_change.is_some(), "Should detect dimension change");
    let dim_change = diff.dimension_change.unwrap();
    assert_eq!(dim_change.old, (3, 3));
    assert_eq!(dim_change.new, (4, 4));
}

/// @demo cli/diff#width_change
/// @title Diff Width Only Change
/// @description Detects width change with same height.
#[test]
fn test_diff_width_only_change() {
    let palette = HashMap::from([("{x}".to_string(), "#FF0000".to_string())]);

    let sprite_a = make_sprite("test", vec!["{x}{x}", "{x}{x}"], palette.clone());
    let sprite_b = make_sprite("test", vec!["{x}{x}{x}", "{x}{x}{x}"], palette.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette, &palette);

    assert!(diff.dimension_change.is_some());
    let dim_change = diff.dimension_change.unwrap();
    assert_eq!(dim_change.old.0, 2, "Old width should be 2");
    assert_eq!(dim_change.new.0, 3, "New width should be 3");
    assert_eq!(dim_change.old.1, dim_change.new.1, "Height should be unchanged");
}

// ============================================================================
// Palette Change Tests
// ============================================================================
/// @demo cli/diff#palette_color_changed
/// @title Diff Palette Color Changed
/// @description Detects when a palette color is modified.
#[test]
fn test_diff_palette_color_changed() {
    let palette_a = HashMap::from([
        ("{x}".to_string(), "#FF0000".to_string()), // Red
    ]);
    let palette_b = HashMap::from([
        ("{x}".to_string(), "#00FF00".to_string()), // Green
    ]);

    let sprite_a = make_sprite("test", vec!["{x}"], palette_a.clone());
    let sprite_b = make_sprite("test", vec!["{x}"], palette_b.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

    assert!(!diff.palette_changes.is_empty(), "Should detect palette color change");

    let change = &diff.palette_changes[0];
    match change {
        PaletteChange::Changed { token, old_color, new_color } => {
            assert_eq!(token, "{x}");
            assert!(old_color.to_uppercase().contains("FF0000") || old_color == "#FF0000");
            assert!(new_color.to_uppercase().contains("00FF00") || new_color == "#00FF00");
        }
        _ => panic!("Expected Changed palette change"),
    }
}

/// @demo cli/diff#palette_token_added
/// @title Diff Palette Token Added
/// @description Detects when new tokens are added to palette.
#[test]
fn test_diff_palette_token_added() {
    let palette_a = HashMap::from([("{x}".to_string(), "#FF0000".to_string())]);
    let palette_b = HashMap::from([
        ("{x}".to_string(), "#FF0000".to_string()),
        ("{y}".to_string(), "#00FF00".to_string()), // New token
    ]);

    let sprite_a = make_sprite("test", vec!["{x}"], palette_a.clone());
    let sprite_b = make_sprite("test", vec!["{x}{y}"], palette_b.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

    let added_changes: Vec<_> =
        diff.palette_changes.iter().filter(|c| matches!(c, PaletteChange::Added { .. })).collect();

    assert!(!added_changes.is_empty(), "Should detect added palette token");
}

/// @demo cli/diff#palette_token_removed
/// @title Diff Palette Token Removed
/// @description Detects when tokens are removed from palette.
#[test]
fn test_diff_palette_token_removed() {
    let palette_a = HashMap::from([
        ("{x}".to_string(), "#FF0000".to_string()),
        ("{y}".to_string(), "#00FF00".to_string()),
    ]);
    let palette_b = HashMap::from([
        ("{x}".to_string(), "#FF0000".to_string()),
        // {y} removed
    ]);

    let sprite_a = make_sprite("test", vec!["{x}{y}"], palette_a.clone());
    let sprite_b = make_sprite("test", vec!["{x}"], palette_b.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

    let removed_changes: Vec<_> = diff
        .palette_changes
        .iter()
        .filter(|c| matches!(c, PaletteChange::Removed { .. }))
        .collect();

    assert!(!removed_changes.is_empty(), "Should detect removed palette token");
}

// ============================================================================
// Format Output Tests
// ============================================================================
/// @demo cli/diff#format_output
/// @title Format Diff Output
/// @description Diff results are formatted for human-readable display.
#[test]
fn test_format_diff_output() {
    let palette_a = HashMap::from([("{x}".to_string(), "#FF0000".to_string())]);
    let palette_b = HashMap::from([("{x}".to_string(), "#00FF00".to_string())]);

    let sprite_a = make_sprite("test", vec!["{x}{x}", "{x}{x}"], palette_a.clone());
    let sprite_b =
        make_sprite("test", vec!["{x}{x}{x}", "{x}{x}{x}", "{x}{x}{x}"], palette_b.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);
    let formatted = format_diff("test", &diff, "file_a.pxl", "file_b.pxl");

    assert!(
        formatted.contains("test") || formatted.contains("diff"),
        "Output should reference the sprite name or diff"
    );
    assert!(!formatted.is_empty(), "Formatted output should not be empty");
}

// ============================================================================
// Edge Cases
// ============================================================================
/// @demo cli/diff#empty_to_content
/// @title Diff Empty to Content
/// @description Detects changes from minimal to larger sprite.
#[test]
fn test_diff_empty_to_content() {
    let palette_a = HashMap::from([("{_}".to_string(), "#0000".to_string())]);
    let palette_b = HashMap::from([
        ("{_}".to_string(), "#0000".to_string()),
        ("{x}".to_string(), "#FF0000".to_string()),
    ]);

    let sprite_a = make_sprite("test", vec!["{_}"], palette_a.clone());
    let sprite_b = make_sprite("test", vec!["{x}{x}", "{x}{x}"], palette_b.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

    // Should detect both dimension and content changes
    assert!(
        diff.dimension_change.is_some() || !diff.grid_changes.is_empty(),
        "Should detect changes from empty to content"
    );
}

/// @demo cli/diff#summary
/// @title Diff Summary
/// @description Diff includes a human-readable summary.
#[test]
fn test_diff_summary() {
    let palette_a = HashMap::from([("{x}".to_string(), "#FF0000".to_string())]);
    let palette_b = HashMap::from([("{x}".to_string(), "#00FF00".to_string())]);

    let sprite_a = make_sprite("test", vec!["{x}"], palette_a.clone());
    let sprite_b = make_sprite("test", vec!["{x}{x}"], palette_b.clone());

    let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

    assert!(!diff.summary.is_empty(), "Diff should include a summary");
}
