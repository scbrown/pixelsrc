//! Structured sprite rendering for region-based sprites.
//!
//! This module provides rendering for structured sprites that use the regions format
//! instead of the grid format. Regions are defined using shape primitives and compound
//! operations, then rasterized and rendered in z-order.

use crate::color::parse_color;
use crate::models::{RegionDef, Role};
use crate::path::parse_path;
use crate::renderer::Warning;
use crate::shapes::{
    flood_fill, intersect, rasterize_ellipse, rasterize_line, rasterize_points,
    rasterize_polygon, rasterize_rect, rasterize_stroke, subtract, union,
};
use image::{Rgba, RgbaImage};
use std::collections::{HashMap, HashSet};

/// Magenta color used for unknown tokens and invalid colors
const MAGENTA: Rgba<u8> = Rgba([255, 0, 255, 255]);

/// Transparent color used for padding
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Default z-order values for semantic roles.
///
/// When a region has a role but no explicit z-value, these defaults are used.
/// Higher values render on top of lower values.
///
/// Priority (highest to lowest):
/// - anchor (100): Critical details like eyes that must be visible
/// - boundary (80): Outlines and edges
/// - shadow/highlight (60): Depth indicators
/// - fill (40): Interior mass
/// - no role (0): Default for untagged regions
fn default_z_for_role(role: Option<&Role>) -> i32 {
    match role {
        Some(Role::Anchor) => 100,
        Some(Role::Boundary) => 80,
        Some(Role::Shadow) | Some(Role::Highlight) => 60,
        Some(Role::Fill) => 40,
        None => 0,
    }
}

