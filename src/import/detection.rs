//! Detection algorithms for import analysis.
//!
//! Includes detection of dither patterns, upscaling, and outlines.

use std::collections::{HashMap, HashSet};

use super::{DitherInfo, DitherPattern, OutlineInfo, UpscaleInfo};
use crate::models::RelationshipType;

/// Infer z-order values from spatial containment relationships.
///
/// If region A is contained within region B, A should be rendered on top (higher z).
/// This computes z-levels by finding how deeply nested each region is.
///
/// Algorithm:
/// 1. Build a containment graph from ContainedWithin relationships
/// 2. For each region, z = 1 + max(z of all containers)
/// 3. Regions not contained get z = 0
pub(crate) fn infer_z_order(
    tokens: &[String],
    relationships: &[(String, RelationshipType, String)],
) -> HashMap<String, i32> {
    // Build containment graph: token -> set of tokens it's contained in
    let mut contained_in: HashMap<String, Vec<String>> = HashMap::new();

    for (source, rel_type, target) in relationships {
        if matches!(rel_type, RelationshipType::ContainedWithin) {
            // source is contained within target
            contained_in
                .entry(source.clone())
                .or_default()
                .push(target.clone());
        }
    }

    // Compute z-order using memoization
    let mut z_order: HashMap<String, i32> = HashMap::new();
    let mut computing: HashSet<String> = HashSet::new(); // Cycle detection

    fn compute_z(
        token: &str,
        contained_in: &HashMap<String, Vec<String>>,
        z_order: &mut HashMap<String, i32>,
        computing: &mut HashSet<String>,
    ) -> i32 {
        // Already computed
        if let Some(&z) = z_order.get(token) {
            return z;
        }

        // Cycle detection - return 0 if we're in a cycle
        if computing.contains(token) {
            return 0;
        }
        computing.insert(token.to_string());

        // Get containers
        let z = if let Some(containers) = contained_in.get(token) {
            // z = 1 + max(z of containers)
            let max_container_z = containers
                .iter()
                .map(|c| compute_z(c, contained_in, z_order, computing))
                .max()
                .unwrap_or(0);
            max_container_z + 1
        } else {
            // Not contained in anything - base level
            0
        };

        computing.remove(token);
        z_order.insert(token.to_string(), z);
        z
    }

    // Compute z for all tokens
    for token in tokens {
        compute_z(token, &contained_in, &mut z_order, &mut computing);
    }

    z_order
}

/// Detect dither patterns in the image.
///
/// Looks for common dithering patterns:
/// - Checkerboard: alternating colors in a 2x2 grid pattern
/// - Horizontal lines: alternating colors in horizontal stripes
/// - Vertical lines: alternating colors in vertical stripes
/// - Ordered dither: Bayer matrix patterns (2x2, 4x4)
pub(crate) fn detect_dither_patterns(
    width: u32,
    height: u32,
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    token_to_color: &HashMap<String, [u8; 4]>,
) -> Vec<DitherInfo> {
    let mut dither_patterns = Vec::new();

    // Build a grid for quick token lookup
    let mut pixel_to_token: HashMap<(i32, i32), String> = HashMap::new();
    for (token, pixels) in token_pixels {
        for &(x, y) in pixels {
            pixel_to_token.insert((x, y), token.clone());
        }
    }

    // Get all token pairs to check for dithering
    let tokens: Vec<String> = token_pixels.keys().cloned().collect();

    for i in 0..tokens.len() {
        for j in (i + 1)..tokens.len() {
            let token1 = &tokens[i];
            let token2 = &tokens[j];

            // Skip transparent token
            if token1 == "{_}" || token2 == "{_}" {
                continue;
            }

            // Get pixels for both tokens
            let pixels1 = &token_pixels[token1];
            let pixels2 = &token_pixels[token2];

            // Check for checkerboard pattern between these two tokens
            if let Some(info) = detect_checkerboard_pattern(
                token1,
                token2,
                pixels1,
                pixels2,
                &pixel_to_token,
                token_to_color,
                width,
                height,
            ) {
                dither_patterns.push(info);
            }

            // Check for horizontal line pattern
            if let Some(info) = detect_line_pattern(
                token1,
                token2,
                pixels1,
                pixels2,
                &pixel_to_token,
                token_to_color,
                width,
                height,
                true, // horizontal
            ) {
                dither_patterns.push(info);
            }

            // Check for vertical line pattern
            if let Some(info) = detect_line_pattern(
                token1,
                token2,
                pixels1,
                pixels2,
                &pixel_to_token,
                token_to_color,
                width,
                height,
                false, // vertical
            ) {
                dither_patterns.push(info);
            }
        }
    }

    dither_patterns
}

