//! Semantic sprite comparison
//!
//! Provides tools for comparing sprites and detecting differences in:
//! - Dimensions (width, height)
//! - Palette colors (added, removed, changed tokens)
//! - Grid content (row-by-row changes)

use crate::models::{PaletteRef, Sprite, TtpObject};
use crate::parser::parse_stream;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Result of comparing two sprites
#[derive(Debug, Clone)]
pub struct SpriteDiff {
    /// Change in dimensions, if any
    pub dimension_change: Option<DimensionChange>,
    /// Changes to palette colors
    pub palette_changes: Vec<PaletteChange>,
    /// Changes to grid content
    pub grid_changes: Vec<GridChange>,
    /// Human-readable summary of the diff
    pub summary: String,
}

impl SpriteDiff {
    /// Returns true if there are no differences
    pub fn is_empty(&self) -> bool {
        self.dimension_change.is_none()
            && self.palette_changes.is_empty()
            && self.grid_changes.is_empty()
    }
}

/// Change in sprite dimensions
#[derive(Debug, Clone, PartialEq)]
pub struct DimensionChange {
    /// Old dimensions (width, height)
    pub old: (u32, u32),
    /// New dimensions (width, height)
    pub new: (u32, u32),
}

/// A change to a palette token
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteChange {
    /// Token was added
    Added { token: String, color: String },
    /// Token was removed
    Removed { token: String },
    /// Token color was changed
    Changed { token: String, old_color: String, new_color: String },
}

/// A change to a grid row
#[derive(Debug, Clone)]
pub struct GridChange {
    /// Row number (0-indexed)
    pub row: usize,
    /// Human-readable description of the change
    pub description: String,
}

/// Context for sprite comparison, containing resolved palettes
struct DiffContext {
    /// Palettes from file A (name -> colors)
    palettes_a: HashMap<String, HashMap<String, String>>,
    /// Palettes from file B (name -> colors)
    palettes_b: HashMap<String, HashMap<String, String>>,
}

impl DiffContext {
    fn new() -> Self {
        Self { palettes_a: HashMap::new(), palettes_b: HashMap::new() }
    }

    /// Resolve a palette reference to its color map
    fn resolve_palette(
        &self,
        palette_ref: &PaletteRef,
        palettes: &HashMap<String, HashMap<String, String>>,
    ) -> HashMap<String, String> {
        match palette_ref {
            PaletteRef::Named(name) => palettes.get(name).cloned().unwrap_or_default(),
            PaletteRef::Inline(colors) => colors.clone(),
        }
    }
}

/// Compare two sprites and return their differences
pub fn diff_sprites(
    a: &Sprite,
    b: &Sprite,
    palette_a: &HashMap<String, String>,
    palette_b: &HashMap<String, String>,
) -> SpriteDiff {
    let mut palette_changes = Vec::new();
    let mut grid_changes = Vec::new();

    // Compare dimensions
    let dim_a = get_sprite_dimensions(a);
    let dim_b = get_sprite_dimensions(b);
    let dimension_change =
        if dim_a != dim_b { Some(DimensionChange { old: dim_a, new: dim_b }) } else { None };

    // Compare palettes
    let tokens_a: HashSet<_> = palette_a.keys().collect();
    let tokens_b: HashSet<_> = palette_b.keys().collect();

    // Find removed tokens
    for token in tokens_a.difference(&tokens_b) {
        palette_changes.push(PaletteChange::Removed { token: (*token).clone() });
    }

    // Find added tokens
    for token in tokens_b.difference(&tokens_a) {
        if let Some(color) = palette_b.get(*token) {
            palette_changes
                .push(PaletteChange::Added { token: (*token).clone(), color: color.clone() });
        }
    }

    // Find changed tokens
    for token in tokens_a.intersection(&tokens_b) {
        let color_a = palette_a.get(*token);
        let color_b = palette_b.get(*token);
        if color_a != color_b {
            if let (Some(old), Some(new)) = (color_a, color_b) {
                palette_changes.push(PaletteChange::Changed {
                    token: (*token).clone(),
                    old_color: old.clone(),
                    new_color: new.clone(),
                });
            }
        }
    }

    // Sort palette changes for consistent output
    palette_changes.sort_by(|a, b| {
        let token_a = match a {
            PaletteChange::Added { token, .. } => token,
            PaletteChange::Removed { token } => token,
            PaletteChange::Changed { token, .. } => token,
        };
        let token_b = match b {
            PaletteChange::Added { token, .. } => token,
            PaletteChange::Removed { token } => token,
            PaletteChange::Changed { token, .. } => token,
        };
        token_a.cmp(token_b)
    });

    // Grid comparison is no longer available (grid format deprecated)
    // grid_changes will always be empty

    // Generate summary
    let summary = generate_summary(&dimension_change, &palette_changes, &grid_changes);

    SpriteDiff { dimension_change, palette_changes, grid_changes, summary }
}