/// Rasterize a RegionDef into a set of pixel coordinates.
///
/// This function recursively evaluates all shape primitives and compound operations
/// to produce a final set of pixels.
///
/// # Arguments
///
/// * `region` - The region definition to rasterize
/// * `all_regions` - Map of all regions (for reference in fill operations)
/// * `canvas_width` - Width of the canvas
/// * `canvas_height` - Height of the canvas
/// * `warnings` - Vector to collect warnings during rasterization
///
/// # Returns
///
/// A HashSet of (x, y) coordinates representing the pixels in this region.
pub fn rasterize_region(
    region: &RegionDef,
    all_regions: &HashMap<String, HashSet<(i32, i32)>>,
    canvas_width: i32,
    canvas_height: i32,
    warnings: &mut Vec<Warning>,
) -> HashSet<(i32, i32)> {
    let mut pixels = HashSet::new();

    // Handle shape primitives
    if let Some(points_data) = &region.points {
        let points: Vec<(i32, i32)> =
            points_data.iter().map(|[x, y]| (*x as i32, *y as i32)).collect();
        pixels = rasterize_points(&points);
    } else if let Some(line_data) = &region.line {
        if line_data.len() >= 2 {
            for i in 0..line_data.len() - 1 {
                let p0 = (line_data[i][0] as i32, line_data[i][1] as i32);
                let p1 = (line_data[i + 1][0] as i32, line_data[i + 1][1] as i32);
                let line_pixels = rasterize_line(p0, p1);
                for pixel in line_pixels {
                    pixels.insert(pixel);
                }
            }
        }
    } else if let Some([x, y, w, h]) = region.rect {
        pixels = rasterize_rect(x as i32, y as i32, w as i32, h as i32);
    } else if let Some([x, y, w, h]) = region.stroke {
        let thickness = region.thickness.unwrap_or(1) as i32;
        pixels = rasterize_stroke(x as i32, y as i32, w as i32, h as i32, thickness);
    } else if let Some([cx, cy, rx, ry]) = region.ellipse {
        pixels = rasterize_ellipse(cx as i32, cy as i32, rx as i32, ry as i32);
    } else if let Some([cx, cy, r]) = region.circle {
        pixels = rasterize_ellipse(cx as i32, cy as i32, r as i32, r as i32);
    } else if let Some(polygon_data) = &region.polygon {
        let vertices: Vec<(i32, i32)> =
            polygon_data.iter().map(|[x, y]| (*x as i32, *y as i32)).collect();
        pixels = rasterize_polygon(&vertices);
    } else if let Some(path_str) = &region.path {
        match parse_path(path_str) {
            Ok(vertices) => {
                let vertices_i32: Vec<(i32, i32)> =
                    vertices.iter().map(|[x, y]| (*x as i32, *y as i32)).collect();
                pixels = rasterize_polygon(&vertices_i32);
            }
            Err(e) => {
                warnings.push(Warning::new(format!("Invalid path: {}", e)));
            }
        }
    } else if let Some(fill_ref) = &region.fill {
        // Parse fill reference: "inside(token_name)"
        if let Some(token_name) = parse_fill_reference(fill_ref) {
            if let Some(boundary) = all_regions.get(token_name) {
                pixels = flood_fill(boundary, None, canvas_width, canvas_height);
            } else {
                warnings.push(Warning::new(format!(
                    "Unknown token '{}' in fill reference",
                    token_name
                )));
            }
        } else {
            warnings.push(Warning::new(format!("Invalid fill reference: {}", fill_ref)));
        }
    } else if let Some(source_name) = &region.auto_shadow {
        // Generate shadow by offsetting source region's pixels
        if let Some(source_pixels) = all_regions.get(source_name) {
            let offset = region.offset.unwrap_or([1, 1]);
            for (x, y) in source_pixels {
                pixels.insert((x + offset[0], y + offset[1]));
            }
        } else {
            warnings.push(Warning::new(format!(
                "Unknown token '{}' in auto-shadow reference",
                source_name
            )));
        }
    }
    // Handle compound operations
    else if let Some(union_regions) = &region.union {
        let mut sub_regions = Vec::new();
        for sub_region in union_regions {
            let sub_pixels = rasterize_region(
                sub_region,
                all_regions,
                canvas_width,
                canvas_height,
                warnings,
            );
            sub_regions.push(sub_pixels);
        }
        pixels = union(&sub_regions);
    } else if let Some(base_region) = &region.base {
        // Subtract operation: base - subtract
        let base_pixels =
            rasterize_region(base_region, all_regions, canvas_width, canvas_height, warnings);

        if let Some(subtract_regions) = &region.subtract {
            let mut subtract_sets = Vec::new();
            for sub_region in subtract_regions {
                let sub_pixels = rasterize_region(
                    sub_region,
                    all_regions,
                    canvas_width,
                    canvas_height,
                    warnings,
                );
                subtract_sets.push(sub_pixels);
            }
            pixels = subtract(&base_pixels, &subtract_sets);
        } else {
            pixels = base_pixels;
        }
    } else if let Some(intersect_regions) = &region.intersect {
        let mut sub_regions = Vec::new();
        for sub_region in intersect_regions {
            let sub_pixels = rasterize_region(
                sub_region,
                all_regions,
                canvas_width,
                canvas_height,
                warnings,
            );
            sub_regions.push(sub_pixels);
        }
        pixels = intersect(&sub_regions);
    }

    // Handle 'except' modifier (subtract named regions)
    if let Some(except_tokens) = &region.except {
        let mut except_sets = Vec::new();
        for token_name in except_tokens {
            if let Some(except_pixels) = all_regions.get(token_name) {
                except_sets.push(except_pixels.clone());
            } else {
                warnings.push(Warning::new(format!(
                    "Unknown token '{}' in except clause",
                    token_name
                )));
            }
        }
        if !except_sets.is_empty() {
            pixels = subtract(&pixels, &except_sets);
        }
    }

    // Handle symmetric modifier
    if let Some(symmetric) = &region.symmetric {
        pixels = apply_symmetric(&pixels, symmetric, canvas_width, canvas_height, warnings);
    }

    pixels
}

