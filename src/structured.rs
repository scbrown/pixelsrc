//! Structured sprite rendering (Format v2 with regions)
//!
//! This module provides functionality to render sprites defined using
//! the regions format, which uses geometric shapes and compound operations
//! instead of a character grid.

use crate::color::parse_color;
use crate::models::RegionDef;
use crate::shapes::{intersect, rasterize_ellipse, rasterize_line, rasterize_points, rasterize_polygon, rasterize_rect, rasterize_stroke, subtract, union};
use crate::renderer::Warning;
use image::{Rgba, RgbaImage};
use std::collections::{HashMap, HashSet};

/// Render a structured sprite (regions format) to an RGBA image buffer.
///
/// Takes a sprite's regions definition and a resolved palette (token -> hex color string).
/// Returns the rendered image and any warnings generated.
///
/// # Region Rendering
///
/// Regions are rendered in z-order (lowest to highest z value, defaulting to definition order).
/// Each region's pixels are determined by:
/// 1. Shape primitives (rect, stroke, ellipse, circle, points, line, polygon, path, fill)
/// 2. Compound operations (union, subtract, intersect)
/// 3. Modifiers (except, auto_outline, auto_shadow, symmetric, repeat, spacing, etc.)
/// 4. Constraints (within, adjacent_to, x, y)
///
/// # Arguments
///
/// * `regions` - HashMap of token name to region definition
/// * `size` - Sprite dimensions [width, height]
/// * `palette` - Resolved palette mapping tokens to hex color strings
///
/// # Returns
///
/// The rendered image and any warnings generated.
///
/// # Examples
///
/// ```ignore
/// use pixelsrc::structured::render_structured;
/// use std::collections::HashMap;
///
/// let mut regions = HashMap::new();
/// regions.insert("o".to_string(), RegionDef {
///     stroke: Some([0, 0, 8, 8]),
///     ..Default::default()
/// });
/// regions.insert("f".to_string(), RegionDef {
///     fill: Some("inside(o)".to_string()),
///     ..Default::default()
/// });
///
/// let palette = HashMap::from([
///     ("_".to_string(), "#00000000".to_string()),
///     ("o".to_string(), "#000000".to_string()),
///     ("f".to_string(), "#FF0000".to_string()),
/// ]);
///
/// let (image, warnings) = render_structured(&regions, Some([8, 8]), &palette);
/// assert_eq!(image.width(), 8);
/// assert_eq!(image.height(), 8);
/// ```
pub fn render_structured(
    regions: &HashMap<String, RegionDef>,
    size: Option<[u32; 2]>,
    palette: &HashMap<String, String>,
) -> (RgbaImage, Vec<Warning>) {
    let mut warnings = Vec::new();

    if regions.is_empty() {
        warnings.push(Warning::new("No regions defined in sprite"));
        return (RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 0])), warnings);
    }

    // Determine sprite dimensions
    let (width, height) = if let Some([w, h]) = size {
        (w as usize, h as usize)
    } else {
        // Infer from region bounds
        let bounds = calculate_region_bounds(regions, &mut warnings);
        if bounds.0 == 1 && bounds.1 == 1 {
            warnings.push(Warning::new("Cannot infer size from empty regions"));
            return (RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 0])), warnings);
        }
        bounds
    };

    if width == 0 || height == 0 {
        warnings.push(Warning::new("Invalid sprite dimensions (zero width or height)"));
        return (RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 0])), warnings);
    }

    // Parse palette colors
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
                color_cache.insert(token.clone(), Rgba([255, 0, 255, 255]));
            }
        }
    }

    // Build region pixel maps
    let mut region_pixels: HashMap<String, HashSet<(i32, i32)>> = HashMap::new();

    // Render all regions in z-order
    let mut sorted_regions: Vec<(&String, &RegionDef)> = regions.iter().collect();
    sorted_regions.sort_by_key(|(name, region)| (region.z.unwrap_or(0), *name));

    for (name, region) in &sorted_regions {
        let pixels = rasterize_region(region, name, &region_pixels, width, height, &mut warnings);
        region_pixels.insert((**name).clone(), pixels);
    }

    // Create final image by compositing regions in z-order
    let mut image = RgbaImage::new(width as u32, height as u32);

    // Start with transparent background
    for y in 0..height {
        for x in 0..width {
            image.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 0]));
        }
    }

    // Render regions in z-order (lowest to highest)
    for (name, _) in &sorted_regions {
        if let Some(pixels) = region_pixels.get(*name) {
            let color = if let Some(&rgba) = color_cache.get(*name) {
                rgba
            } else {
                warnings.push(Warning::new(format!("Unknown token {} in regions", name)));
                Rgba([255, 0, 255, 255])
            };

        for (x, y) in pixels {
            if *x >= 0 && *x < width as i32 && *y >= 0 && *y < height as i32 {
                image.put_pixel(*x as u32, *y as u32, color);
            }
        }
        }
    }

    (image, warnings)
}

