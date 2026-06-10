//! Semantic sprite comparison
//!
//! Provides tools for comparing sprites and detecting differences in:
//! - Dimensions (width, height)
//! - Palette colors (added, removed, changed tokens)
//! - Region geometry (added, removed, shifted, or changed pixels per token)
//! - Presence (a sprite existing in only one of the two files)

use crate::models::{PaletteRef, RegionDef, Sprite, TtpObject};
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
    /// Changes to region geometry
    pub region_changes: Vec<RegionChange>,
    /// Set when the sprite exists in only one of the files
    pub presence_change: Option<String>,
    /// Human-readable summary of the diff
    pub summary: String,
}

impl SpriteDiff {
    /// Returns true if there are no differences
    pub fn is_empty(&self) -> bool {
        self.dimension_change.is_none()
            && self.palette_changes.is_empty()
            && self.region_changes.is_empty()
            && self.presence_change.is_none()
    }
}

/// A geometric change to one region (token) between two sprites
#[derive(Debug, Clone, PartialEq)]
pub enum RegionChange {
    /// Region exists only in the second sprite
    Added { token: String },
    /// Region exists only in the first sprite
    Removed { token: String },
    /// Same shape, translated as a unit (a walk-cycle bob shows up here)
    Shifted { token: String, dx: i32, dy: i32 },
    /// Shape changed: `differing` pixels in the symmetric difference
    Changed { token: String, differing: usize },
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

    // Compare region geometry (the part "No differences found" used to skip
    // entirely: two same-size, same-palette sprites with completely
    // different pixels diffed as identical).
    let region_changes = diff_regions(a, b);

    // Generate summary
    let summary = generate_summary(&dimension_change, &palette_changes, &region_changes);

    SpriteDiff { dimension_change, palette_changes, region_changes, presence_change: None, summary }
}

/// Rasterize both sprites' regions (renderer's two-pass order) and compare
/// them token by token. Sprites without regions+size (e.g. derived sprites)
/// are not compared — geometry comparison needs concrete regions.
fn diff_regions(a: &Sprite, b: &Sprite) -> Vec<RegionChange> {
    let (Some(regions_a), Some(regions_b)) = (&a.regions, &b.regions) else {
        return Vec::new();
    };
    let (Some([wa, ha]), Some([wb, hb])) = (a.size, b.size) else {
        return Vec::new();
    };

    let raster_a = rasterize_all(regions_a, wa as i32, ha as i32);
    let raster_b = rasterize_all(regions_b, wb as i32, hb as i32);

    let mut tokens: Vec<&String> =
        raster_a.keys().chain(raster_b.keys()).collect::<HashSet<_>>().into_iter().collect();
    tokens.sort();

    let mut changes = Vec::new();
    for token in tokens {
        match (raster_a.get(token), raster_b.get(token)) {
            (Some(_), None) => changes.push(RegionChange::Removed { token: token.clone() }),
            (None, Some(_)) => changes.push(RegionChange::Added { token: token.clone() }),
            (Some(pa), Some(pb)) => {
                if pa == pb {
                    continue;
                }
                if let Some((dx, dy)) = detect_shift(pa, pb) {
                    changes.push(RegionChange::Shifted { token: token.clone(), dx, dy });
                } else {
                    let differing = pa.symmetric_difference(pb).count();
                    changes.push(RegionChange::Changed { token: token.clone(), differing });
                }
            }
            (None, None) => unreachable!(),
        }
    }
    changes
}

fn rasterize_all(
    regions: &HashMap<String, RegionDef>,
    width: i32,
    height: i32,
) -> HashMap<String, HashSet<(i32, i32)>> {
    let mut warnings = Vec::new();
    let mut rasterized: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
    let mut pending: Vec<(&String, &RegionDef)> = Vec::new();
    for (token, region) in regions {
        if region.fill.is_some() || region.auto_shadow.is_some() {
            pending.push((token, region));
        } else {
            let pixels =
                crate::structured::rasterize_region(region, &rasterized, width, height, &mut warnings);
            rasterized.insert(token.clone(), pixels);
        }
    }
    for (token, region) in pending {
        let pixels =
            crate::structured::rasterize_region(region, &rasterized, width, height, &mut warnings);
        rasterized.insert(token.clone(), pixels);
    }
    rasterized
}

/// If `b` is exactly `a` translated by a constant offset, return it.
fn detect_shift(a: &HashSet<(i32, i32)>, b: &HashSet<(i32, i32)>) -> Option<(i32, i32)> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let min_a = (
        a.iter().map(|p| p.0).min().expect("non-empty"),
        a.iter().map(|p| p.1).min().expect("non-empty"),
    );
    let min_b = (
        b.iter().map(|p| p.0).min().expect("non-empty"),
        b.iter().map(|p| p.1).min().expect("non-empty"),
    );
    let (dx, dy) = (min_b.0 - min_a.0, min_b.1 - min_a.1);
    if (dx, dy) == (0, 0) {
        return None; // equal sets are handled before this
    }
    if a.iter().all(|(x, y)| b.contains(&(x + dx, y + dy))) {
        Some((dx, dy))
    } else {
        None
    }
}

/// Get sprite dimensions from size field
fn get_sprite_dimensions(sprite: &Sprite) -> (u32, u32) {
    if let Some([w, h]) = sprite.size {
        return (w, h);
    }

    // Grid format deprecated - cannot infer dimensions without size field
    (0, 0)
}