/// Check if two tokens form a checkerboard dither pattern.
fn detect_checkerboard_pattern(
    token1: &str,
    token2: &str,
    pixels1: &HashSet<(i32, i32)>,
    pixels2: &HashSet<(i32, i32)>,
    pixel_to_token: &HashMap<(i32, i32), String>,
    token_to_color: &HashMap<String, [u8; 4]>,
    _width: u32,
    _height: u32,
) -> Option<DitherInfo> {
    // Find overlapping bounding box
    let min_x1 = pixels1.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x1 = pixels1.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y1 = pixels1.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y1 = pixels1.iter().map(|(_, y)| *y).max().unwrap_or(0);

    let min_x2 = pixels2.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x2 = pixels2.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y2 = pixels2.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y2 = pixels2.iter().map(|(_, y)| *y).max().unwrap_or(0);

    // Find the intersection of bounding boxes
    let min_x = min_x1.max(min_x2);
    let max_x = max_x1.min(max_x2);
    let min_y = min_y1.max(min_y2);
    let max_y = max_y1.min(max_y2);

    // Must have overlap
    if max_x < min_x || max_y < min_y {
        return None;
    }

    // Count checkerboard matches
    let mut checkerboard_matches = 0;
    let mut total_cells = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Expected token based on checkerboard pattern
            let is_even_cell = (x + y) % 2 == 0;
            let expected_token = if is_even_cell { token1 } else { token2 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                total_cells += 1;
                if actual_token == expected_token {
                    checkerboard_matches += 1;
                }
            }
        }
    }

    // Also try the inverse pattern
    let mut inverse_matches = 0;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let is_even_cell = (x + y) % 2 == 0;
            let expected_token = if is_even_cell { token2 } else { token1 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                if actual_token == expected_token {
                    inverse_matches += 1;
                }
            }
        }
    }

    let best_matches = checkerboard_matches.max(inverse_matches);

    // Minimum coverage and match ratio for detection
    let coverage = total_cells as f64 / ((max_x - min_x + 1) * (max_y - min_y + 1)) as f64;
    let match_ratio = if total_cells > 0 {
        best_matches as f64 / total_cells as f64
    } else {
        0.0
    };

    // Need at least 4 cells (2x2 minimum) and 80% match ratio
    if total_cells >= 4 && match_ratio >= 0.8 && coverage >= 0.7 {
        // Compute merged color
        let color1 = token_to_color.get(token1).copied().unwrap_or([0, 0, 0, 255]);
        let color2 = token_to_color.get(token2).copied().unwrap_or([0, 0, 0, 255]);
        let merged = average_colors(&[color1, color2]);

        Some(DitherInfo {
            tokens: vec![token1.to_string(), token2.to_string()],
            pattern: DitherPattern::Checkerboard,
            bounds: [min_x as u32, min_y as u32, (max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32],
            merged_color: format!("#{:02X}{:02X}{:02X}", merged[0], merged[1], merged[2]),
            confidence: match_ratio * coverage,
        })
    } else {
        None
    }
}