/// Parse a fill reference like "inside(token_name)" to extract the token name.
fn parse_fill_reference(fill_ref: &str) -> Option<&str> {
    let trimmed = fill_ref.trim();
    if trimmed.starts_with("inside(") && trimmed.ends_with(')') {
        Some(&trimmed[7..trimmed.len() - 1])
    } else {
        None
    }
}

/// Apply symmetric mirroring to a set of pixels.
fn apply_symmetric(
    pixels: &HashSet<(i32, i32)>,
    symmetric: &str,
    canvas_width: i32,
    canvas_height: i32,
    _warnings: &mut Vec<Warning>,
) -> HashSet<(i32, i32)> {
    let mut result = pixels.clone();

    match symmetric {
        "x" => {
            // Mirror across vertical center line
            let center_x = canvas_width / 2;
            for (x, y) in pixels {
                let mirrored_x = 2 * center_x - x - 1;
                if mirrored_x >= 0 && mirrored_x < canvas_width {
                    result.insert((mirrored_x, *y));
                }
            }
        }
        "y" => {
            // Mirror across horizontal center line
            let center_y = canvas_height / 2;
            for (x, y) in pixels {
                let mirrored_y = 2 * center_y - y - 1;
                if mirrored_y >= 0 && mirrored_y < canvas_height {
                    result.insert((*x, mirrored_y));
                }
            }
        }
        "xy" | "yx" => {
            // Mirror across both axes
            let center_x = canvas_width / 2;
            let center_y = canvas_height / 2;
            for (x, y) in pixels {
                let mirrored_x = 2 * center_x - x - 1;
                let mirrored_y = 2 * center_y - y - 1;

                // Add x-mirrored
                if mirrored_x >= 0 && mirrored_x < canvas_width {
                    result.insert((mirrored_x, *y));
                }
                // Add y-mirrored
                if mirrored_y >= 0 && mirrored_y < canvas_height {
                    result.insert((*x, mirrored_y));
                }
                // Add both-mirrored
                if mirrored_x >= 0
                    && mirrored_x < canvas_width
                    && mirrored_y >= 0
                    && mirrored_y < canvas_height
                {
                    result.insert((mirrored_x, mirrored_y));
                }
            }
        }
        _ => {
            // Could be a specific coordinate, but we'll skip that for now
        }
    }

    result
}