/// Calculate the bounding box of all regions.
///
/// Returns (width, height) of the minimal bounding box that contains all regions.
fn calculate_region_bounds(
    regions: &HashMap<String, RegionDef>,
    warnings: &mut Vec<Warning>,
) -> (usize, usize) {
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for (name, region) in regions {
        let pixels = rasterize_region_simple(region, name, warnings);
        for (x, y) in &pixels {
            min_x = min_x.min(*x);
            max_x = max_x.max(*x);
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }
    }

    if min_x == i32::MAX {
        return (1, 1);
    }

    let width = (max_x - min_x + 1).max(1) as usize;
    let height = (max_y - min_y + 1).max(1) as usize;

    (width, height)
}

/// Rasterize a single region to a set of pixel coordinates.
///
/// Handles shape primitives, compound operations, and modifiers.
fn rasterize_region(
    region: &RegionDef,
    name: &str,
    all_regions: &HashMap<String, HashSet<(i32, i32)>>,
    width: usize,
    height: usize,
    warnings: &mut Vec<Warning>,
) -> HashSet<(i32, i32)> {
    let mut pixels = rasterize_region_simple(region, name, warnings);

    // Apply compound operations
    if let Some(base_region) = &region.base {
        let base_pixels = rasterize_region(base_region, name, all_regions, width, height, warnings);
        if let Some(subtracts) = &region.subtract {
            let subtract_regions: Vec<HashSet<(i32, i32)>> = subtracts
                .iter()
                .map(|r| rasterize_region(r, name, all_regions, width, height, warnings))
                .collect();
            pixels = subtract(&base_pixels, &subtract_regions);
        } else {
            pixels = base_pixels;
        }
    }

    if let Some(union_regions) = &region.union {
        let union_pixels: Vec<HashSet<(i32, i32)>> = union_regions
            .iter()
            .map(|r| rasterize_region(r, name, all_regions, width, height, warnings))
            .collect();
        pixels = union(&union_pixels);
    }

    if let Some(intersect_regions) = &region.intersect {
        let intersect_pixels: Vec<HashSet<(i32, i32)>> = intersect_regions
            .iter()
            .map(|r| rasterize_region(r, name, all_regions, width, height, warnings))
            .collect();
        pixels = intersect(&intersect_pixels);
    }

    // Apply except modifier
    if let Some(except_tokens) = &region.except {
        for token in except_tokens {
            if let Some(except_pixels) = all_regions.get(token) {
                for pixel in except_pixels {
                    pixels.remove(pixel);
                }
            } else {
                warnings.push(Warning::new(format!(
                    "Region '{name}' references unknown token '{token}' in except modifier"
                )));
            }
        }
    }

    // Apply fill operation (flood fill inside boundary)
    if let Some(fill_spec) = &region.fill {
        if fill_spec.starts_with("inside(") && fill_spec.ends_with(')') {
            let token = &fill_spec[7..fill_spec.len() - 1];
            if let Some(boundary_pixels) = all_regions.get(token) {
                pixels = crate::shapes::flood_fill(boundary_pixels, None, width as i32, height as i32);
            } else {
                warnings.push(Warning::new(format!(
                    "Region '{name}' references unknown token '{token}' in fill modifier"
                )));
            }
        }
    }

    // Apply symmetric modifier
    if let Some(symmetric) = &region.symmetric {
        pixels = apply_symmetry(pixels, symmetric, warnings);
    }

    // Apply repeat modifier
    if let Some([count_x, count_y]) = region.repeat {
        let spacing = region.spacing.unwrap_or([0, 0]);
        pixels = apply_repeat(pixels, count_x, count_y, spacing[0], spacing[1], region.offset_alternate.unwrap_or(false));
    }

    // Apply range constraints (x, y)
    if let Some([x_min, x_max]) = region.x {
        pixels.retain(|(x, _)| *x >= x_min as i32 && *x <= x_max as i32);
    }
    if let Some([y_min, y_max]) = region.y {
        pixels.retain(|(_, y)| *y >= y_min as i32 && *y <= y_max as i32);
    }

    pixels
}

