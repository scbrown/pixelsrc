//! Semantic context extraction for antialiasing.
//!
//! This module provides the `SemanticContext` struct which extracts semantic
//! information from rendered sprite regions to guide antialiasing decisions.

use crate::models::{Palette, RegionDef, RelationshipType, Role};
#[cfg(test)]
use crate::models::Relationship;
use crate::transforms::AnchorBounds;
use image::Rgba;
use std::collections::{HashMap, HashSet};

/// Information about a gradient pair from DerivesFrom relationships.
///
/// Gradient pairs represent color transitions (like skin to shadow) that
/// should be smoothly interpolated during antialiasing.
#[derive(Debug, Clone)]
pub struct GradientPair {
    /// Source token name (e.g., "skin_shadow")
    pub source_token: String,
    /// Target token name (e.g., "skin")
    pub target_token: String,
    /// RGBA color of the source token
    pub source_color: Rgba<u8>,
    /// RGBA color of the target token
    pub target_color: Rgba<u8>,
    /// Pixel coordinates where the two regions meet
    pub boundary_pixels: Vec<(i32, i32)>,
}

/// Information about adjacent regions.
///
/// Tracks which regions share boundaries, useful for smart blending decisions.
#[derive(Debug, Clone)]
pub struct AdjacencyInfo {
    /// First region name
    pub region_a: String,
    /// Second region name
    pub region_b: String,
    /// Pixel coordinates along the shared edge
    pub shared_edges: Vec<(i32, i32)>,
}

/// Semantic context extracted from rendered sprite regions.
///
/// This struct provides efficient lookups for antialiasing algorithms to make
/// per-pixel decisions based on semantic meaning rather than just color values.
///
/// # Example
///
/// ```ignore
/// let context = SemanticContext::extract(&rendered_regions, &palette, (width, height));
///
/// // Check if a pixel should be preserved
/// if context.is_anchor((5, 10)) {
///     // Skip antialiasing for this anchor pixel
/// }
///
/// // Check for gradient blending opportunities
/// if let Some(gradient) = context.get_gradient_at((x, y)) {
///     // Apply gradient interpolation
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct SemanticContext {
    /// Pixels belonging to each semantic role.
    ///
    /// Maps Role -> set of (x, y) coordinates for all pixels with that role.
    pub role_masks: HashMap<Role, HashSet<(i32, i32)>>,

    /// Quick lookup for anchor pixels.
    ///
    /// Anchors are critical details (like eyes) that should typically be
    /// preserved from smoothing to maintain sharpness.
    pub anchor_pixels: HashSet<(i32, i32)>,

    /// Bounding boxes for anchor regions.
    ///
    /// Used for anchor preservation during scaling operations.
    pub anchor_bounds: Vec<AnchorBounds>,

    /// Containment boundary pixels (hard edges).
    ///
    /// Pixels on the boundary of ContainedWithin relationships should not
    /// be blended across, preventing color bleeding between distinct regions.
    pub containment_edges: HashSet<(i32, i32)>,

    /// Gradient pairs from DerivesFrom relationships.
    ///
    /// These pairs represent shadow/highlight transitions that can be
    /// smoothly interpolated during antialiasing.
    pub gradient_pairs: Vec<GradientPair>,

    /// Adjacent region boundaries.
    ///
    /// Maps region pairs to their shared boundary pixels.
    pub adjacencies: Vec<AdjacencyInfo>,
}

impl SemanticContext {
    /// Create an empty semantic context.
    ///
    /// Use this when semantic information is not available or when
    /// `--no-semantic-aa` mode is enabled.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if a pixel position is an anchor that should be preserved.
    pub fn is_anchor(&self, pos: (i32, i32)) -> bool {
        self.anchor_pixels.contains(&pos)
    }

    /// Check if a pixel is on a containment edge (hard boundary).
    ///
    /// Containment edges should not be blended across during antialiasing.
    pub fn is_containment_edge(&self, pos: (i32, i32)) -> bool {
        self.containment_edges.contains(&pos)
    }

    /// Get the semantic role for a pixel position, if any.
    pub fn get_role(&self, pos: (i32, i32)) -> Option<Role> {
        for (role, pixels) in &self.role_masks {
            if pixels.contains(&pos) {
                return Some(*role);
            }
        }
        None
    }

