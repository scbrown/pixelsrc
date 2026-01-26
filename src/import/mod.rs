//! PNG import functionality for converting images to Pixelsrc format.
//!
//! This module provides functionality to:
//! - Read PNG images and extract unique colors
//! - Quantize colors using median cut algorithm if too many colors
//! - Generate Pixelsrc JSONL output with palette and sprite definitions
//! - Detect shapes, symmetry, roles, and relationships when analysis is enabled

mod analysis;
mod color_quantization;
mod detection;
mod structured_regions;

use image::GenericImageView;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::analyze::{
    detect_symmetry, infer_relationships_batch, infer_roles_batch, RegionData,
    RoleInferenceContext, Symmetric,
};
use crate::models::{RelationshipType, Role};

pub use color_quantization::Color;
use color_quantization::{find_closest_color, median_cut_quantize_lab};
pub use structured_regions::{
    extract_structured_regions, filter_points_for_half_sprite,
    filter_structured_region_for_half_sprite, StructuredRegion,
};

use analysis::generate_naming_hints;
use detection::{detect_dither_patterns, detect_outlines, detect_upscale, infer_z_order};

/// How to handle detected dither patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DitherHandling {
    /// Keep dithered regions as-is (separate color tokens).
    #[default]
    Keep,
    /// Merge dithered regions into a single averaged color.
    Merge,
    /// Only detect and flag dithered regions (no merging).
    Analyze,
}

/// A detected dither pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherPattern {
    /// Checkerboard pattern (alternating colors in 2x2 grid).
    Checkerboard,
    /// Ordered dither (Bayer matrix patterns).
    Ordered,
    /// Horizontal line dither.
    HorizontalLines,
    /// Vertical line dither.
    VerticalLines,
}

impl std::fmt::Display for DitherPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DitherPattern::Checkerboard => write!(f, "checkerboard"),
            DitherPattern::Ordered => write!(f, "ordered"),
            DitherPattern::HorizontalLines => write!(f, "horizontal-lines"),
            DitherPattern::VerticalLines => write!(f, "vertical-lines"),
        }
    }
}

/// Information about a detected dithered region.
#[derive(Debug, Clone)]
pub struct DitherInfo {
    /// The tokens involved in the dither pattern.
    pub tokens: Vec<String>,
    /// The detected pattern type.
    pub pattern: DitherPattern,
    /// Bounding box of the dithered region [x, y, width, height].
    pub bounds: [u32; 4],
    /// Suggested merged color (hex) if merging is desired.
    pub merged_color: String,
    /// Confidence of the detection (0.0-1.0).
    pub confidence: f64,
}

/// Information about detected upscaling.
#[derive(Debug, Clone)]
pub struct UpscaleInfo {
    /// Detected scale factor (e.g., 2 means 2x upscaled).
    pub scale: u32,
    /// Native resolution [width, height] before upscaling.
    pub native_size: [u32; 2],
    /// Confidence of the detection (0.0-1.0).
    pub confidence: f64,
}

/// Information about a detected outline/stroke region.
#[derive(Debug, Clone)]
pub struct OutlineInfo {
    /// The token that appears to be an outline.
    pub token: String,
    /// The tokens this outlines/borders.
    pub borders: Vec<String>,
    /// Average width of the outline in pixels.
    pub width: f64,
    /// Confidence of the detection (0.0-1.0).
    pub confidence: f64,
}

/// Options for PNG import.
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Enable role/relationship inference
    pub analyze: bool,
    /// Confidence threshold for inferences (0.0-1.0)
    pub confidence_threshold: f64,
    /// Generate token naming hints
    pub hints: bool,
    /// Extract structured regions (polygons, rects) instead of raw points
    pub extract_shapes: bool,
    /// Export only half the sprite data when symmetry is detected.
    /// The symmetry flag will indicate how to mirror the data during rendering.
    pub half_sprite: bool,
    /// How to handle dither patterns.
    pub dither_handling: DitherHandling,
    /// Detect if image appears to be upscaled pixel art.
    pub detect_upscale: bool,
    /// Detect thin dark regions that may be outlines/strokes.
    pub detect_outlines: bool,
}