/// Check if two tokens form a horizontal or vertical line dither pattern.
fn detect_line_pattern(
    token1: &str,
    token2: &str,
    pixels1: &HashSet<(i32, i32)>,
    pixels2: &HashSet<(i32, i32)>,
    pixel_to_token: &HashMap<(i32, i32), String>,
    token_to_color: &HashMap<String, [u8; 4]>,
    _width: u32,
    _height: u32,
    horizontal: bool,
) -> Option<DitherInfo> {
    // Find overlapping bounding box
    let min_x1 = pixels1.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x1 = pixels1.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y1 = pixels1.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y1 = pixels1.iter().map(|(_, y)| *y).max().unwrap_or(0);

    let min_x2 = pixels2.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x2 = pixels2.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y2 = pixels2.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y2 = pixels2.iter().map(|(_, y)| *y).max().unwrap_or(0);

    let min_x = min_x1.max(min_x2);
    let max_x = max_x1.min(max_x2);
    let min_y = min_y1.max(min_y2);
    let max_y = max_y1.min(max_y2);

    if max_x < min_x || max_y < min_y {
        return None;
    }

    // Count line pattern matches
    let mut line_matches = 0;
    let mut total_cells = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            // Expected token based on line pattern
            let line_idx = if horizontal { y } else { x };
            let is_even_line = line_idx % 2 == 0;
            let expected_token = if is_even_line { token1 } else { token2 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                total_cells += 1;
                if actual_token == expected_token {
                    line_matches += 1;
                }
            }
        }
    }

    // Also try inverse
    let mut inverse_matches = 0;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let line_idx = if horizontal { y } else { x };
            let is_even_line = line_idx % 2 == 0;
            let expected_token = if is_even_line { token2 } else { token1 };

            if let Some(actual_token) = pixel_to_token.get(&(x, y)) {
                if actual_token == expected_token {
                    inverse_matches += 1;
                }
            }
        }
    }

    let best_matches = line_matches.max(inverse_matches);
    let coverage = total_cells as f64 / ((max_x - min_x + 1) * (max_y - min_y + 1)) as f64;
    let match_ratio = if total_cells > 0 {
        best_matches as f64 / total_cells as f64
    } else {
        0.0
    };

    // Need at least 4 cells and 90% match ratio for line patterns (stricter than checkerboard)
    // Also need at least 2 lines in the primary direction
    let line_count = if horizontal { max_y - min_y + 1 } else { max_x - min_x + 1 };

    if total_cells >= 4 && match_ratio >= 0.9 && coverage >= 0.8 && line_count >= 2 {
        let color1 = token_to_color.get(token1).copied().unwrap_or([0, 0, 0, 255]);
        let color2 = token_to_color.get(token2).copied().unwrap_or([0, 0, 0, 255]);
        let merged = average_colors(&[color1, color2]);

        let pattern = if horizontal {
            DitherPattern::HorizontalLines
        } else {
            DitherPattern::VerticalLines
        };

        Some(DitherInfo {
            tokens: vec![token1.to_string(), token2.to_string()],
            pattern,
            bounds: [min_x as u32, min_y as u32, (max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32],
            merged_color: format!("#{:02X}{:02X}{:02X}", merged[0], merged[1], merged[2]),
            confidence: match_ratio * coverage,
        })
    } else {
        None
    }
}

/// Average multiple colors together.
pub(crate) fn average_colors(colors: &[[u8; 4]]) -> [u8; 4] {
    if colors.is_empty() {
        return [0, 0, 0, 255];
    }

    let mut r: u32 = 0;
    let mut g: u32 = 0;
    let mut b: u32 = 0;
    let mut a: u32 = 0;

    for c in colors {
        r += c[0] as u32;
        g += c[1] as u32;
        b += c[2] as u32;
        a += c[3] as u32;
    }

    let n = colors.len() as u32;
    [(r / n) as u8, (g / n) as u8, (b / n) as u8, (a / n) as u8]
}