/// Get sprite dimensions from size field
fn get_sprite_dimensions(sprite: &Sprite) -> (u32, u32) {
    if let Some([w, h]) = sprite.size {
        return (w, h);
    }

    // Grid format deprecated - cannot infer dimensions without size field
    (0, 0)
}

/// Describe the change between two rows
fn describe_row_change(_row_idx: usize, old: &str, new: &str) -> String {
    // Grid tokenization removed - just compare strings
    if old == new {
        "No change".to_string()
    } else {
        "Row content changed".to_string()
    }
}

/// Generate a human-readable summary of the diff
fn generate_summary(
    dimension_change: &Option<DimensionChange>,
    palette_changes: &[PaletteChange],
    grid_changes: &[GridChange],
) -> String {
    let mut parts = Vec::new();

    if let Some(dim) = dimension_change {
        parts
            .push(format!("Dimensions: {}x{} → {}x{}", dim.old.0, dim.old.1, dim.new.0, dim.new.1));
    }

    let added_count =
        palette_changes.iter().filter(|c| matches!(c, PaletteChange::Added { .. })).count();
    let removed_count =
        palette_changes.iter().filter(|c| matches!(c, PaletteChange::Removed { .. })).count();
    let changed_count =
        palette_changes.iter().filter(|c| matches!(c, PaletteChange::Changed { .. })).count();

    if added_count > 0 || removed_count > 0 || changed_count > 0 {
        let mut palette_parts = Vec::new();
        if added_count > 0 {
            palette_parts.push(format!("+{} token(s)", added_count));
        }
        if removed_count > 0 {
            palette_parts.push(format!("-{} token(s)", removed_count));
        }
        if changed_count > 0 {
            palette_parts.push(format!("~{} color(s)", changed_count));
        }
        parts.push(format!("Palette: {}", palette_parts.join(", ")));
    }

    if !grid_changes.is_empty() {
        parts.push(format!("Grid: {} row(s) changed", grid_changes.len()));
    }

    if parts.is_empty() {
        "No differences".to_string()
    } else {
        parts.join(". ")
    }
}