/// A naming hint for a token based on detected features.
#[derive(Debug, Clone)]
pub struct NamingHint {
    /// The current token name
    pub token: String,
    /// Suggested name based on detected features
    pub suggested_name: String,
    /// Reason for the suggestion
    pub reason: String,
}

/// Analysis results from import.
#[derive(Debug, Clone, Default)]
pub struct ImportAnalysis {
    /// Inferred roles for tokens (token -> role)
    pub roles: HashMap<String, Role>,
    /// Inferred relationships between tokens
    pub relationships: Vec<(String, RelationshipType, String)>,
    /// Detected symmetry
    pub symmetry: Option<Symmetric>,
    /// Token naming hints
    pub naming_hints: Vec<NamingHint>,
    /// Inferred z-order from spatial containment (token -> z-level)
    /// Higher z means the region should be rendered on top.
    pub z_order: HashMap<String, i32>,
    /// Detected dither patterns and their info.
    pub dither_patterns: Vec<DitherInfo>,
    /// Detected upscaling info (if image appears to be upscaled pixel art).
    pub upscale_info: Option<UpscaleInfo>,
    /// Detected outline/stroke regions.
    pub outlines: Vec<OutlineInfo>,
}

/// Result of importing a PNG image.
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// The generated sprite name.
    pub name: String,
    /// Width of the sprite in pixels.
    pub width: u32,
    /// Height of the sprite in pixels.
    pub height: u32,
    /// Color palette mapping tokens to hex colors.
    pub palette: HashMap<String, String>,
    /// Grid rows with token sequences (legacy format).
    pub grid: Vec<String>,
    /// Region definitions for each token (v2 format) - raw points.
    pub regions: HashMap<String, Vec<[u32; 2]>>,
    /// Structured region definitions (polygons, rects, unions).
    pub structured_regions: Option<HashMap<String, StructuredRegion>>,
    /// Analysis results (if analysis was enabled).
    pub analysis: Option<ImportAnalysis>,
    /// Whether half-sprite export is enabled.
    pub half_sprite: bool,
}

impl ImportResult {
    /// Serialize to JSONL format (palette line + sprite line with regions).
    pub fn to_jsonl(&self) -> String {
        let palette_json = serde_json::json!({
            "type": "palette",
            "name": format!("{}_palette", self.name),
            "colors": self.palette
        });

        // Build regions object - use structured regions if available, fallback to points
        let regions: HashMap<String, serde_json::Value> =
            if let Some(ref structured) = self.structured_regions {
                structured.iter().map(|(token, region)| (token.clone(), region.to_json())).collect()
            } else {
                self.regions
                    .iter()
                    .map(|(token, points)| (token.clone(), serde_json::json!({ "points": points })))
                    .collect()
            };

        let sprite_json = serde_json::json!({
            "type": "sprite",
            "name": self.name,
            "size": [self.width, self.height],
            "palette": format!("{}_palette", self.name),
            "regions": regions
        });

        format!("{}\n{}", palette_json, sprite_json)
    }