    /// Get gradient pair information for a pixel, if it's at a gradient boundary.
    pub fn get_gradient_at(&self, pos: (i32, i32)) -> Option<&GradientPair> {
        for gradient in &self.gradient_pairs {
            if gradient.boundary_pixels.contains(&pos) {
                return Some(gradient);
            }
        }
        None
    }

    /// Check if a pixel is at an adjacency boundary between two regions.
    pub fn is_adjacency_boundary(&self, pos: (i32, i32)) -> bool {
        for adjacency in &self.adjacencies {
            if adjacency.shared_edges.contains(&pos) {
                return true;
            }
        }
        false
    }

    /// Scale the context for use with upscaled images.
    ///
    /// When applying 2x or 4x algorithms, the semantic context needs to be
    /// scaled to match the new pixel coordinates.
    pub fn scale(&self, factor: u32) -> Self {
        let scale_pixel_set = |pixels: &HashSet<(i32, i32)>| -> HashSet<(i32, i32)> {
            let mut scaled = HashSet::new();
            for &(x, y) in pixels {
                // Each pixel expands to a factor x factor block
                for dy in 0..factor as i32 {
                    for dx in 0..factor as i32 {
                        scaled.insert((x * factor as i32 + dx, y * factor as i32 + dy));
                    }
                }
            }
            scaled
        };

        let scale_pixel_vec = |pixels: &[(i32, i32)]| -> Vec<(i32, i32)> {
            let mut scaled = Vec::new();
            for &(x, y) in pixels {
                for dy in 0..factor as i32 {
                    for dx in 0..factor as i32 {
                        scaled.push((x * factor as i32 + dx, y * factor as i32 + dy));
                    }
                }
            }
            scaled
        };

        SemanticContext {
            role_masks: self
                .role_masks
                .iter()
                .map(|(role, pixels)| (*role, scale_pixel_set(pixels)))
                .collect(),
            anchor_pixels: scale_pixel_set(&self.anchor_pixels),
            anchor_bounds: self
                .anchor_bounds
                .iter()
                .map(|b| b.scaled(factor as f32, factor as f32))
                .collect(),
            containment_edges: scale_pixel_set(&self.containment_edges),
            gradient_pairs: self
                .gradient_pairs
                .iter()
                .map(|g| GradientPair {
                    source_token: g.source_token.clone(),
                    target_token: g.target_token.clone(),
                    source_color: g.source_color,
                    target_color: g.target_color,
                    boundary_pixels: scale_pixel_vec(&g.boundary_pixels),
                })
                .collect(),
            adjacencies: self
                .adjacencies
                .iter()
                .map(|a| AdjacencyInfo {
                    region_a: a.region_a.clone(),
                    region_b: a.region_b.clone(),
                    shared_edges: scale_pixel_vec(&a.shared_edges),
                })
                .collect(),
        }
    }
}

/// Rendered region data for semantic context extraction.
///
/// This represents a single region after rasterization, with its pixels,
/// color, and semantic metadata.
#[derive(Debug, Clone)]
pub struct RenderedRegion {
    /// Region/token name
    pub name: String,
    /// Set of pixel coordinates belonging to this region
    pub pixels: HashSet<(i32, i32)>,
    /// RGBA color of this region
    pub color: Rgba<u8>,
    /// Semantic role of this region, if any
    pub role: Option<Role>,
}