/// Rasterize a region's shape primitives without compound operations.
fn rasterize_region_simple(region: &RegionDef, name: &str, warnings: &mut Vec<Warning>) -> HashSet<(i32, i32)> {
    let thickness = region.thickness.unwrap_or(1) as i32;

    if let Some(points) = &region.points {
        return rasterize_points(&points.iter().map(|p| (p[0] as i32, p[1] as i32)).collect::<Vec<_>>());
    }

    if let Some(line_points) = &region.line {
        let mut pixels = HashSet::new();
        for window in line_points.windows(2) {
            let p0 = (window[0][0] as i32, window[0][1] as i32);
            let p1 = (window[1][0] as i32, window[1][1] as i32);
            pixels.extend(rasterize_line(p0, p1));
        }
        return pixels;
    }

    if let Some([x, y, w, h]) = region.rect {
        return rasterize_rect(x as i32, y as i32, w as i32, h as i32);
    }

    if let Some([x, y, w, h]) = region.stroke {
        return rasterize_stroke(x as i32, y as i32, w as i32, h as i32, thickness);
    }

    if let Some([cx, cy, rx, ry]) = region.ellipse {
        return rasterize_ellipse(cx as i32, cy as i32, rx as i32, ry as i32);
    }

    if let Some([cx, cy, r]) = region.circle {
        return rasterize_ellipse(cx as i32, cy as i32, r as i32, r as i32);
    }

    if let Some(polygon) = &region.polygon {
        let points: Vec<(i32, i32)> = polygon.iter().map(|p| (p[0] as i32, p[1] as i32)).collect();
        return rasterize_polygon(&points);
    }

    if let Some(_path) = &region.path {
        warnings.push(Warning::new(format!("Region '{name}' has path but path parsing not yet implemented")));
        return HashSet::new();
    }

    // If no shape primitive, check for compound operations
    if region.union.is_some() || region.base.is_some() || region.subtract.is_some() || region.intersect.is_some() {
        return HashSet::new();
    }

    warnings.push(Warning::new(format!("Region '{name}' has no shape primitive")));
    HashSet::new()
}

/// Apply symmetry transformation to a set of pixels.
fn apply_symmetry(pixels: HashSet<(i32, i32)>, symmetric: &str, _warnings: &mut Vec<Warning>) -> HashSet<(i32, i32)> {
    let mut result = pixels.clone();

    for (x, y) in &pixels {
        match symmetric {
            "x" => {
                // Mirror horizontally (around x=0, so -x)
                result.insert((-x, *y));
            }
            "y" => {
                // Mirror vertically (around y=0, so -y)
                result.insert((*x, -y));
            }
            "xy" => {
                // Mirror both
                result.insert((-x, *y));
                result.insert((*x, -y));
                result.insert((-x, -y));
            }
            _ => {
                // Try parsing as a specific coordinate (e.g., "4" for mirror around x=4)
                if let Ok(coord) = symmetric.parse::<i32>() {
                    result.insert((coord - (x - coord), *y));
                }
            }
        }
    }

    result
}

/// Apply repeat tiling to a set of pixels.
fn apply_repeat(
    pixels: HashSet<(i32, i32)>,
    count_x: u32,
    count_y: u32,
    spacing_x: u32,
    spacing_y: u32,
    offset_alternate: bool,
) -> HashSet<(i32, i32)> {
    let mut result = pixels.clone();

    // Get bounding box to determine tile size
    let (min_x, max_x, min_y, max_y) = pixels
        .iter()
        .fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |(mnx, mxx, mny, mxy), (x, y)| {
            (mnx.min(*x), mxx.max(*x), mny.min(*y), mxy.max(*y))
        });

    let tile_width = (max_x - min_x + 1 + spacing_x as i32).max(1);
    let tile_height = (max_y - min_y + 1 + spacing_y as i32).max(1);

    for iy in 0..count_y {
        for ix in 0..count_x {
            if ix == 0 && iy == 0 {
                continue;
            }

            let offset_x = ix as i32 * tile_width;
            let offset_y = iy as i32 * tile_height;

            let mut row_offset_x = 0;
            if offset_alternate && ix % 2 == 1 {
                row_offset_x = tile_width / 2;
            }

            for (x, y) in &pixels {
                let new_x = x + offset_x + row_offset_x;
                let new_y = y + offset_y;
                result.insert((new_x, new_y));
            }
        }
    }

    result
}