    /// Serialize to structured JSONL format (v2 with regions, roles, relationships).
    ///
    /// If `half_sprite` is true and symmetry is detected, only the primary half
    /// of the sprite data is exported, with a `symmetry` field indicating how
    /// to mirror the data during rendering.
    pub fn to_structured_jsonl(&self) -> String {
        let mut palette_obj = serde_json::json!({
            "type": "palette",
            "name": format!("{}_palette", self.name),
            "colors": self.palette
        });

        // Add roles if analysis was performed
        if let Some(ref analysis) = self.analysis {
            if !analysis.roles.is_empty() {
                let roles: HashMap<String, String> =
                    analysis.roles.iter().map(|(k, v)| (k.clone(), v.to_string())).collect();
                palette_obj["roles"] = serde_json::json!(roles);
            }

            // Add relationships
            if !analysis.relationships.is_empty() {
                let relationships: HashMap<String, serde_json::Value> = analysis
                    .relationships
                    .iter()
                    .map(|(source, rel_type, target)| {
                        let rel_str = match rel_type {
                            RelationshipType::DerivesFrom => "derives-from",
                            RelationshipType::ContainedWithin => "contained-within",
                            RelationshipType::AdjacentTo => "adjacent-to",
                            RelationshipType::PairedWith => "paired-with",
                        };
                        (
                            source.clone(),
                            serde_json::json!({
                                "type": rel_str,
                                "target": target
                            }),
                        )
                    })
                    .collect();
                palette_obj["relationships"] = serde_json::json!(relationships);
            }
        }

        // Determine if we should apply half-sprite filtering
        let apply_half_sprite = self.half_sprite
            && self.analysis.as_ref().map(|a| a.symmetry.is_some()).unwrap_or(false);
        let symmetry = self.analysis.as_ref().and_then(|a| a.symmetry);

        // Build regions object - use structured regions if available, adding z-order if present
        let z_order = self.analysis.as_ref().map(|a| &a.z_order);
        let regions: HashMap<String, serde_json::Value> =
            if let Some(ref structured) = self.structured_regions {
                structured
                    .iter()
                    .filter_map(|(token, region)| {
                        // Apply half-sprite filtering if enabled
                        let filtered_region = if apply_half_sprite {
                            let sym = symmetry.unwrap();
                            let filtered = filter_structured_region_for_half_sprite(
                                region,
                                sym,
                                self.width,
                                self.height,
                            );
                            // Skip empty regions
                            if matches!(&filtered, StructuredRegion::Points(p) if p.is_empty()) {
                                return None;
                            }
                            filtered
                        } else {
                            region.clone()
                        };

                        let mut region_json = filtered_region.to_json();
                        // Add z-order if available
                        if let Some(z_map) = z_order {
                            if let Some(&z) = z_map.get(token) {
                                if let serde_json::Value::Object(ref mut obj) = region_json {
                                    obj.insert("z".to_string(), serde_json::json!(z));
                                }
                            }
                        }
                        Some((token.clone(), region_json))
                    })
                    .collect()
            } else {
                self.regions
                    .iter()
                    .filter_map(|(token, points)| {
                        // Apply half-sprite filtering if enabled
                        let filtered_points = if apply_half_sprite {
                            let sym = symmetry.unwrap();
                            let pts =
                                filter_points_for_half_sprite(points, sym, self.width, self.height);
                            // Skip empty regions
                            if pts.is_empty() {
                                return None;
                            }
                            pts
                        } else {
                            points.clone()
                        };

                        let mut region_json = serde_json::json!({ "points": filtered_points });
                        // Add z-order if available
                        if let Some(z_map) = z_order {
                            if let Some(&z) = z_map.get(token) {
                                if let serde_json::Value::Object(ref mut obj) = region_json {
                                    obj.insert("z".to_string(), serde_json::json!(z));
                                }
                            }
                        }
                        Some((token.clone(), region_json))
                    })
                    .collect()
            };

        let mut sprite_obj = serde_json::json!({
            "type": "sprite",
            "name": self.name,
            "size": [self.width, self.height],
            "palette": format!("{}_palette", self.name),
            "regions": regions
        });

        // Add symmetry if detected and half-sprite is enabled
        if apply_half_sprite {
            if let Some(sym) = symmetry {
                let sym_str = match sym {
                    Symmetric::X => "x",
                    Symmetric::Y => "y",
                    Symmetric::XY => "xy",
                };
                sprite_obj["symmetry"] = serde_json::json!(sym_str);
            }
        }

        format!("{}\n{}", palette_obj, sprite_obj)
    }
}

/// Import a PNG file and convert it to Pixelsrc format (legacy, no analysis).
pub fn import_png<P: AsRef<Path>>(
    path: P,
    name: &str,
    max_colors: usize,
) -> Result<ImportResult, String> {
    let options = ImportOptions::default();
    import_png_with_options(path, name, max_colors, &options)
}