/// Extract semantic context from rendered regions.
///
/// This function analyzes rendered sprite regions along with palette metadata
/// to build a `SemanticContext` that guides antialiasing decisions.
///
/// # Arguments
///
/// * `regions` - Map of region names to rendered region data
/// * `palette` - The palette containing role and relationship definitions
/// * `canvas_size` - Width and height of the canvas
///
/// # Returns
///
/// A `SemanticContext` populated with role masks, anchor pixels, containment
/// edges, gradient pairs, and adjacency information.
pub fn extract_semantic_context(
    regions: &HashMap<String, RenderedRegion>,
    palette: &Palette,
    _canvas_size: (u32, u32),
) -> SemanticContext {
    let mut ctx = SemanticContext::default();

    // Extract role information from palette
    let roles = palette.roles.as_ref();
    let relationships = palette.relationships.as_ref();

    // Build role masks from regions
    for (name, region) in regions {
        // First check region's own role, then fall back to palette role
        let role = region.role.or_else(|| roles.and_then(|r| r.get(name).copied()));

        if let Some(role) = role {
            let mask = ctx.role_masks.entry(role).or_default();
            mask.extend(region.pixels.iter().copied());

            // Track anchor pixels separately for quick lookup
            if role == Role::Anchor {
                ctx.anchor_pixels.extend(region.pixels.iter().copied());

                // Build anchor bounds
                let points: Vec<(i32, i32)> = region.pixels.iter().copied().collect();
                if let Some(bounds) = AnchorBounds::from_points(&points) {
                    ctx.anchor_bounds.push(bounds);
                }
            }
        }
    }

    // Process relationships from palette
    if let Some(rels) = relationships {
        for (source_token, relationship) in rels {
            let target_token = &relationship.target;

            match relationship.relationship_type {
                RelationshipType::DerivesFrom => {
                    // Find boundary pixels between source and target
                    if let (Some(source_region), Some(target_region)) =
                        (regions.get(source_token), regions.get(target_token))
                    {
                        let boundary = find_boundary_pixels(&source_region.pixels, &target_region.pixels);
                        if !boundary.is_empty() {
                            ctx.gradient_pairs.push(GradientPair {
                                source_token: source_token.clone(),
                                target_token: target_token.clone(),
                                source_color: source_region.color,
                                target_color: target_region.color,
                                boundary_pixels: boundary,
                            });
                        }
                    }
                }
                RelationshipType::ContainedWithin => {
                    // Mark boundary pixels as containment edges
                    if let (Some(inner_region), Some(outer_region)) =
                        (regions.get(source_token), regions.get(target_token))
                    {
                        let boundary = find_boundary_pixels(&inner_region.pixels, &outer_region.pixels);
                        ctx.containment_edges.extend(boundary.iter().copied());
                    }
                }
                RelationshipType::AdjacentTo => {
                    // Track adjacency information
                    if let (Some(region_a), Some(region_b)) =
                        (regions.get(source_token), regions.get(target_token))
                    {
                        let boundary = find_boundary_pixels(&region_a.pixels, &region_b.pixels);
                        if !boundary.is_empty() {
                            ctx.adjacencies.push(AdjacencyInfo {
                                region_a: source_token.clone(),
                                region_b: target_token.clone(),
                                shared_edges: boundary,
                            });
                        }
                    }
                }
                RelationshipType::PairedWith => {
                    // PairedWith doesn't affect antialiasing directly
                }
            }
        }
    }

    // Auto-detect adjacencies for regions without explicit relationships
    let region_names: Vec<&String> = regions.keys().collect();
    for i in 0..region_names.len() {
        for j in (i + 1)..region_names.len() {
            let name_a = region_names[i];
            let name_b = region_names[j];

            // Skip if already tracked via explicit relationship
            if ctx.adjacencies.iter().any(|a| {
                (a.region_a == *name_a && a.region_b == *name_b)
                    || (a.region_a == *name_b && a.region_b == *name_a)
            }) {
                continue;
            }

            if let (Some(region_a), Some(region_b)) = (regions.get(name_a), regions.get(name_b)) {
                let boundary = find_boundary_pixels(&region_a.pixels, &region_b.pixels);
                if !boundary.is_empty() {
                    ctx.adjacencies.push(AdjacencyInfo {
                        region_a: name_a.clone(),
                        region_b: name_b.clone(),
                        shared_edges: boundary,
                    });
                }
            }
        }
    }

    ctx
}

/// Find boundary pixels between two regions.
///
/// Returns pixels from region_a that are adjacent (4-connected) to region_b.
fn find_boundary_pixels(
    region_a: &HashSet<(i32, i32)>,
    region_b: &HashSet<(i32, i32)>,
) -> Vec<(i32, i32)> {
    let mut boundary = Vec::new();

    for &(x, y) in region_a {
        let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        if neighbors.iter().any(|n| region_b.contains(n)) {
            boundary.push((x, y));
        }
    }

    boundary
}