/// Render a structured sprite (regions format) to an RGBA image buffer.
///
/// # Arguments
///
/// * `name` - Sprite name (for error messages)
/// * `size` - Sprite size [width, height]
/// * `regions` - Map of token names to region definitions
/// * `palette` - Map of token names to hex color strings
///
/// # Returns
///
/// The rendered image and any warnings generated.
pub fn render_structured(
    name: &str,
    size: Option<[u32; 2]>,
    regions: &HashMap<String, RegionDef>,
    palette: &HashMap<String, String>,
) -> (RgbaImage, Vec<Warning>) {
    let mut warnings = Vec::new();

    // Determine canvas size
    let (width, height) = if let Some([w, h]) = size {
        (w as i32, h as i32)
    } else {
        warnings.push(Warning::new(format!(
            "Structured sprite '{}' requires explicit size",
            name
        )));
        return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
    };

    if width <= 0 || height <= 0 {
        warnings.push(Warning::new(format!(
            "Invalid size for sprite '{}': {}x{}",
            name, width, height
        )));
        return (RgbaImage::from_pixel(1, 1, TRANSPARENT), warnings);
    }

    // Build color lookup with parsed RGBA values
    let mut color_cache: HashMap<String, Rgba<u8>> = HashMap::new();
    for (token, hex_color) in palette {
        match parse_color(hex_color) {
            Ok(rgba) => {
                color_cache.insert(token.clone(), rgba);
            }
            Err(e) => {
                warnings.push(Warning::new(format!(
                    "Invalid color '{}' for token {}: {}, using magenta",
                    hex_color, token, e
                )));
                color_cache.insert(token.clone(), MAGENTA);
            }
        }
    }

    // Rasterize all regions first (needed for fill references)
    let mut rasterized_regions: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();

    // We need to rasterize in dependency order. For now, we'll do a simple two-pass:
    // 1. Rasterize regions without fill/auto-shadow references
    // 2. Rasterize regions with fill/auto-shadow references
    let mut pending_regions: Vec<(String, RegionDef)> = Vec::new();

    for (token, region) in regions {
        if region.fill.is_some() || region.auto_shadow.is_some() {
            // Defer regions with fill or auto-shadow references
            pending_regions.push((token.clone(), region.clone()));
        } else {
            let pixels = rasterize_region(region, &rasterized_regions, width, height, &mut warnings);
            rasterized_regions.insert(token.clone(), pixels);
        }
    }

    // Now process regions with fill/auto-shadow references
    for (token, region) in pending_regions {
        let pixels = rasterize_region(&region, &rasterized_regions, width, height, &mut warnings);
        rasterized_regions.insert(token, pixels);
    }

    // Create image
    let mut image = RgbaImage::new(width as u32, height as u32);

    // Collect regions with their z-order for sorting
    // Uses explicit z if provided, otherwise infers from semantic role
    let mut region_order: Vec<(String, i32)> = regions
        .iter()
        .map(|(token, region)| {
            let z = region.z.unwrap_or_else(|| default_z_for_role(region.role.as_ref()));
            (token.clone(), z)
        })
        .collect();

    // Sort by z-order (lowest to highest)
    region_order.sort_by_key(|(_, z)| *z);

    // Render regions in z-order
    for (token, _z) in region_order {
        if let Some(pixels) = rasterized_regions.get(&token) {
            let color = if let Some(&rgba) = color_cache.get(&token) {
                rgba
            } else {
                warnings.push(Warning::new(format!(
                    "Unknown token {} in sprite '{}'",
                    token, name
                )));
                color_cache.insert(token.clone(), MAGENTA);
                MAGENTA
            };

            // Render all pixels for this region
            for (x, y) in pixels {
                if *x >= 0 && *x < width && *y >= 0 && *y < height {
                    image.put_pixel(*x as u32, *y as u32, color);
                }
            }
        }
    }

    (image, warnings)
}