/// Import a PNG file with analysis options.
pub fn import_png_with_options<P: AsRef<Path>>(
    path: P,
    name: &str,
    max_colors: usize,
    options: &ImportOptions,
) -> Result<ImportResult, String> {
    let img = image::open(path.as_ref()).map_err(|e| format!("Failed to open image: {}", e))?;

    let (width, height) = img.dimensions();

    // Extract all unique colors with their pixel counts
    let mut color_counts: HashMap<Color, u32> = HashMap::new();
    for (_, _, pixel) in img.pixels() {
        let color = Color::from_rgba(pixel);
        *color_counts.entry(color).or_insert(0) += 1;
    }

    // Quantize if needed using perceptual LAB color space
    let palette_colors = median_cut_quantize_lab(color_counts.clone(), max_colors);

    // Build color to index mapping
    let original_colors: Vec<Color> = color_counts.keys().cloned().collect();

    // Map original colors to palette colors
    let mut color_to_palette_idx: HashMap<Color, usize> = HashMap::new();
    for orig_color in &original_colors {
        let idx = find_closest_color(*orig_color, &palette_colors);
        color_to_palette_idx.insert(*orig_color, idx);
    }

    // Generate token names
    // Find transparent color index for special {_} token
    let transparent_idx = palette_colors.iter().position(|c| c.is_transparent());

    let mut palette: HashMap<String, String> = HashMap::new();
    let mut idx_to_token: HashMap<usize, String> = HashMap::new();
    let mut idx_to_color: HashMap<usize, Color> = HashMap::new();

    let mut color_num = 1;
    for (idx, color) in palette_colors.iter().enumerate() {
        let token = if Some(idx) == transparent_idx {
            "{_}".to_string()
        } else {
            let t = format!("{{c{}}}", color_num);
            color_num += 1;
            t
        };
        palette.insert(token.clone(), color.to_hex());
        idx_to_token.insert(idx, token);
        idx_to_color.insert(idx, *color);
    }

    // Build grid and regions simultaneously
    let mut grid: Vec<String> = Vec::with_capacity(height as usize);
    let mut regions: HashMap<String, Vec<[u32; 2]>> = HashMap::new();
    let mut token_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();

    for y in 0..height {
        let mut row = String::new();
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let color = Color::from_rgba(pixel);
            let palette_idx = color_to_palette_idx[&color];
            let token = &idx_to_token[&palette_idx];
            row.push_str(token);

            // Add to regions
            regions.entry(token.clone()).or_default().push([x, y]);
            token_pixels.entry(token.clone()).or_default().insert((x as i32, y as i32));
        }
        grid.push(row);
    }

    // Perform analysis if requested
    let analysis = if options.analyze {
        Some(perform_analysis(width, height, &token_pixels, &idx_to_token, &idx_to_color, options))
    } else {
        None
    };

    // Extract structured regions if requested
    let structured_regions = if options.extract_shapes {
        Some(
            regions
                .iter()
                .map(|(token, points)| {
                    (token.clone(), extract_structured_regions(points, width, height))
                })
                .collect(),
        )
    } else {
        None
    };

    Ok(ImportResult {
        name: name.to_string(),
        width,
        height,
        palette,
        grid,
        regions,
        structured_regions,
        analysis,
        half_sprite: options.half_sprite,
    })
}