/// Extract semantic context from region definitions and palette.
///
/// This is a convenience function that works with region definitions before
/// they are fully rendered, useful for pre-processing.
///
/// # Arguments
///
/// * `region_defs` - Map of region names to their definitions
/// * `rasterized` - Map of region names to their rasterized pixel sets
/// * `palette` - The palette with colors, roles, and relationships
/// * `canvas_size` - Width and height of the canvas
pub fn extract_from_definitions(
    region_defs: &HashMap<String, RegionDef>,
    rasterized: &HashMap<String, HashSet<(i32, i32)>>,
    palette: &Palette,
    canvas_size: (u32, u32),
) -> SemanticContext {
    // Build RenderedRegion structs from definitions and rasterized pixels
    let mut regions = HashMap::new();

    for (name, def) in region_defs {
        if let Some(pixels) = rasterized.get(name) {
            // Get color from palette
            let color = palette
                .colors
                .get(name)
                .and_then(|hex| crate::color::parse_color(hex).ok())
                .unwrap_or(Rgba([255, 0, 255, 255])); // Magenta for unknown

            regions.insert(
                name.clone(),
                RenderedRegion {
                    name: name.clone(),
                    pixels: pixels.clone(),
                    color,
                    role: def.role,
                },
            );
        }
    }

    extract_semantic_context(&regions, palette, canvas_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_context() {
        let ctx = SemanticContext::empty();
        assert!(ctx.role_masks.is_empty());
        assert!(ctx.anchor_pixels.is_empty());
        assert!(ctx.containment_edges.is_empty());
        assert!(ctx.gradient_pairs.is_empty());
    }

    #[test]
    fn test_is_anchor() {
        let mut ctx = SemanticContext::empty();
        ctx.anchor_pixels.insert((5, 10));
        ctx.anchor_pixels.insert((6, 10));

        assert!(ctx.is_anchor((5, 10)));
        assert!(ctx.is_anchor((6, 10)));
        assert!(!ctx.is_anchor((7, 10)));
    }

    #[test]
    fn test_is_containment_edge() {
        let mut ctx = SemanticContext::empty();
        ctx.containment_edges.insert((3, 4));

        assert!(ctx.is_containment_edge((3, 4)));
        assert!(!ctx.is_containment_edge((3, 5)));
    }

    #[test]
    fn test_get_role() {
        let mut ctx = SemanticContext::empty();
        let mut anchor_pixels = HashSet::new();
        anchor_pixels.insert((1, 1));
        ctx.role_masks.insert(Role::Anchor, anchor_pixels);

        let mut fill_pixels = HashSet::new();
        fill_pixels.insert((2, 2));
        ctx.role_masks.insert(Role::Fill, fill_pixels);

        assert_eq!(ctx.get_role((1, 1)), Some(Role::Anchor));
        assert_eq!(ctx.get_role((2, 2)), Some(Role::Fill));
        assert_eq!(ctx.get_role((3, 3)), None);
    }

    #[test]
    fn test_find_boundary_pixels() {
        let mut region_a = HashSet::new();
        region_a.insert((0, 0));
        region_a.insert((1, 0));
        region_a.insert((2, 0));

        let mut region_b = HashSet::new();
        region_b.insert((0, 1));
        region_b.insert((1, 1));
        region_b.insert((2, 1));

        let boundary = find_boundary_pixels(&region_a, &region_b);

        // All of region_a's pixels are adjacent to region_b
        assert_eq!(boundary.len(), 3);
        assert!(boundary.contains(&(0, 0)));
        assert!(boundary.contains(&(1, 0)));
        assert!(boundary.contains(&(2, 0)));
    }

    #[test]
    fn test_scale_context() {
        let mut ctx = SemanticContext::empty();
        ctx.anchor_pixels.insert((1, 1));
        ctx.containment_edges.insert((2, 2));

        let scaled = ctx.scale(2);

        // (1, 1) should expand to (2, 2), (2, 3), (3, 2), (3, 3)
        assert!(scaled.anchor_pixels.contains(&(2, 2)));
        assert!(scaled.anchor_pixels.contains(&(2, 3)));
        assert!(scaled.anchor_pixels.contains(&(3, 2)));
        assert!(scaled.anchor_pixels.contains(&(3, 3)));
        assert_eq!(scaled.anchor_pixels.len(), 4);

        // (2, 2) should expand to (4, 4), (4, 5), (5, 4), (5, 5)
        assert!(scaled.containment_edges.contains(&(4, 4)));
        assert!(scaled.containment_edges.contains(&(4, 5)));
        assert!(scaled.containment_edges.contains(&(5, 4)));
        assert!(scaled.containment_edges.contains(&(5, 5)));
    }

    #[test]
    fn test_extract_anchor_pixels() {
        let mut regions = HashMap::new();

        let mut anchor_pixels = HashSet::new();
        anchor_pixels.insert((5, 6));
        anchor_pixels.insert((10, 6));

        regions.insert(
            "eye".to_string(),
            RenderedRegion {
                name: "eye".to_string(),
                pixels: anchor_pixels,
                color: Rgba([0, 0, 0, 255]),
                role: Some(Role::Anchor),
            },
        );

        let palette = Palette::default();
        let ctx = extract_semantic_context(&regions, &palette, (16, 16));

        assert!(ctx.is_anchor((5, 6)));
        assert!(ctx.is_anchor((10, 6)));
        assert!(!ctx.is_anchor((7, 6)));
        assert_eq!(ctx.anchor_bounds.len(), 1);
    }

    #[test]
    fn test_extract_containment_edges() {
        let mut regions = HashMap::new();

        // Inner region (eye)
        let mut eye_pixels = HashSet::new();
        eye_pixels.insert((5, 5));

        regions.insert(
            "eye".to_string(),
            RenderedRegion {
                name: "eye".to_string(),
                pixels: eye_pixels,
                color: Rgba([0, 0, 0, 255]),
                role: Some(Role::Anchor),
            },
        );

        // Outer region (skin) - surrounds the eye
        let mut skin_pixels = HashSet::new();
        skin_pixels.insert((4, 5));
        skin_pixels.insert((6, 5));
        skin_pixels.insert((5, 4));
        skin_pixels.insert((5, 6));

        regions.insert(
            "skin".to_string(),
            RenderedRegion {
                name: "skin".to_string(),
                pixels: skin_pixels,
                color: Rgba([255, 200, 150, 255]),
                role: Some(Role::Fill),
            },
        );

        // Create palette with ContainedWithin relationship
        let mut palette = Palette::default();
        let mut relationships = HashMap::new();
        relationships.insert(
            "eye".to_string(),
            Relationship {
                relationship_type: RelationshipType::ContainedWithin,
                target: "skin".to_string(),
            },
        );
        palette.relationships = Some(relationships);

        let ctx = extract_semantic_context(&regions, &palette, (16, 16));

        // The eye pixel (5, 5) should be a containment edge since it borders skin
        assert!(ctx.is_containment_edge((5, 5)));
    }

    #[test]
    fn test_extract_gradient_pairs() {
        let mut regions = HashMap::new();

        // Base skin region
        let mut skin_pixels = HashSet::new();
        skin_pixels.insert((5, 5));
        skin_pixels.insert((5, 6));

        regions.insert(
            "skin".to_string(),
            RenderedRegion {
                name: "skin".to_string(),
                pixels: skin_pixels,
                color: Rgba([255, 200, 150, 255]),
                role: Some(Role::Fill),
            },
        );

        // Shadow region - adjacent to skin
        let mut shadow_pixels = HashSet::new();
        shadow_pixels.insert((5, 7));
        shadow_pixels.insert((5, 8));

        regions.insert(
            "skin_shadow".to_string(),
            RenderedRegion {
                name: "skin_shadow".to_string(),
                pixels: shadow_pixels,
                color: Rgba([200, 150, 100, 255]),
                role: Some(Role::Shadow),
            },
        );

        // Create palette with DerivesFrom relationship
        let mut palette = Palette::default();
        let mut relationships = HashMap::new();
        relationships.insert(
            "skin_shadow".to_string(),
            Relationship {
                relationship_type: RelationshipType::DerivesFrom,
                target: "skin".to_string(),
            },
        );
        palette.relationships = Some(relationships);

        let ctx = extract_semantic_context(&regions, &palette, (16, 16));

        assert_eq!(ctx.gradient_pairs.len(), 1);
        let gradient = &ctx.gradient_pairs[0];
        assert_eq!(gradient.source_token, "skin_shadow");
        assert_eq!(gradient.target_token, "skin");
        // The boundary is at (5, 7) which is adjacent to (5, 6)
        assert!(gradient.boundary_pixels.contains(&(5, 7)));
    }
}