/// Compare two files and return differences for each matching sprite
pub fn diff_files(path_a: &Path, path_b: &Path) -> Result<Vec<(String, SpriteDiff)>, String> {
    // Parse both files
    let file_a =
        File::open(path_a).map_err(|e| format!("Cannot open '{}': {}", path_a.display(), e))?;
    let file_b =
        File::open(path_b).map_err(|e| format!("Cannot open '{}': {}", path_b.display(), e))?;

    let result_a = parse_stream(BufReader::new(file_a));
    let result_b = parse_stream(BufReader::new(file_b));

    // Build context with palettes
    let mut ctx = DiffContext::new();

    // Collect sprites and palettes from file A
    let mut sprites_a: HashMap<String, Sprite> = HashMap::new();
    for obj in &result_a.objects {
        match obj {
            TtpObject::Palette(p) => {
                ctx.palettes_a.insert(p.name.clone(), p.colors.clone());
            }
            TtpObject::Sprite(s) => {
                sprites_a.insert(s.name.clone(), s.clone());
            }
            _ => {}
        }
    }

    // Collect sprites and palettes from file B
    let mut sprites_b: HashMap<String, Sprite> = HashMap::new();
    for obj in &result_b.objects {
        match obj {
            TtpObject::Palette(p) => {
                ctx.palettes_b.insert(p.name.clone(), p.colors.clone());
            }
            TtpObject::Sprite(s) => {
                sprites_b.insert(s.name.clone(), s.clone());
            }
            _ => {}
        }
    }

    // Find sprites to compare (present in both files)
    let mut diffs = Vec::new();

    // All sprite names from both files
    let mut all_names: Vec<_> = sprites_a
        .keys()
        .chain(sprites_b.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_names.sort();

    for name in all_names {
        match (sprites_a.get(&name), sprites_b.get(&name)) {
            (Some(sprite_a), Some(sprite_b)) => {
                // Both files have this sprite - compare them
                let palette_a = ctx.resolve_palette(&sprite_a.palette, &ctx.palettes_a);
                let palette_b = ctx.resolve_palette(&sprite_b.palette, &ctx.palettes_b);
                let diff = diff_sprites(sprite_a, sprite_b, &palette_a, &palette_b);
                diffs.push((name, diff));
            }
            (Some(_), None) => {
                // Sprite only in file A (removed)
                diffs.push((
                    name.clone(),
                    SpriteDiff {
                        dimension_change: None,
                        palette_changes: Vec::new(),
                        grid_changes: Vec::new(),
                        summary: format!("Sprite '{}' removed in second file", name),
                    },
                ));
            }
            (None, Some(_)) => {
                // Sprite only in file B (added)
                diffs.push((
                    name.clone(),
                    SpriteDiff {
                        dimension_change: None,
                        palette_changes: Vec::new(),
                        grid_changes: Vec::new(),
                        summary: format!("Sprite '{}' added in second file", name),
                    },
                ));
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(diffs)
}

/// Format a diff for display
pub fn format_diff(name: &str, diff: &SpriteDiff, file_a: &str, file_b: &str) -> String {
    let mut output = Vec::new();

    output.push(format!("Comparing sprite \"{}\" ({}) vs ({}):", name, file_a, file_b));
    output.push(String::new());

    // Dimensions
    if let Some(dim) = &diff.dimension_change {
        output
            .push(format!("Dimensions: {}x{} → {}x{}", dim.old.0, dim.old.1, dim.new.0, dim.new.1));
    } else if diff.is_empty() {
        output.push("No differences found.".to_string());
        return output.join("\n");
    } else {
        output.push("Dimensions: Same".to_string());
    }

    // Palette changes
    if !diff.palette_changes.is_empty() {
        output.push(String::new());
        output.push("Token changes:".to_string());
        for change in &diff.palette_changes {
            match change {
                PaletteChange::Added { token, color } => {
                    output.push(format!("  + {} = {}", token, color));
                }
                PaletteChange::Removed { token } => {
                    output.push(format!("  - {}", token));
                }
                PaletteChange::Changed { token, old_color, new_color } => {
                    output.push(format!("  ~ {} color: {} → {}", token, old_color, new_color));
                }
            }
        }
    }

    // Grid changes
    if !diff.grid_changes.is_empty() {
        output.push(String::new());
        output.push("Grid changes:".to_string());
        for change in &diff.grid_changes {
            output.push(format!("  Row {}: {}", change.row + 1, change.description));
        }
    }

    // Summary
    output.push(String::new());
    output.push(format!("Summary: {}", diff.summary));

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sprite(name: &str, palette: HashMap<String, String>, grid: Vec<&str>) -> Sprite {
        // Compute dimensions from grid (for backwards compatibility in tests)
        let height = grid.len() as u32;
        let width = grid.first().map(|r| r.matches('{').count() as u32).unwrap_or(0);

        Sprite {
            name: name.to_string(),
            size: if height > 0 && width > 0 { Some([width, height]) } else { None },
            palette: PaletteRef::Inline(palette),
            metadata: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_identical_sprites() {
        let palette = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{a}".to_string(), "#FF0000".to_string()),
        ]);

        let sprite = make_sprite("test", palette.clone(), vec!["{_}{a}{_}", "{a}{a}{a}"]);

        let diff = diff_sprites(&sprite, &sprite, &palette, &palette);

        assert!(diff.is_empty());
        assert!(diff.dimension_change.is_none());
        assert!(diff.palette_changes.is_empty());
        assert!(diff.grid_changes.is_empty());
    }

    #[test]
    fn test_color_change() {
        let palette_a = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{skin}".to_string(), "#FFCC99".to_string()),
        ]);
        let palette_b = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{skin}".to_string(), "#FFD4AA".to_string()),
        ]);

        let sprite_a = make_sprite("test", palette_a.clone(), vec!["{skin}{skin}"]);
        let sprite_b = make_sprite("test", palette_b.clone(), vec!["{skin}{skin}"]);

        let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

        assert!(!diff.is_empty());
        assert_eq!(diff.palette_changes.len(), 1);
        assert!(matches!(
            &diff.palette_changes[0],
            PaletteChange::Changed {
                token,
                old_color,
                new_color
            } if token == "{skin}" && old_color == "#FFCC99" && new_color == "#FFD4AA"
        ));
    }

    #[test]
    fn test_added_token() {
        let palette_a = HashMap::from([("{_}".to_string(), "#00000000".to_string())]);
        let palette_b = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{highlight}".to_string(), "#FFFFFF".to_string()),
        ]);

        let sprite_a = make_sprite("test", palette_a.clone(), vec!["{_}{_}"]);
        let sprite_b = make_sprite("test", palette_b.clone(), vec!["{_}{_}"]);

        let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

        assert!(!diff.is_empty());
        assert_eq!(diff.palette_changes.len(), 1);
        assert!(matches!(
            &diff.palette_changes[0],
            PaletteChange::Added { token, color }
            if token == "{highlight}" && color == "#FFFFFF"
        ));
    }

    #[test]
    fn test_removed_token() {
        let palette_a = HashMap::from([
            ("{_}".to_string(), "#00000000".to_string()),
            ("{old}".to_string(), "#FF0000".to_string()),
        ]);
        let palette_b = HashMap::from([("{_}".to_string(), "#00000000".to_string())]);

        let sprite_a = make_sprite("test", palette_a.clone(), vec!["{_}{_}"]);
        let sprite_b = make_sprite("test", palette_b.clone(), vec!["{_}{_}"]);

        let diff = diff_sprites(&sprite_a, &sprite_b, &palette_a, &palette_b);

        assert!(!diff.is_empty());
        assert_eq!(diff.palette_changes.len(), 1);
        assert!(matches!(
            &diff.palette_changes[0],
            PaletteChange::Removed { token } if token == "{old}"
        ));
    }

    #[test]
    fn test_dimension_change() {
        let palette = HashMap::from([("{a}".to_string(), "#FF0000".to_string())]);

        let sprite_a = Sprite {
            name: "test".to_string(),
            size: Some([8, 8]),
            palette: PaletteRef::Inline(palette.clone()),
            metadata: None,
            ..Default::default()
        };
        let sprite_b = Sprite {
            name: "test".to_string(),
            size: Some([16, 16]),
            palette: PaletteRef::Inline(palette.clone()),
            metadata: None,
            ..Default::default()
        };

        let diff = diff_sprites(&sprite_a, &sprite_b, &palette, &palette);

        assert!(diff.dimension_change.is_some());
        let dim = diff.dimension_change.unwrap();
        assert_eq!(dim.old, (8, 8));
        assert_eq!(dim.new, (16, 16));
    }    #[test]
    fn test_get_sprite_dimensions_from_size() {
        let sprite = Sprite {
            name: "test".to_string(),
            size: Some([16, 8]),
            palette: PaletteRef::Inline(HashMap::new()),
            metadata: None,
            ..Default::default()
        };
        assert_eq!(get_sprite_dimensions(&sprite), (16, 8));
    }

    #[test]
    fn test_get_sprite_dimensions_no_size() {
        // With grid format deprecated, sprites without size return (0, 0)
        let sprite = Sprite {
            name: "test".to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::new()),
            metadata: None,
            ..Default::default()
        };
        assert_eq!(get_sprite_dimensions(&sprite), (0, 0));
    }
}