/// Extract bounding boxes for all anchor-role regions in a structured sprite.
///
/// This is used for anchor preservation during scaling operations.
/// Regions with `role: "anchor"` are identified and their pixel coordinates
/// are converted to bounding boxes.
///
/// # Arguments
///
/// * `regions` - Map of token names to region definitions
/// * `canvas_width` - Width of the sprite canvas
/// * `canvas_height` - Height of the sprite canvas
///
/// # Returns
///
/// A vector of `AnchorBounds` representing the bounding box of each anchor region.
pub fn extract_anchor_bounds(
    regions: &HashMap<String, RegionDef>,
    canvas_width: i32,
    canvas_height: i32,
) -> Vec<crate::transforms::AnchorBounds> {
    let mut anchor_bounds = Vec::new();
    let mut warnings = Vec::new();

    // First pass: rasterize non-fill regions (needed for fill references)
    let mut rasterized_regions: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();
    let mut pending_regions: Vec<(String, RegionDef)> = Vec::new();

    for (token, region) in regions {
        if region.fill.is_some() || region.auto_shadow.is_some() {
            pending_regions.push((token.clone(), region.clone()));
        } else {
            let pixels =
                rasterize_region(region, &rasterized_regions, canvas_width, canvas_height, &mut warnings);
            rasterized_regions.insert(token.clone(), pixels);
        }
    }

    // Second pass: rasterize fill/auto-shadow regions
    for (token, region) in pending_regions {
        let pixels =
            rasterize_region(&region, &rasterized_regions, canvas_width, canvas_height, &mut warnings);
        rasterized_regions.insert(token, pixels);
    }

    // Now extract bounding boxes for anchor regions
    for (token, region) in regions {
        if region.role == Some(Role::Anchor) {
            if let Some(pixels) = rasterized_regions.get(token) {
                let points: Vec<(i32, i32)> = pixels.iter().copied().collect();
                if let Some(bounds) = crate::transforms::AnchorBounds::from_points(&points) {
                    anchor_bounds.push(bounds);
                }
            }
        }
    }

    anchor_bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_line;
    use crate::registry::{PaletteRegistry, SpriteRegistry};

    #[test]
    fn test_parse_fill_reference() {
        assert_eq!(parse_fill_reference("inside(outline)"), Some("outline"));
        assert_eq!(parse_fill_reference("inside(o)"), Some("o"));
        assert_eq!(parse_fill_reference(" inside(token) "), Some("token"));
        assert_eq!(parse_fill_reference("invalid"), None);
        assert_eq!(parse_fill_reference("inside("), None);
    }

    #[test]
    fn test_rasterize_region_rect() {
        let region = RegionDef {
            rect: Some([0, 0, 3, 2]),
            ..Default::default()
        };
        let all_regions = HashMap::new();
        let mut warnings = Vec::new();

        let pixels = rasterize_region(&region, &all_regions, 10, 10, &mut warnings);

        assert_eq!(pixels.len(), 6);
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(2, 1)));
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_rasterize_region_stroke() {
        let region = RegionDef {
            stroke: Some([0, 0, 4, 4]),
            thickness: Some(1),
            ..Default::default()
        };
        let all_regions = HashMap::new();
        let mut warnings = Vec::new();

        let pixels = rasterize_region(&region, &all_regions, 10, 10, &mut warnings);

        // Should have border pixels but not interior
        assert!(pixels.contains(&(0, 0)));
        assert!(pixels.contains(&(3, 0)));
        assert!(!pixels.contains(&(1, 1))); // Interior
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_rasterize_region_fill() {
        // Create an outline region first
        let mut all_regions = HashMap::new();
        let outline_region = RegionDef {
            stroke: Some([0, 0, 5, 5]),
            thickness: Some(1),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let outline_pixels = rasterize_region(&outline_region, &all_regions, 10, 10, &mut warnings);
        all_regions.insert("outline".to_string(), outline_pixels);

        // Now create a fill region
        let fill_region = RegionDef {
            fill: Some("inside(outline)".to_string()),
            ..Default::default()
        };

        let pixels = rasterize_region(&fill_region, &all_regions, 10, 10, &mut warnings);

        // Should fill the interior
        assert!(pixels.contains(&(2, 2)));
        assert!(!pixels.contains(&(0, 0))); // Border
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_render_structured_basic() {
        let mut regions = HashMap::new();
        regions.insert(
            "o".to_string(),
            RegionDef {
                stroke: Some([0, 0, 8, 8]),
                thickness: Some(1),
                ..Default::default()
            },
        );
        regions.insert(
            "f".to_string(),
            RegionDef {
                fill: Some("inside(o)".to_string()),
                z: Some(1),
                ..Default::default()
            },
        );

        let mut palette = HashMap::new();
        palette.insert("o".to_string(), "#000000".to_string());
        palette.insert("f".to_string(), "#FF0000".to_string());

        let (image, warnings) = render_structured("test", Some([8, 8]), &regions, &palette);

        assert_eq!(image.width(), 8);
        assert_eq!(image.height(), 8);
        assert!(warnings.is_empty());

        // Check border is black
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 255]));
        assert_eq!(*image.get_pixel(7, 0), Rgba([0, 0, 0, 255]));

        // Check interior is red
        assert_eq!(*image.get_pixel(2, 2), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(5, 5), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_verification_example_from_spec() {
        // This is the verification example from the task specification
        // Create palette
        let palette_line = "{type: \"palette\", name: \"p\", colors: {_: \"#0000\", o: \"#000\", f: \"#FF0000\"}}";
        let palette_obj = parse_line(palette_line, 0).unwrap();

        // Create sprite with regions
        let sprite_line = "{type: \"sprite\", name: \"s\", size: [8, 8], palette: \"p\", regions: { o: { stroke: [0, 0, 8, 8] }, f: { fill: \"inside(o)\" } }}";
        let sprite_obj = parse_line(sprite_line, 0).unwrap();

        // Build registries
        let mut palette_registry = PaletteRegistry::new();
        let mut sprite_registry = SpriteRegistry::new();

        // Register objects
        match palette_obj {
            crate::models::TtpObject::Palette(p) => palette_registry.register(p),
            _ => panic!("Expected palette"),
        }

        match sprite_obj {
            crate::models::TtpObject::Sprite(s) => sprite_registry.register_sprite(s),
            _ => panic!("Expected sprite"),
        }

        // Resolve and render
        let resolved = sprite_registry
            .resolve("s", &palette_registry, false)
            .expect("Failed to resolve sprite");
        let (image, warnings) = crate::renderer::render_resolved(&resolved);

        assert!(warnings.is_empty(), "Unexpected warnings: {:?}", warnings);
        assert_eq!(image.width(), 8);
        assert_eq!(image.height(), 8);

        // Verify it's a red square with black outline
        // Corners should be black (outline)
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 0, 255]));
        assert_eq!(*image.get_pixel(7, 0), Rgba([0, 0, 0, 255]));
        assert_eq!(*image.get_pixel(0, 7), Rgba([0, 0, 0, 255]));
        assert_eq!(*image.get_pixel(7, 7), Rgba([0, 0, 0, 255]));

        // Edges should be black
        assert_eq!(*image.get_pixel(3, 0), Rgba([0, 0, 0, 255]));
        assert_eq!(*image.get_pixel(0, 3), Rgba([0, 0, 0, 255]));

        // Interior should be red (#F00 = #FF0000)
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(4, 4), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(6, 6), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn test_rasterize_auto_shadow() {
        // Create a body region first
        let mut all_regions = HashMap::new();
        let body_region = RegionDef {
            rect: Some([2, 2, 4, 4]),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let body_pixels = rasterize_region(&body_region, &all_regions, 10, 10, &mut warnings);
        all_regions.insert("body".to_string(), body_pixels);

        // Now create an auto-shadow region with offset
        let shadow_region = RegionDef {
            auto_shadow: Some("body".to_string()),
            offset: Some([2, 2]),
            ..Default::default()
        };

        let pixels = rasterize_region(&shadow_region, &all_regions, 10, 10, &mut warnings);

        // Shadow should be offset by [2, 2] from body
        // Body is at (2,2) to (5,5), so shadow should be at (4,4) to (7,7)
        assert!(pixels.contains(&(4, 4)));
        assert!(pixels.contains(&(7, 7)));
        assert!(!pixels.contains(&(2, 2))); // Body position, not shadow
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_render_structured_with_auto_shadow() {
        let mut regions = HashMap::new();
        regions.insert(
            "body".to_string(),
            RegionDef {
                rect: Some([1, 1, 4, 4]),
                z: Some(1),
                ..Default::default()
            },
        );
        regions.insert(
            "shadow".to_string(),
            RegionDef {
                auto_shadow: Some("body".to_string()),
                offset: Some([1, 1]),
                z: Some(0), // Shadow behind body
                ..Default::default()
            },
        );

        let mut palette = HashMap::new();
        palette.insert("body".to_string(), "#FF0000".to_string());
        palette.insert("shadow".to_string(), "#000000".to_string());

        let (image, warnings) = render_structured("test", Some([8, 8]), &regions, &palette);

        assert_eq!(image.width(), 8);
        assert_eq!(image.height(), 8);
        assert!(warnings.is_empty(), "Unexpected warnings: {:?}", warnings);

        // Body at (1,1)-(4,4) should be red
        assert_eq!(*image.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*image.get_pixel(4, 4), Rgba([255, 0, 0, 255]));

        // Shadow visible at offset (5,5) where body doesn't overlap
        assert_eq!(*image.get_pixel(5, 5), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn test_default_z_for_role() {
        // Anchor should be highest
        assert_eq!(default_z_for_role(Some(&Role::Anchor)), 100);
        // Boundary is high
        assert_eq!(default_z_for_role(Some(&Role::Boundary)), 80);
        // Shadow and highlight are medium
        assert_eq!(default_z_for_role(Some(&Role::Shadow)), 60);
        assert_eq!(default_z_for_role(Some(&Role::Highlight)), 60);
        // Fill is low
        assert_eq!(default_z_for_role(Some(&Role::Fill)), 40);
        // No role defaults to 0
        assert_eq!(default_z_for_role(None), 0);
    }

    #[test]
    fn test_role_based_z_ordering() {
        // Create overlapping regions with different roles but no explicit z
        // The anchor (eye) should render on top of fill (skin) despite being added first
        let mut regions = HashMap::new();

        // Fill region at position (2,2)-(5,5) - should be at bottom
        regions.insert(
            "skin".to_string(),
            RegionDef {
                rect: Some([2, 2, 4, 4]),
                role: Some(Role::Fill),
                ..Default::default()
            },
        );

        // Anchor region at same position - should be on top
        regions.insert(
            "eye".to_string(),
            RegionDef {
                rect: Some([3, 3, 2, 2]),
                role: Some(Role::Anchor),
                ..Default::default()
            },
        );

        let mut palette = HashMap::new();
        palette.insert("skin".to_string(), "#FFCC99".to_string());
        palette.insert("eye".to_string(), "#000000".to_string());

        let (image, warnings) = render_structured("test", Some([8, 8]), &regions, &palette);

        assert!(warnings.is_empty(), "Unexpected warnings: {:?}", warnings);

        // Eye at (3,3) should be visible (black) because anchor > fill
        assert_eq!(*image.get_pixel(3, 3), Rgba([0, 0, 0, 255]));
        // Skin at (2,2) outside eye should still be visible
        assert_eq!(*image.get_pixel(2, 2), Rgba([255, 204, 153, 255]));
    }

    #[test]
    fn test_explicit_z_overrides_role() {
        // Explicit z should override role-based z
        let mut regions = HashMap::new();

        // Anchor role but explicit low z - should be behind
        regions.insert(
            "anchor_low".to_string(),
            RegionDef {
                rect: Some([2, 2, 4, 4]),
                role: Some(Role::Anchor),
                z: Some(-10), // Explicit low z overrides anchor default of 100
                ..Default::default()
            },
        );

        // Fill role but explicit high z - should be on top
        regions.insert(
            "fill_high".to_string(),
            RegionDef {
                rect: Some([3, 3, 2, 2]),
                role: Some(Role::Fill),
                z: Some(200), // Explicit high z overrides fill default of 40
                ..Default::default()
            },
        );

        let mut palette = HashMap::new();
        palette.insert("anchor_low".to_string(), "#FF0000".to_string());
        palette.insert("fill_high".to_string(), "#00FF00".to_string());

        let (image, warnings) = render_structured("test", Some([8, 8]), &regions, &palette);

        assert!(warnings.is_empty(), "Unexpected warnings: {:?}", warnings);

        // Green fill_high should be on top despite Fill role
        assert_eq!(*image.get_pixel(3, 3), Rgba([0, 255, 0, 255]));
        // Red anchor visible where not overlapped
        assert_eq!(*image.get_pixel(2, 2), Rgba([255, 0, 0, 255]));
    }
}
