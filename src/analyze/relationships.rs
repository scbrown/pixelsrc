//! Relationship inference between sprite regions

use std::collections::HashSet;

use crate::models::RelationshipType;

use super::shapes::bounding_box;

/// Result of relationship inference with confidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipInference {
    /// The source token/region name
    pub source: String,
    /// The inferred relationship type
    pub relationship_type: RelationshipType,
    /// The target token/region name
    pub target: String,
    /// Confidence score from 0.0 to 1.0
    pub confidence: f64,
}

impl RelationshipInference {
    /// Create a new relationship inference result.
    pub fn new(
        source: String,
        relationship_type: RelationshipType,
        target: String,
        confidence: f64,
    ) -> Self {
        Self { source, relationship_type, target, confidence: confidence.clamp(0.0, 1.0) }
    }
}

/// HSL color representation for derives-from inference.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Hsl {
    pub(crate) h: f64,
    pub(crate) s: f64,
    pub(crate) l: f64,
}

/// Convert RGB to HSL color space.
pub(crate) fn rgb_to_hsl(r: u8, g: u8, b: u8) -> Hsl {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return Hsl { h: 0.0, s: 0.0, l };
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

    let h = if (max - r).abs() < f64::EPSILON {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < f64::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    Hsl { h: h * 60.0, s, l }
}

/// Infers relationships between regions based on their properties.
pub struct RelationshipInferrer;

impl RelationshipInferrer {
    /// Infer 'derives-from' relationship: colors differ by lightness only.
    pub fn infer_derives_from(
        source_name: &str,
        source_color: [u8; 4],
        target_name: &str,
        target_color: [u8; 4],
    ) -> Option<RelationshipInference> {
        let source_hsl = rgb_to_hsl(source_color[0], source_color[1], source_color[2]);
        let target_hsl = rgb_to_hsl(target_color[0], target_color[1], target_color[2]);

        let hue_diff = {
            let diff = (source_hsl.h - target_hsl.h).abs();
            diff.min(360.0 - diff)
        };

        let sat_diff = (source_hsl.s - target_hsl.s).abs();
        let light_diff = (source_hsl.l - target_hsl.l).abs();

        if hue_diff <= 15.0 && sat_diff <= 0.15 && light_diff >= 0.1 {
            let hue_score = 1.0 - (hue_diff / 15.0);
            let sat_score = 1.0 - (sat_diff / 0.15);
            let light_score = (light_diff - 0.1).min(0.4) / 0.4;

            let confidence = (hue_score * 0.3 + sat_score * 0.3 + light_score * 0.4).min(1.0);

            if confidence >= 0.5 {
                return Some(RelationshipInference::new(
                    source_name.to_string(),
                    RelationshipType::DerivesFrom,
                    target_name.to_string(),
                    confidence,
                ));
            }
        }

        None
    }

    /// Infer 'contained-within' relationship: region pixels fully inside another.
    pub fn infer_contained_within(
        inner_name: &str,
        inner_pixels: &HashSet<(i32, i32)>,
        outer_name: &str,
        outer_pixels: &HashSet<(i32, i32)>,
    ) -> Option<RelationshipInference> {
        if inner_pixels.is_empty() || outer_pixels.is_empty() {
            return None;
        }

        if inner_name == outer_name {
            return None;
        }

        let (inner_min_x, inner_min_y, inner_max_x, inner_max_y) = bounding_box(inner_pixels)?;
        let (outer_min_x, outer_min_y, outer_max_x, outer_max_y) = bounding_box(outer_pixels)?;

        let bbox_contained = inner_min_x >= outer_min_x
            && inner_min_y >= outer_min_y
            && inner_max_x <= outer_max_x
            && inner_max_y <= outer_max_y;

        if !bbox_contained {
            return None;
        }

        let mut surrounded_count = 0;
        for &(x, y) in inner_pixels {
            let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
            if neighbors.iter().any(|n| outer_pixels.contains(n)) {
                surrounded_count += 1;
            }
        }

        let surrounded_ratio = surrounded_count as f64 / inner_pixels.len() as f64;

        if surrounded_ratio >= 0.5 {
            let confidence = (surrounded_ratio * 0.7 + 0.3).min(1.0);
            return Some(RelationshipInference::new(
                inner_name.to_string(),
                RelationshipType::ContainedWithin,
                outer_name.to_string(),
                confidence,
            ));
        }

        None
    }

    /// Infer 'adjacent-to' relationship: regions share boundary pixels.
    pub fn infer_adjacent_to(
        region_a_name: &str,
        region_a_pixels: &HashSet<(i32, i32)>,
        region_b_name: &str,
        region_b_pixels: &HashSet<(i32, i32)>,
    ) -> Option<RelationshipInference> {
        if region_a_pixels.is_empty() || region_b_pixels.is_empty() {
            return None;
        }

        if region_a_name == region_b_name {
            return None;
        }

        let mut boundary_count = 0;
        for &(x, y) in region_a_pixels {
            let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
            if neighbors.iter().any(|n| region_b_pixels.contains(n)) {
                boundary_count += 1;
            }
        }

        if boundary_count == 0 {
            return None;
        }

        let smaller_region_size = region_a_pixels.len().min(region_b_pixels.len());
        let boundary_ratio = boundary_count as f64 / smaller_region_size as f64;

        let confidence = (0.5 + boundary_ratio * 0.5).min(1.0);

        Some(RelationshipInference::new(
            region_a_name.to_string(),
            RelationshipType::AdjacentTo,
            region_b_name.to_string(),
            confidence,
        ))
    }

    /// Infer 'paired-with' relationship: symmetric regions at mirrored positions.
    pub fn infer_paired_with(
        region_a_name: &str,
        region_a_pixels: &HashSet<(i32, i32)>,
        region_b_name: &str,
        region_b_pixels: &HashSet<(i32, i32)>,
        sprite_width: u32,
    ) -> Option<RelationshipInference> {
        if region_a_pixels.is_empty() || region_b_pixels.is_empty() {
            return None;
        }

        if region_a_name == region_b_name {
            return None;
        }

        let size_a = region_a_pixels.len();
        let size_b = region_b_pixels.len();
        let size_ratio = size_a.min(size_b) as f64 / size_a.max(size_b) as f64;

        if size_ratio < 0.8 {
            return None;
        }

        let centroid_a = {
            let sum: (i64, i64) = region_a_pixels
                .iter()
                .fold((0i64, 0i64), |acc, &(x, y)| (acc.0 + x as i64, acc.1 + y as i64));
            (sum.0 as f64 / size_a as f64, sum.1 as f64 / size_a as f64)
        };

        let centroid_b = {
            let sum: (i64, i64) = region_b_pixels
                .iter()
                .fold((0i64, 0i64), |acc, &(x, y)| (acc.0 + x as i64, acc.1 + y as i64));
            (sum.0 as f64 / size_b as f64, sum.1 as f64 / size_b as f64)
        };

        let center_x = sprite_width as f64 / 2.0;
        let expected_mirror_x = 2.0 * center_x - centroid_a.0;

        let x_mirror_diff = (centroid_b.0 - expected_mirror_x).abs();
        let y_diff = (centroid_a.1 - centroid_b.1).abs();

        let tolerance = sprite_width as f64 * 0.1;

        if x_mirror_diff <= tolerance && y_diff <= tolerance {
            let mirrored_a: HashSet<(i32, i32)> =
                region_a_pixels.iter().map(|&(x, y)| (sprite_width as i32 - 1 - x, y)).collect();

            let intersection = mirrored_a.intersection(region_b_pixels).count();
            let union = mirrored_a.union(region_b_pixels).count();
            let shape_similarity = intersection as f64 / union as f64;

            if shape_similarity >= 0.5 {
                let position_score = 1.0 - (x_mirror_diff + y_diff) / (2.0 * tolerance);
                let confidence =
                    (size_ratio * 0.2 + shape_similarity * 0.5 + position_score * 0.3).min(1.0);

                if confidence >= 0.6 {
                    return Some(RelationshipInference::new(
                        region_a_name.to_string(),
                        RelationshipType::PairedWith,
                        region_b_name.to_string(),
                        confidence,
                    ));
                }
            }
        }

        None
    }
}

/// Input for relationship inference: region with its pixels and color.
#[derive(Debug, Clone)]
pub struct RegionData {
    /// Region/token name
    pub name: String,
    /// Set of pixel coordinates belonging to this region
    pub pixels: HashSet<(i32, i32)>,
    /// RGBA color of this region
    pub color: [u8; 4],
}

/// Batch inference of relationships between regions.
pub fn infer_relationships_batch(
    regions: &[RegionData],
    sprite_width: u32,
) -> Vec<RelationshipInference> {
    let mut relationships = Vec::new();

    for i in 0..regions.len() {
        for j in 0..regions.len() {
            if i == j {
                continue;
            }

            let a = &regions[i];
            let b = &regions[j];

            if let Some(rel) =
                RelationshipInferrer::infer_derives_from(&a.name, a.color, &b.name, b.color)
            {
                relationships.push(rel);
            }

            if i < j {
                if let Some(rel) = RelationshipInferrer::infer_contained_within(
                    &a.name, &a.pixels, &b.name, &b.pixels,
                ) {
                    relationships.push(rel);
                }
                if let Some(rel) = RelationshipInferrer::infer_contained_within(
                    &b.name, &b.pixels, &a.name, &a.pixels,
                ) {
                    relationships.push(rel);
                }
            }

            if i < j {
                if let Some(rel) =
                    RelationshipInferrer::infer_adjacent_to(&a.name, &a.pixels, &b.name, &b.pixels)
                {
                    relationships.push(rel);
                }
            }

            if i < j {
                if let Some(rel) = RelationshipInferrer::infer_paired_with(
                    &a.name,
                    &a.pixels,
                    &b.name,
                    &b.pixels,
                    sprite_width,
                ) {
                    relationships.push(rel);
                }
            }
        }
    }

    relationships.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    relationships
}