/// Generate a human-readable summary of the diff
fn generate_summary(
    dimension_change: &Option<DimensionChange>,
    palette_changes: &[PaletteChange],
    region_changes: &[RegionChange],
) -> String {
    let mut parts = Vec::new();

    if let Some(dim) = dimension_change {
        parts
            .push(format!("Dimensions: {}x{} → {}x{}", dim.old.0, dim.old.1, dim.new.0, dim.new.1));
    }

    if !region_changes.is_empty() {
        parts.push(format!("Regions: {} changed", region_changes.len()));
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
                let note = format!("Sprite '{}' removed in second file", name);
                diffs.push((
                    name.clone(),
                    SpriteDiff {
                        dimension_change: None,
                        palette_changes: Vec::new(),
                        region_changes: Vec::new(),
                        presence_change: Some(note.clone()),
                        summary: note,
                    },
                ));
            }
            (None, Some(_)) => {
                // Sprite only in file B (added)
                let note = format!("Sprite '{}' added in second file", name);
                diffs.push((
                    name.clone(),
                    SpriteDiff {
                        dimension_change: None,
                        palette_changes: Vec::new(),
                        region_changes: Vec::new(),
                        presence_change: Some(note.clone()),
                        summary: note,
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

    // Presence: the sprite exists in only one file — say that, never
    // "No differences found" (which is what this case used to print).
    if let Some(note) = &diff.presence_change {
        output.push(note.clone());
        return output.join("\n");
    }

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

    // Region geometry changes
    if !diff.region_changes.is_empty() {
        output.push(String::new());
        output.push("Region changes:".to_string());
        for change in &diff.region_changes {
            match change {
                RegionChange::Added { token } => {
                    output.push(format!("  + {} (new region)", token));
                }
                RegionChange::Removed { token } => {
                    output.push(format!("  - {} (region gone)", token));
                }
                RegionChange::Shifted { token, dx, dy } => {
                    output.push(format!("  → {} shifted ({}, {})", token, dx, dy));
                }
                RegionChange::Changed { token, differing } => {
                    output.push(format!("  ~ {} shape changed ({} pixel(s) differ)", token, differing));
                }
            }
        }
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

    // Summary
    output.push(String::new());
    output.push(format!("Summary: {}", diff.summary));

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite_with_regions(name: &str, json_regions: &str) -> Sprite {
        let json = format!(
            r##"{{"type": "sprite", "name": "{}", "size": [16, 16],
                 "palette": {{"_": "transparent", "a": "#FF0000", "b": "#00FF00"}},
                 "regions": {} }}"##,
            name, json_regions
        );
        json5::from_str::<Sprite>(&json).expect("test sprite parses")
    }

    #[test]
    fn test_region_shift_detected() {
        // The walk-cycle bob: same shape, one region moved up a pixel.
        let a = sprite_with_regions("s", r#"{"a": {"rect": [4, 4, 3, 3]}, "b": {"rect": [0, 0, 2, 2]}}"#);
        let b = sprite_with_regions("s", r#"{"a": {"rect": [4, 3, 3, 3]}, "b": {"rect": [0, 0, 2, 2]}}"#);
        let palette = HashMap::new();
        let diff = diff_sprites(&a, &b, &palette, &palette);
        assert_eq!(
            diff.region_changes,
            vec![RegionChange::Shifted { token: "a".to_string(), dx: 0, dy: -1 }]
        );
        assert!(!diff.is_empty(), "a shifted region is a difference");
    }

    #[test]
    fn test_region_shape_change_detected() {
        let a = sprite_with_regions("s", r#"{"a": {"rect": [4, 4, 3, 3]}}"#);
        let b = sprite_with_regions("s", r#"{"a": {"rect": [4, 4, 3, 2]}}"#);
        let palette = HashMap::new();
        let diff = diff_sprites(&a, &b, &palette, &palette);
        assert_eq!(
            diff.region_changes,
            vec![RegionChange::Changed { token: "a".to_string(), differing: 3 }]
        );
    }

    #[test]
    fn test_region_added_and_removed_detected() {
        let a = sprite_with_regions("s", r#"{"a": {"rect": [4, 4, 3, 3]}}"#);
        let b = sprite_with_regions("s", r#"{"b": {"rect": [4, 4, 3, 3]}}"#);
        let palette = HashMap::new();
        let diff = diff_sprites(&a, &b, &palette, &palette);
        assert!(diff.region_changes.contains(&RegionChange::Added { token: "b".to_string() }));
        assert!(diff.region_changes.contains(&RegionChange::Removed { token: "a".to_string() }));
    }

    #[test]
    fn test_presence_diff_never_says_no_differences() {
        let diff = SpriteDiff {
            dimension_change: None,
            palette_changes: Vec::new(),
            region_changes: Vec::new(),
            presence_change: Some("Sprite 'x' removed in second file".to_string()),
            summary: "Sprite 'x' removed in second file".to_string(),
        };
        assert!(!diff.is_empty());
        let text = format_diff("x", &diff, "a.pxl", "b.pxl");
        assert!(text.contains("removed in second file"), "got: {}", text);
        assert!(!text.contains("No differences"), "got: {}", text);
    }

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
        assert!(diff.palette_changes.is_empty());
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
    }
    #[test]
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