/// Detect if an image appears to be upscaled pixel art.
///
/// Checks for repeated NxN blocks of identical colors, which indicates
/// the image was upscaled from a lower resolution using nearest-neighbor scaling.
///
/// Returns the detected scale and native resolution if upscaling is found.
pub(crate) fn detect_upscale(
    pixel_data: &[u8],
    width: u32,
    height: u32,
) -> Option<UpscaleInfo> {
    // Check common scale factors: 2x, 3x, 4x, 5x, 6x, 8x
    let scales_to_check = [2, 3, 4, 5, 6, 8];

    for scale in scales_to_check {
        // Skip if dimensions aren't divisible by scale
        if width % scale != 0 || height % scale != 0 {
            continue;
        }

        let native_width = width / scale;
        let native_height = height / scale;

        // Check if all scale x scale blocks are uniform
        let confidence = check_uniform_blocks(pixel_data, width, height, scale);

        // Require high confidence (>95% of blocks are uniform)
        if confidence >= 0.95 {
            return Some(UpscaleInfo {
                scale,
                native_size: [native_width, native_height],
                confidence,
            });
        }
    }

    None
}

/// Check what fraction of scale x scale blocks in the image are uniform.
fn check_uniform_blocks(
    pixel_data: &[u8],
    width: u32,
    height: u32,
    scale: u32,
) -> f64 {
    let native_width = width / scale;
    let native_height = height / scale;
    let mut uniform_blocks = 0u64;
    let total_blocks = (native_width * native_height) as u64;

    for block_y in 0..native_height {
        for block_x in 0..native_width {
            // Get the color at the top-left of this block
            let base_x = block_x * scale;
            let base_y = block_y * scale;
            let base_idx = ((base_y * width + base_x) * 4) as usize;

            if base_idx + 3 >= pixel_data.len() {
                continue;
            }

            let base_color = [
                pixel_data[base_idx],
                pixel_data[base_idx + 1],
                pixel_data[base_idx + 2],
                pixel_data[base_idx + 3],
            ];

            // Check if all pixels in this block match
            let mut is_uniform = true;
            'block_check: for dy in 0..scale {
                for dx in 0..scale {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    let idx = ((py * width + px) * 4) as usize;

                    if idx + 3 >= pixel_data.len() {
                        is_uniform = false;
                        break 'block_check;
                    }

                    let color = [
                        pixel_data[idx],
                        pixel_data[idx + 1],
                        pixel_data[idx + 2],
                        pixel_data[idx + 3],
                    ];

                    if color != base_color {
                        is_uniform = false;
                        break 'block_check;
                    }
                }
            }

            if is_uniform {
                uniform_blocks += 1;
            }
        }
    }

    if total_blocks == 0 {
        return 0.0;
    }

    uniform_blocks as f64 / total_blocks as f64
}

/// Detect thin dark regions that appear to be outlines/strokes.
///
/// Looks for regions that are:
/// - Thin (1-3 pixels wide on average)
/// - Dark colored (low luminosity)
/// - Adjacent to other regions (bordering them)
pub(crate) fn detect_outlines(
    token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
    token_to_color: &HashMap<String, [u8; 4]>,
    _width: u32,
    _height: u32,
) -> Vec<OutlineInfo> {
    let mut outlines = Vec::new();

    for (token, pixels) in token_pixels {
        // Skip transparent token
        if token == "{_}" || pixels.is_empty() {
            continue;
        }

        // Check if color is dark
        let color = token_to_color.get(token).copied().unwrap_or([0, 0, 0, 255]);
        let luminosity = (color[0] as f64 * 0.299 + color[1] as f64 * 0.587 + color[2] as f64 * 0.114) / 255.0;

        // Skip if not dark enough (luminosity > 0.3 means not dark)
        if luminosity > 0.3 {
            continue;
        }

        // Calculate average width using distance transform approximation
        let avg_width = calculate_average_width(pixels);

        // Skip if not thin (1-3px average width)
        if avg_width < 0.8 || avg_width > 3.5 {
            continue;
        }

        // Find what regions this borders
        let borders = find_bordered_regions(token, pixels, token_pixels);

        // Skip if it doesn't border anything substantial
        if borders.is_empty() {
            continue;
        }

        // Calculate confidence based on width consistency and border coverage
        let width_score = if avg_width >= 1.0 && avg_width <= 2.0 {
            1.0
        } else if avg_width <= 3.0 {
            0.8
        } else {
            0.6
        };

        let border_score = (borders.len() as f64 / 5.0).min(1.0);
        let confidence = (width_score * 0.6 + border_score * 0.4) * (1.0 - luminosity);

        if confidence >= 0.3 {
            outlines.push(OutlineInfo {
                token: token.clone(),
                borders,
                width: avg_width,
                confidence,
            });
        }
    }

    outlines
}