/// Perform analysis on imported regions.
fn perform_analysis(
    width: u32,
    height: u32,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    idx_to_token: &HashMap<usize, String>,
    idx_to_color: &HashMap<usize, Color>,
    options: &ImportOptions,
) -> ImportAnalysis {
    let mut analysis = ImportAnalysis::default();

    // Build token to color mapping
    let token_to_color: HashMap<String, [u8; 4]> = idx_to_token
        .iter()
        .filter_map(|(idx, token)| {
            idx_to_color.get(idx).map(|c| (token.clone(), [c.r, c.g, c.b, c.a]))
        })
        .collect();

    // Detect symmetry using raw pixel data
    // For symmetry detection, we need the raw pixel bytes
    // We'll create a simplified version based on the grid
    let bpp = 4;
    let mut pixel_data = vec![0u8; (width * height * bpp as u32) as usize];
    for (token, pixels) in token_pixels {
        if let Some(color) = token_to_color.get(token) {
            for &(x, y) in pixels {
                let idx = ((y as u32 * width + x as u32) * bpp as u32) as usize;
                if idx + 3 < pixel_data.len() {
                    pixel_data[idx] = color[0];
                    pixel_data[idx + 1] = color[1];
                    pixel_data[idx + 2] = color[2];
                    pixel_data[idx + 3] = color[3];
                }
            }
        }
    }

    // Detect symmetry
    analysis.symmetry = detect_symmetry(&pixel_data, width, height);

    // Prepare data for role inference
    let ctx = RoleInferenceContext::new(width, height);
    let role_input: HashMap<String, (HashSet<(i32, i32)>, Option<[u8; 4]>)> = token_pixels
        .iter()
        .map(|(token, pixels)| {
            let color = token_to_color.get(token).copied();
            (token.clone(), (pixels.clone(), color))
        })
        .collect();

    // Infer roles
    let (role_inferences, _warnings) = infer_roles_batch(&role_input, &ctx);
    for (token, inference) in role_inferences {
        if inference.confidence >= options.confidence_threshold {
            analysis.roles.insert(token, inference.role);
        }
    }

    // Prepare data for relationship inference
    let region_data: Vec<RegionData> = token_pixels
        .iter()
        .map(|(token, pixels)| {
            let color = token_to_color.get(token).copied().unwrap_or([0, 0, 0, 255]);
            RegionData { name: token.clone(), pixels: pixels.clone(), color }
        })
        .collect();

    // Infer relationships
    let rel_inferences = infer_relationships_batch(&region_data, width);
    for rel in rel_inferences {
        if rel.confidence >= options.confidence_threshold {
            analysis.relationships.push((rel.source, rel.relationship_type, rel.target));
        }
    }

    // Infer z-order from containment relationships
    let tokens: Vec<String> = token_pixels.keys().cloned().collect();
    analysis.z_order = infer_z_order(&tokens, &analysis.relationships);

    // Detect dither patterns if dither handling is not Keep
    if options.dither_handling != DitherHandling::Keep {
        let mut patterns = detect_dither_patterns(width, height, token_pixels, &token_to_color);
        // Filter by confidence threshold
        patterns.retain(|p| p.confidence >= options.confidence_threshold);
        analysis.dither_patterns = patterns;
    }

    // Detect upscaled pixel art if requested
    if options.detect_upscale {
        if let Some(upscale_info) = detect_upscale(&pixel_data, width, height) {
            if upscale_info.confidence >= options.confidence_threshold {
                analysis.upscale_info = Some(upscale_info);
            }
        }
    }

    // Detect outline/stroke regions if requested
    if options.detect_outlines {
        let mut outlines = detect_outlines(token_pixels, &token_to_color, width, height);
        outlines.retain(|o| o.confidence >= options.confidence_threshold);
        analysis.outlines = outlines;
    }

    // Generate naming hints if requested
    if options.hints {
        analysis.naming_hints =
            generate_naming_hints(&analysis.roles, token_pixels, &token_to_color, width, height);
    }

    analysis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_result_to_jsonl() {
        let mut palette = HashMap::new();
        palette.insert("{_}".to_string(), "#00000000".to_string());
        palette.insert("{c1}".to_string(), "#FF0000".to_string());

        let mut regions = HashMap::new();
        regions.insert("{c1}".to_string(), vec![[0, 0], [1, 1]]);
        regions.insert("{_}".to_string(), vec![[1, 0], [0, 1]]);

        let result = ImportResult {
            name: "test_sprite".to_string(),
            width: 2,
            height: 2,
            palette,
            grid: vec!["{c1}{_}".to_string(), "{_}{c1}".to_string()],
            regions,
            structured_regions: None,
            analysis: None,
            half_sprite: false,
        };

        let jsonl = result.to_jsonl();
        assert!(jsonl.contains("\"type\":\"palette\""));
        assert!(jsonl.contains("\"type\":\"sprite\""));
        assert!(jsonl.contains("test_sprite_palette"));
        assert!(jsonl.contains("test_sprite"));
        assert!(jsonl.contains("\"regions\""));
        assert!(!jsonl.contains("\"grid\""));
    }
}