/// Calculate the average width of a region using morphological thinning approximation.
fn calculate_average_width(pixels: &HashSet<(i32, i32)>) -> f64 {
    if pixels.is_empty() {
        return 0.0;
    }

    // Approximate average width by looking at the ratio of perimeter to area
    // A thin line has high perimeter to area ratio

    let area = pixels.len() as f64;

    // Count perimeter pixels (pixels with at least one non-region neighbor)
    let mut perimeter = 0;
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    for &(x, y) in pixels {
        for (dx, dy) in directions {
            let nx = x + dx;
            let ny = y + dy;
            if !pixels.contains(&(nx, ny)) {
                perimeter += 1;
                break;
            }
        }
    }

    if perimeter == 0 {
        return area.sqrt(); // Rough estimate for solid regions
    }

    // For a thin line: area ≈ length * width, perimeter ≈ 2 * length + 2 * width
    // For very thin lines (width=1): perimeter ≈ 2 * length, so width ≈ area / (perimeter / 2)
    // This gives us an approximation of average width
    let estimated_length = perimeter as f64 / 2.0;
    if estimated_length > 0.0 {
        area / estimated_length
    } else {
        1.0
    }
}

/// Find regions that are adjacent to (bordered by) the given region.
fn find_bordered_regions(
    outline_token: &str,
    outline_pixels: &HashSet<(i32, i32)>,
    all_token_pixels: &HashMap<String, HashSet<(i32, i32)>>,
) -> Vec<String> {
    let mut bordered = HashSet::new();
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1)];

    // Build reverse lookup for efficiency
    let mut pixel_to_token: HashMap<(i32, i32), &str> = HashMap::new();
    for (token, pixels) in all_token_pixels {
        for &(x, y) in pixels {
            pixel_to_token.insert((x, y), token.as_str());
        }
    }

    // Check each outline pixel for adjacent non-outline regions
    for &(x, y) in outline_pixels {
        for (dx, dy) in directions {
            let nx = x + dx;
            let ny = y + dy;

            if let Some(&adjacent_token) = pixel_to_token.get(&(nx, ny)) {
                if adjacent_token != outline_token && adjacent_token != "{_}" {
                    bordered.insert(adjacent_token.to_string());
                }
            }
        }
    }

    bordered.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_z_order_no_containment() {
        // No containment relationships - all tokens get z = 0
        let tokens = vec!["{a}".to_string(), "{b}".to_string(), "{c}".to_string()];
        let relationships = vec![];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{a}"), Some(&0));
        assert_eq!(z_order.get("{b}"), Some(&0));
        assert_eq!(z_order.get("{c}"), Some(&0));
    }

    #[test]
    fn test_infer_z_order_single_containment() {
        // {inner} is contained within {outer}
        // {inner} should have z = 1, {outer} should have z = 0
        let tokens = vec!["{inner}".to_string(), "{outer}".to_string()];
        let relationships = vec![(
            "{inner}".to_string(),
            RelationshipType::ContainedWithin,
            "{outer}".to_string(),
        )];

        let z_order = infer_z_order(&tokens, &relationships);

        assert_eq!(z_order.get("{inner}"), Some(&1));
        assert_eq!(z_order.get("{outer}"), Some(&0));
    }
}
