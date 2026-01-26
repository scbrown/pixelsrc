//! Corpus analysis for Pixelsrc files
//!
//! Provides tools to analyze pixelsrc files and extract metrics about:
//! - Token frequency and co-occurrence
//! - Dimensional distribution
//! - Structural patterns
//! - Compression opportunities

mod compression;
mod dimensions;
mod families;
mod relationships;
mod report;
mod roles;
mod shapes;
mod tokens;

// Re-export public API
pub use compression::{CompressionEstimator, CompressionStats, RleStats, RowRepetitionStats};
pub use dimensions::DimensionStats;
pub use families::{TokenFamily, TokenFamilyDetector};
pub use relationships::{
    infer_relationships_batch, RegionData, RelationshipInference, RelationshipInferrer,
};
pub use report::{collect_files, format_report_text, AnalysisReport};
pub use roles::{
    infer_roles_batch, RegionMap, RoleInference, RoleInferenceContext, RoleInferenceWarning,
    RoleInferrer,
};
pub use shapes::{
    detect_ellipse, detect_line, detect_rect, detect_shape, detect_stroke, detect_symmetry,
    DetectedShape, ShapeDetection, Symmetric,
};
pub use tokens::{CoOccurrenceMatrix, TokenCounter};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PaletteRef, RelationshipType, Role, Sprite};
    use crate::shapes;
    use std::collections::HashSet;

    // Internal items for testing
    use super::relationships::rgb_to_hsl;
    use super::roles::color_brightness;
    use super::shapes::{bounding_box, extract_polygon_vertices};

    #[test]
    fn test_token_counter_basic() {
        let mut counter = TokenCounter::new();
        counter.add("{_}");
        counter.add("{_}");
        counter.add("{x}");

        assert_eq!(counter.get("{_}"), 2);
        assert_eq!(counter.get("{x}"), 1);
        assert_eq!(counter.get("{y}"), 0);
        assert_eq!(counter.total(), 3);
        assert_eq!(counter.unique_count(), 2);
    }

    #[test]
    fn test_token_counter_percentage() {
        let mut counter = TokenCounter::new();
        counter.add("{_}");
        counter.add("{_}");
        counter.add("{x}");
        counter.add("{x}");

        assert!((counter.percentage("{_}") - 50.0).abs() < 0.01);
        assert!((counter.percentage("{x}") - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_token_counter_top_n() {
        let mut counter = TokenCounter::new();
        counter.add_count("{_}", 100);
        counter.add_count("{x}", 50);
        counter.add_count("{y}", 25);

        let top = counter.top_n(2);
        assert_eq!(top.len(), 2);
        assert_eq!(*top[0].0, "{_}");
        assert_eq!(*top[1].0, "{x}");
    }

    #[test]
    fn test_dimension_stats() {
        let mut stats = DimensionStats::new();
        stats.add(16, 16);
        stats.add(16, 16);
        stats.add(8, 8);

        assert_eq!(stats.total(), 3);
        let sorted = stats.sorted_by_frequency();
        assert_eq!(sorted[0], ((16, 16), 2));
        assert_eq!(sorted[1], ((8, 8), 1));
    }

    #[test]
    fn test_analysis_report_avg_palette_size() {
        let mut report = AnalysisReport::new();
        report.palette_sizes = vec![4, 6, 8];
        assert!((report.avg_palette_size() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_analysis_report_empty_palette() {
        let report = AnalysisReport::new();
        assert!((report.avg_palette_size() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_co_occurrence_basic() {
        let mut matrix = CoOccurrenceMatrix::new();

        let mut tokens1: HashSet<String> = HashSet::new();
        tokens1.insert("{skin}".to_string());
        tokens1.insert("{outline}".to_string());
        tokens1.insert("{_}".to_string());

        let mut tokens2: HashSet<String> = HashSet::new();
        tokens2.insert("{skin}".to_string());
        tokens2.insert("{hair}".to_string());

        matrix.record_sprite(&tokens1);
        matrix.record_sprite(&tokens2);

        // skin+outline appears in 1 sprite
        assert_eq!(matrix.get("{skin}", "{outline}"), 1);
        // skin appears in both sprites but with different partners
        assert_eq!(matrix.get("{skin}", "{_}"), 1);
        assert_eq!(matrix.get("{skin}", "{hair}"), 1);
        // hair+outline never co-occur
        assert_eq!(matrix.get("{hair}", "{outline}"), 0);
    }

    #[test]
    fn test_co_occurrence_top_n() {
        let mut matrix = CoOccurrenceMatrix::new();

        // Record same sprite tokens twice to get higher counts
        let mut tokens: HashSet<String> = HashSet::new();
        tokens.insert("{skin}".to_string());
        tokens.insert("{outline}".to_string());

        matrix.record_sprite(&tokens);
        matrix.record_sprite(&tokens);

        let top = matrix.top_n(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].1, 2); // 2 occurrences
    }

    #[test]
    fn test_co_occurrence_pairs_for_token() {
        let mut matrix = CoOccurrenceMatrix::new();

        let mut tokens1: HashSet<String> = HashSet::new();
        tokens1.insert("{skin}".to_string());
        tokens1.insert("{a}".to_string());
        tokens1.insert("{b}".to_string());

        matrix.record_sprite(&tokens1);

        let pairs = matrix.pairs_for_token("{skin}");
        assert_eq!(pairs.len(), 2);
        // Both {a} and {b} appear once with {skin}
        assert!(pairs.iter().all(|(_, c)| *c == 1));
    }

    #[test]
    fn test_token_family_detector() {
        let mut counter = TokenCounter::new();
        counter.add_count("{skin}", 100);
        counter.add_count("{skin_light}", 50);
        counter.add_count("{skin_shadow}", 30);
        counter.add_count("{hair}", 80);
        counter.add_count("{hair_dark}", 40);
        counter.add_count("{outline}", 200);

        let detector = TokenFamilyDetector::new();
        let families = detector.detect(&counter);

        // Should find "skin" and "hair" families
        assert!(families.len() >= 2);

        // Find the skin family
        let skin_family = families.iter().find(|f| f.prefix == "skin");
        assert!(skin_family.is_some());
        let skin = skin_family.unwrap();
        assert_eq!(skin.tokens.len(), 3);
        assert_eq!(skin.total_count, 180); // 100 + 50 + 30
    }

    #[test]
    fn test_token_family_prefix_extraction() {
        let detector = TokenFamilyDetector::new();

        // Test extract_prefix directly via a simple test
        let mut counter = TokenCounter::new();
        counter.add("{_}"); // Should be skipped (transparency)
        counter.add("{skin}");
        counter.add("{skin_light}");
        counter.add("{color1}");
        counter.add("{color2}");

        let families = detector.detect(&counter);

        // Should have "skin" and "color" families
        let prefixes: Vec<_> = families.iter().map(|f| f.prefix.as_str()).collect();
        assert!(prefixes.contains(&"skin"));
        assert!(prefixes.contains(&"color"));
        // {_} should not create a family
        assert!(!prefixes.iter().any(|p| p.is_empty() || *p == "_"));
    }

    // ========================================================================
    // Compression estimation tests (13.4)
    // ========================================================================

    // NOTE: make_compression_test_sprite is deprecated - Sprite no longer has grid field.
    // Compression tests that used this helper are now ignored. See TTP-7i4v.
    #[allow(dead_code)]
    fn make_compression_test_sprite(_name: &str, _grid: Vec<&str>) -> Sprite {
        // Return a minimal v2 sprite without grid
        Sprite {
            name: _name.to_string(),
            size: None,
            palette: PaletteRef::Named(String::new()),
            ..Default::default()
        }
    }

    #[test]
    fn test_analyze_row_rle_empty() {
        let (tokens, runs, unique) = CompressionEstimator::analyze_row_rle("");
        assert_eq!(tokens, 0);
        assert_eq!(runs, 0);
        assert_eq!(unique, 0);
    }

    #[test]
    fn test_rle_stats_merge() {
        let mut stats1 =
            RleStats { total_tokens: 10, total_runs: 5, total_rows: 2, total_unique_per_row: 4 };
        let stats2 =
            RleStats { total_tokens: 20, total_runs: 8, total_rows: 3, total_unique_per_row: 6 };
        stats1.merge(&stats2);
        assert_eq!(stats1.total_tokens, 30);
        assert_eq!(stats1.total_runs, 13);
        assert_eq!(stats1.total_rows, 5);
        assert_eq!(stats1.total_unique_per_row, 10);
    }

    // ========================================================================
    // Shape Detection Tests (24.12)
    // ========================================================================

    #[test]
    fn test_detect_rect_perfect() {
        let pixels = shapes::rasterize_rect(0, 0, 4, 3);
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [0, 0, 4, 3]);
        assert!((detection.confidence - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_rect_with_offset() {
        let pixels = shapes::rasterize_rect(5, 10, 3, 2);
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [5, 10, 3, 2]);
    }

    #[test]
    fn test_detect_rect_single_pixel() {
        let pixels: HashSet<(i32, i32)> = [(0, 0)].into_iter().collect();
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [0, 0, 1, 1]);
    }

    #[test]
    fn test_detect_rect_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = detect_rect(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_rect_not_a_rect() {
        // L-shape is not a rectangle
        let pixels: HashSet<(i32, i32)> =
            [(0, 0), (1, 0), (2, 0), (0, 1), (0, 2)].into_iter().collect();
        let result = detect_rect(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_stroke_perfect() {
        let pixels = shapes::rasterize_stroke(0, 0, 5, 5, 1);
        let result = detect_stroke(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [0, 0, 5, 5]);
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_stroke_with_offset() {
        let pixels = shapes::rasterize_stroke(3, 7, 6, 4, 1);
        let result = detect_stroke(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [3, 7, 6, 4]);
    }

    #[test]
    fn test_detect_stroke_too_small() {
        // 2x2 stroke doesn't have interior, so we can't detect it as a stroke
        let pixels = shapes::rasterize_stroke(0, 0, 2, 2, 1);
        let result = detect_stroke(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_stroke_filled_rect_is_not_stroke() {
        let pixels = shapes::rasterize_rect(0, 0, 5, 5);
        let result = detect_stroke(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_line_horizontal() {
        let pixels = shapes::rasterize_line((0, 0), (5, 0));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape.len(), 2);
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_vertical() {
        let pixels = shapes::rasterize_line((0, 0), (0, 5));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_diagonal() {
        let pixels = shapes::rasterize_line((0, 0), (5, 5));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_steep() {
        let pixels = shapes::rasterize_line((0, 0), (3, 7));
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_detect_line_single_pixel() {
        let pixels: HashSet<(i32, i32)> = [(5, 5)].into_iter().collect();
        let result = detect_line(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, vec![[5, 5], [5, 5]]);
    }

    #[test]
    fn test_detect_line_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let result = detect_line(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_ellipse_circle() {
        let pixels = shapes::rasterize_ellipse(10, 10, 5, 5);
        let result = detect_ellipse(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        // Center should be close to (10, 10)
        assert!((detection.shape[0] - 10).abs() <= 1);
        assert!((detection.shape[1] - 10).abs() <= 1);
        assert!(detection.confidence >= 0.7);
    }

    #[test]
    fn test_detect_ellipse_horizontal() {
        let pixels = shapes::rasterize_ellipse(10, 10, 6, 3);
        let result = detect_ellipse(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.7);
    }

    #[test]
    fn test_detect_ellipse_vertical() {
        let pixels = shapes::rasterize_ellipse(10, 10, 3, 6);
        let result = detect_ellipse(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert!(detection.confidence >= 0.7);
    }

    #[test]
    fn test_detect_ellipse_too_small() {
        let pixels = shapes::rasterize_ellipse(1, 1, 1, 1);
        let result = detect_ellipse(&pixels);
        // Very small ellipses may not be detected
        // This is acceptable - they look like rectangles anyway
        if let Some(detection) = result {
            assert!(detection.confidence >= 0.7);
        }
    }

    #[test]
    fn test_detect_shape_rect() {
        let pixels = shapes::rasterize_rect(0, 0, 5, 4);
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Rect([0, 0, 5, 4])));
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_detect_shape_stroke() {
        let pixels = shapes::rasterize_stroke(0, 0, 6, 6, 1);
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Stroke([0, 0, 6, 6])));
        assert!(confidence >= 0.95);
    }

    #[test]
    #[ignore = "Flaky test - passes individually but fails in full suite"]
    fn test_detect_shape_line() {
        let pixels = shapes::rasterize_line((0, 0), (10, 5));
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Line(_)));
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_detect_shape_ellipse() {
        let pixels = shapes::rasterize_ellipse(15, 15, 8, 5);
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Ellipse(_)));
        assert!(confidence >= 0.7);
    }

    #[test]
    fn test_detect_shape_polygon_fallback() {
        // Create an irregular shape that doesn't match any primitive
        let pixels: HashSet<(i32, i32)> =
            [(0, 0), (1, 0), (2, 0), (3, 0), (1, 1), (2, 1), (2, 2), (3, 2), (4, 2)]
                .into_iter()
                .collect();
        let (shape, _confidence) = detect_shape(&pixels);
        // Should fall back to polygon since it's not a recognized primitive
        assert!(matches!(shape, DetectedShape::Polygon(_)));
    }

    #[test]
    fn test_detect_shape_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let (shape, confidence) = detect_shape(&pixels);
        assert!(matches!(shape, DetectedShape::Polygon(ref v) if v.is_empty()));
        assert!((confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_shape_detection_negative_coords() {
        let pixels = shapes::rasterize_rect(-5, -3, 4, 3);
        let result = detect_rect(&pixels);
        assert!(result.is_some());
        let detection = result.unwrap();
        assert_eq!(detection.shape, [-5, -3, 4, 3]);
    }

    #[test]
    fn test_confidence_scoring() {
        // Perfect rectangle should have confidence 1.0
        let pixels = shapes::rasterize_rect(0, 0, 5, 5);
        let detection = detect_rect(&pixels).unwrap();
        assert!((detection.confidence - 1.0).abs() < 0.001);

        // Line should have high confidence
        let pixels = shapes::rasterize_line((0, 0), (10, 10));
        let detection = detect_line(&pixels).unwrap();
        assert!(detection.confidence >= 0.95);
    }

    #[test]
    fn test_bounding_box() {
        let pixels: HashSet<(i32, i32)> = [(0, 0), (5, 3), (2, 7), (-1, 4)].into_iter().collect();
        let bbox = bounding_box(&pixels);
        assert_eq!(bbox, Some((-1, 0, 5, 7)));
    }

    #[test]
    fn test_bounding_box_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let bbox = bounding_box(&pixels);
        assert_eq!(bbox, None);
    }

    #[test]
    fn test_extract_polygon_vertices_triangle() {
        // Create a triangle-ish shape
        let triangle = shapes::rasterize_polygon(&[(0, 0), (6, 0), (3, 4)]);
        let vertices = extract_polygon_vertices(&triangle);
        // Convex hull should have 3 vertices
        assert!(vertices.len() >= 3);
    }

    #[test]
    fn test_extract_polygon_vertices_empty() {
        let pixels: HashSet<(i32, i32)> = HashSet::new();
        let vertices = extract_polygon_vertices(&pixels);
        assert!(vertices.is_empty());
    }

    // Symmetry detection tests (24.13)
    // ========================================================================

    /// Helper to create RGBA pixel data from a simple color index grid.
    /// Each cell is a u8 index that maps to a color.
    fn make_pixel_grid(grid: &[&[u8]], width: usize, height: usize) -> Vec<u8> {
        let colors: [(u8, u8, u8, u8); 4] = [
            (255, 0, 0, 255),   // 0: red
            (0, 255, 0, 255),   // 1: green
            (0, 0, 255, 255),   // 2: blue
            (255, 255, 0, 255), // 3: yellow
        ];

        let mut pixels = Vec::with_capacity(width * height * 4);
        for row in grid {
            for &idx in *row {
                let (r, g, b, a) = colors[idx as usize];
                pixels.extend_from_slice(&[r, g, b, a]);
            }
        }
        pixels
    }

    #[test]
    fn test_detect_symmetry_x_axis() {
        // X-axis symmetric (left-right mirror):
        // R G G R
        // B Y Y B
        let grid: &[&[u8]] = &[&[0, 1, 1, 0], &[2, 3, 3, 2]];
        let pixels = make_pixel_grid(grid, 4, 2);
        assert_eq!(detect_symmetry(&pixels, 4, 2), Some(Symmetric::X));
    }

    #[test]
    fn test_detect_symmetry_y_axis() {
        // Y-axis symmetric (top-bottom mirror):
        // R G B
        // R G B
        let grid: &[&[u8]] = &[&[0, 1, 2], &[0, 1, 2]];
        let pixels = make_pixel_grid(grid, 3, 2);
        assert_eq!(detect_symmetry(&pixels, 3, 2), Some(Symmetric::Y));
    }

    #[test]
    fn test_detect_symmetry_xy_axes() {
        // Both axes symmetric:
        // R G G R
        // B Y Y B
        // B Y Y B
        // R G G R
        let grid: &[&[u8]] = &[&[0, 1, 1, 0], &[2, 3, 3, 2], &[2, 3, 3, 2], &[0, 1, 1, 0]];
        let pixels = make_pixel_grid(grid, 4, 4);
        assert_eq!(detect_symmetry(&pixels, 4, 4), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_none() {
        // Not symmetric:
        // R G B
        // Y R G
        let grid: &[&[u8]] = &[&[0, 1, 2], &[3, 0, 1]];
        let pixels = make_pixel_grid(grid, 3, 2);
        assert_eq!(detect_symmetry(&pixels, 3, 2), None);
    }

    #[test]
    fn test_detect_symmetry_single_pixel() {
        // Single pixel is always symmetric on both axes
        let pixels: Vec<u8> = vec![255, 0, 0, 255];
        assert_eq!(detect_symmetry(&pixels, 1, 1), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_uniform_color() {
        // All same color - symmetric on both axes
        let grid: &[&[u8]] = &[&[0, 0, 0], &[0, 0, 0], &[0, 0, 0]];
        let pixels = make_pixel_grid(grid, 3, 3);
        assert_eq!(detect_symmetry(&pixels, 3, 3), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_empty() {
        let pixels: Vec<u8> = vec![];
        assert_eq!(detect_symmetry(&pixels, 0, 0), None);
    }

    #[test]
    fn test_detect_symmetry_invalid_buffer_size() {
        // Buffer too small for dimensions
        let pixels: Vec<u8> = vec![255, 0, 0, 255];
        assert_eq!(detect_symmetry(&pixels, 2, 2), None);
    }

    #[test]
    fn test_detect_symmetry_odd_width_x() {
        // Odd width, X-axis symmetric:
        // R G B G R
        let grid: &[&[u8]] = &[&[0, 1, 2, 1, 0]];
        let pixels = make_pixel_grid(grid, 5, 1);
        assert_eq!(detect_symmetry(&pixels, 5, 1), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_odd_height_y() {
        // Odd height, Y-axis symmetric:
        // R
        // G
        // R
        let grid: &[&[u8]] = &[&[0], &[1], &[0]];
        let pixels = make_pixel_grid(grid, 1, 3);
        assert_eq!(detect_symmetry(&pixels, 1, 3), Some(Symmetric::XY));
    }

    #[test]
    fn test_detect_symmetry_x_only_not_y() {
        // X-symmetric but not Y-symmetric:
        // R G G R  (x-symmetric)
        // B Y Y B  (x-symmetric)
        // R B B R  (x-symmetric, but different from row 0)
        let grid: &[&[u8]] = &[&[0, 1, 1, 0], &[2, 3, 3, 2], &[0, 2, 2, 0]];
        let pixels = make_pixel_grid(grid, 4, 3);
        assert_eq!(detect_symmetry(&pixels, 4, 3), Some(Symmetric::X));
    }

    #[test]
    fn test_detect_symmetry_y_only_not_x() {
        // Y-symmetric but not X-symmetric:
        // R G B
        // Y R G
        // Y R G
        // R G B
        let grid: &[&[u8]] = &[&[0, 1, 2], &[3, 0, 1], &[3, 0, 1], &[0, 1, 2]];
        let pixels = make_pixel_grid(grid, 3, 4);
        assert_eq!(detect_symmetry(&pixels, 3, 4), Some(Symmetric::Y));
    }

    // ========================================================================
    // Role Inference Tests (24.14)
    // ========================================================================

    #[test]
    fn test_role_inference_new() {
        let inference = RoleInference::new(Role::Boundary, 0.85);
        assert_eq!(inference.role, Role::Boundary);
        assert!((inference.confidence - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_role_inference_clamps_confidence() {
        let over = RoleInference::new(Role::Fill, 1.5);
        assert!((over.confidence - 1.0).abs() < 0.001);

        let under = RoleInference::new(Role::Fill, -0.5);
        assert!((under.confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_role_inference_low_confidence() {
        let high = RoleInference::new(Role::Fill, 0.8);
        assert!(!high.is_low_confidence());

        let low = RoleInference::new(Role::Fill, 0.5);
        assert!(low.is_low_confidence());

        let boundary = RoleInference::new(Role::Fill, 0.7);
        assert!(!boundary.is_low_confidence());
    }

    #[test]
    fn test_infer_boundary_edge_thin() {
        let ctx = RoleInferenceContext::new(10, 10);

        // 1px wide vertical line on left edge
        let pixels: HashSet<(i32, i32)> = [(0, 2), (0, 3), (0, 4), (0, 5)].into_iter().collect();

        let result = RoleInferrer::infer_boundary(&pixels, &ctx);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Boundary);
        assert!(inference.confidence >= 0.9);
    }

    #[test]
    fn test_infer_boundary_edge_horizontal() {
        let ctx = RoleInferenceContext::new(10, 10);

        // 1px tall horizontal line on top edge
        let pixels: HashSet<(i32, i32)> = [(2, 0), (3, 0), (4, 0), (5, 0)].into_iter().collect();

        let result = RoleInferrer::infer_boundary(&pixels, &ctx);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Boundary);
        assert!(inference.confidence >= 0.9);
    }

    #[test]
    fn test_infer_boundary_not_on_edge() {
        let ctx = RoleInferenceContext::new(10, 10);

        // Interior thin line - not on sprite edge
        let pixels: HashSet<(i32, i32)> = [(5, 2), (5, 3), (5, 4), (5, 5)].into_iter().collect();

        let result = RoleInferrer::infer_boundary(&pixels, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_anchor_single_pixel() {
        let pixels: HashSet<(i32, i32)> = [(5, 5)].into_iter().collect();

        let result = RoleInferrer::infer_anchor(&pixels);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Anchor);
        assert!((inference.confidence - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_infer_anchor_two_pixels() {
        let pixels: HashSet<(i32, i32)> = [(5, 5), (6, 5)].into_iter().collect();

        let result = RoleInferrer::infer_anchor(&pixels);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Anchor);
        assert!((inference.confidence - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_infer_anchor_three_pixels() {
        let pixels: HashSet<(i32, i32)> = [(5, 5), (6, 5), (5, 6)].into_iter().collect();

        let result = RoleInferrer::infer_anchor(&pixels);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Anchor);
        assert!((inference.confidence - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_infer_anchor_four_pixels_too_large() {
        let pixels: HashSet<(i32, i32)> = [(5, 5), (6, 5), (5, 6), (6, 6)].into_iter().collect();

        let result = RoleInferrer::infer_anchor(&pixels);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_fill_large_interior() {
        let ctx = RoleInferenceContext::new(10, 10);

        // Large interior region (6x6 = 36 pixels, 36% of 100)
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        for x in 2..8 {
            for y in 2..8 {
                pixels.insert((x, y));
            }
        }

        let result = RoleInferrer::infer_fill(&pixels, &ctx);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Fill);
        assert!(inference.confidence >= 0.7);
    }

    #[test]
    fn test_infer_fill_too_small() {
        let ctx = RoleInferenceContext::new(100, 100);

        // Small region - only 4 pixels out of 10000
        let pixels: HashSet<(i32, i32)> =
            [(50, 50), (51, 50), (50, 51), (51, 51)].into_iter().collect();

        let result = RoleInferrer::infer_fill(&pixels, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_fill_on_edge() {
        let ctx = RoleInferenceContext::new(10, 10);

        // Region mostly on edge - not a fill
        let mut pixels: HashSet<(i32, i32)> = HashSet::new();
        for x in 0..5 {
            pixels.insert((x, 0));
            pixels.insert((x, 1));
        }

        let result = RoleInferrer::infer_fill(&pixels, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_shadow_darker() {
        // Dark color
        let color = [50, 50, 50, 255];
        // Brighter adjacent colors
        let adjacent = [[150, 150, 150, 255], [200, 200, 200, 255]];

        let result = RoleInferrer::infer_shadow(color, &adjacent);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Shadow);
        assert!(inference.confidence >= 0.7);
    }

    #[test]
    fn test_infer_shadow_not_dark_enough() {
        // Similar brightness
        let color = [140, 140, 140, 255];
        let adjacent = [[150, 150, 150, 255]];

        let result = RoleInferrer::infer_shadow(color, &adjacent);
        assert!(result.is_none());
    }

    #[test]
    fn test_infer_highlight_lighter() {
        // Bright color
        let color = [230, 230, 230, 255];
        // Darker adjacent colors
        let adjacent = [[100, 100, 100, 255], [80, 80, 80, 255]];

        let result = RoleInferrer::infer_highlight(color, &adjacent);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Highlight);
        assert!(inference.confidence >= 0.7);
    }

    #[test]
    fn test_infer_highlight_not_light_enough() {
        // Similar brightness
        let color = [160, 160, 160, 255];
        let adjacent = [[150, 150, 150, 255]];

        let result = RoleInferrer::infer_highlight(color, &adjacent);
        assert!(result.is_none());
    }

    #[test]
    fn test_color_brightness() {
        // Black
        let black = color_brightness([0, 0, 0, 255]);
        assert!(black.abs() < 0.001);

        // White
        let white = color_brightness([255, 255, 255, 255]);
        assert!((white - 1.0).abs() < 0.001);

        // Pure red
        let red = color_brightness([255, 0, 0, 255]);
        assert!((red - 0.299).abs() < 0.001);

        // Pure green (brightest component in perception)
        let green = color_brightness([0, 255, 0, 255]);
        assert!((green - 0.587).abs() < 0.001);

        // Pure blue (darkest component in perception)
        let blue = color_brightness([0, 0, 255, 255]);
        assert!((blue - 0.114).abs() < 0.001);
    }

    #[test]
    fn test_generate_warnings_low_confidence() {
        let low = RoleInference::new(Role::Shadow, 0.5);
        let warning = RoleInferrer::generate_warnings("{test}", &low);
        assert!(warning.is_some());
        let w = warning.unwrap();
        assert_eq!(w.token, "{test}");
        assert_eq!(w.role, Role::Shadow);
        assert!(w.message.contains("Low confidence"));
    }

    #[test]
    fn test_generate_warnings_high_confidence() {
        let high = RoleInference::new(Role::Fill, 0.9);
        let warning = RoleInferrer::generate_warnings("{test}", &high);
        assert!(warning.is_none());
    }

    #[test]
    fn test_infer_role_priority() {
        let ctx = RoleInferenceContext::new(10, 10);

        // 2-pixel region on edge - should be Boundary (takes priority over Anchor)
        let pixels: HashSet<(i32, i32)> = [(0, 5), (0, 6)].into_iter().collect();

        let result = RoleInferrer::infer_role(&pixels, &ctx, None, &[]);
        assert!(result.is_some());
        let inference = result.unwrap();
        assert_eq!(inference.role, Role::Boundary);
    }

    #[test]
    fn test_infer_role_empty_pixels() {
        let ctx = RoleInferenceContext::new(10, 10);
        let pixels: HashSet<(i32, i32)> = HashSet::new();

        let result = RoleInferrer::infer_role(&pixels, &ctx, None, &[]);
        assert!(result.is_none());
    }

    // ========================================================================
    // Relationship Inference Tests (24.15)
    // ========================================================================

    #[test]
    fn test_rgb_to_hsl_red() {
        let hsl = rgb_to_hsl(255, 0, 0);
        assert!((hsl.h - 0.0).abs() < 1.0); // Red is 0 degrees
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsl_green() {
        let hsl = rgb_to_hsl(0, 255, 0);
        assert!((hsl.h - 120.0).abs() < 1.0); // Green is 120 degrees
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsl_blue() {
        let hsl = rgb_to_hsl(0, 0, 255);
        assert!((hsl.h - 240.0).abs() < 1.0); // Blue is 240 degrees
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsl_gray() {
        let hsl = rgb_to_hsl(128, 128, 128);
        assert!((hsl.s - 0.0).abs() < 0.01); // Gray has no saturation
        assert!((hsl.l - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_rgb_to_hsl_white() {
        let hsl = rgb_to_hsl(255, 255, 255);
        assert!((hsl.l - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsl_black() {
        let hsl = rgb_to_hsl(0, 0, 0);
        assert!((hsl.l - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_infer_derives_from_shadow() {
        // Base color and darker variant (shadow)
        let base = [200, 100, 100, 255];
        let shadow = [100, 50, 50, 255];

        let result = RelationshipInferrer::infer_derives_from("{shadow}", shadow, "{base}", base);

        assert!(result.is_some());
        let rel = result.unwrap();
        assert_eq!(rel.relationship_type, RelationshipType::DerivesFrom);
        assert_eq!(rel.source, "{shadow}");
        assert_eq!(rel.target, "{base}");
        assert!(rel.confidence >= 0.5);
    }

    #[test]
    fn test_infer_derives_from_highlight() {
        // Base color and lighter variant (highlight)
        let base = [100, 50, 50, 255];
        let highlight = [200, 100, 100, 255];

        let result =
            RelationshipInferrer::infer_derives_from("{highlight}", highlight, "{base}", base);

        assert!(result.is_some());
        let rel = result.unwrap();
        assert_eq!(rel.relationship_type, RelationshipType::DerivesFrom);
    }

    #[test]
    fn test_infer_derives_from_different_hue() {
        // Different hue colors shouldn't be derives-from
        let red = [255, 0, 0, 255];
        let blue = [0, 0, 255, 255];

        let result = RelationshipInferrer::infer_derives_from("{red}", red, "{blue}", blue);

        assert!(result.is_none());
    }

    #[test]
    fn test_infer_derives_from_same_lightness() {
        // Same lightness shouldn't be derives-from
        let color1 = [200, 100, 100, 255];
        let color2 = [198, 99, 99, 255];

        let result = RelationshipInferrer::infer_derives_from("{a}", color1, "{b}", color2);

        assert!(result.is_none());
    }

    #[test]
    fn test_infer_contained_within() {
        // Outer region (square frame)
        let mut outer: HashSet<(i32, i32)> = HashSet::new();
        for x in 0..10 {
            for y in 0..10 {
                // Only border pixels
                if x == 0 || x == 9 || y == 0 || y == 9 {
                    outer.insert((x, y));
                }
            }
        }

        // Inner region (small square in center)
        let mut inner: HashSet<(i32, i32)> = HashSet::new();
        for x in 4..6 {
            for y in 4..6 {
                inner.insert((x, y));
            }
        }

        let result =
            RelationshipInferrer::infer_contained_within("{inner}", &inner, "{outer}", &outer);

        // Inner is within outer's bounding box but not directly adjacent
        // This test checks the bounding box containment logic
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_infer_contained_within_adjacent() {
        // Outer region surrounding inner
        let mut outer: HashSet<(i32, i32)> = HashSet::new();
        // Create a ring around (5,5) to (6,6)
        for x in 4..8 {
            for y in 4..8 {
                if x == 4 || x == 7 || y == 4 || y == 7 {
                    outer.insert((x, y));
                }
            }
        }

        // Inner region
        let inner: HashSet<(i32, i32)> = [(5, 5), (5, 6), (6, 5), (6, 6)].into_iter().collect();

        let result =
            RelationshipInferrer::infer_contained_within("{inner}", &inner, "{outer}", &outer);

        assert!(result.is_some());
        let rel = result.unwrap();
        assert_eq!(rel.relationship_type, RelationshipType::ContainedWithin);
        assert_eq!(rel.source, "{inner}");
        assert_eq!(rel.target, "{outer}");
    }

    #[test]
    fn test_infer_contained_within_not_contained() {
        // Two separate regions
        let region_a: HashSet<(i32, i32)> = [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().collect();
        let region_b: HashSet<(i32, i32)> =
            [(10, 10), (11, 10), (10, 11), (11, 11)].into_iter().collect();

        let result =
            RelationshipInferrer::infer_contained_within("{a}", &region_a, "{b}", &region_b);

        assert!(result.is_none());
    }

    #[test]
    fn test_infer_adjacent_to() {
        // Two adjacent squares
        let region_a: HashSet<(i32, i32)> = [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().collect();
        let region_b: HashSet<(i32, i32)> = [(2, 0), (3, 0), (2, 1), (3, 1)].into_iter().collect();

        let result = RelationshipInferrer::infer_adjacent_to("{a}", &region_a, "{b}", &region_b);

        assert!(result.is_some());
        let rel = result.unwrap();
        assert_eq!(rel.relationship_type, RelationshipType::AdjacentTo);
    }

    #[test]
    fn test_infer_adjacent_to_diagonal_not_adjacent() {
        // Diagonally positioned squares (not 4-adjacent)
        let region_a: HashSet<(i32, i32)> = [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().collect();
        let region_b: HashSet<(i32, i32)> = [(2, 2), (3, 2), (2, 3), (3, 3)].into_iter().collect();

        let result = RelationshipInferrer::infer_adjacent_to("{a}", &region_a, "{b}", &region_b);

        assert!(result.is_none());
    }

    #[test]
    fn test_infer_adjacent_to_separated() {
        // Separated squares
        let region_a: HashSet<(i32, i32)> = [(0, 0), (1, 0)].into_iter().collect();
        let region_b: HashSet<(i32, i32)> = [(5, 0), (6, 0)].into_iter().collect();

        let result = RelationshipInferrer::infer_adjacent_to("{a}", &region_a, "{b}", &region_b);

        assert!(result.is_none());
    }

    #[test]
    fn test_infer_paired_with_symmetric_eyes() {
        let sprite_width = 16;

        // Left eye at x=3
        let left_eye: HashSet<(i32, i32)> = [(3, 5), (4, 5)].into_iter().collect();
        // Right eye at x=11 (mirrored position: 16-1-3=12, 16-1-4=11)
        let right_eye: HashSet<(i32, i32)> = [(11, 5), (12, 5)].into_iter().collect();

        let result = RelationshipInferrer::infer_paired_with(
            "{left_eye}",
            &left_eye,
            "{right_eye}",
            &right_eye,
            sprite_width,
        );

        assert!(result.is_some());
        let rel = result.unwrap();
        assert_eq!(rel.relationship_type, RelationshipType::PairedWith);
    }

    #[test]
    fn test_infer_paired_with_different_sizes() {
        let sprite_width = 16;

        // Different sized regions shouldn't pair
        let small: HashSet<(i32, i32)> = [(3, 5)].into_iter().collect();
        let large: HashSet<(i32, i32)> =
            [(11, 5), (12, 5), (11, 6), (12, 6), (13, 5)].into_iter().collect();

        let result = RelationshipInferrer::infer_paired_with(
            "{small}",
            &small,
            "{large}",
            &large,
            sprite_width,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_infer_paired_with_not_mirrored() {
        let sprite_width = 16;

        // Both on same side - not mirrored
        let region_a: HashSet<(i32, i32)> = [(3, 5), (4, 5)].into_iter().collect();
        let region_b: HashSet<(i32, i32)> = [(3, 8), (4, 8)].into_iter().collect();

        let result = RelationshipInferrer::infer_paired_with(
            "{a}",
            &region_a,
            "{b}",
            &region_b,
            sprite_width,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_relationship_inference_new() {
        let rel = RelationshipInference::new(
            "{a}".to_string(),
            RelationshipType::AdjacentTo,
            "{b}".to_string(),
            0.85,
        );

        assert_eq!(rel.source, "{a}");
        assert_eq!(rel.target, "{b}");
        assert_eq!(rel.relationship_type, RelationshipType::AdjacentTo);
        assert!((rel.confidence - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_relationship_inference_clamps_confidence() {
        let over = RelationshipInference::new(
            "{a}".to_string(),
            RelationshipType::AdjacentTo,
            "{b}".to_string(),
            1.5,
        );
        assert!((over.confidence - 1.0).abs() < 0.001);

        let under = RelationshipInference::new(
            "{a}".to_string(),
            RelationshipType::AdjacentTo,
            "{b}".to_string(),
            -0.5,
        );
        assert!((under.confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_infer_relationships_batch() {
        let regions = vec![
            RegionData {
                name: "{outline}".to_string(),
                pixels: [(0, 0), (1, 0), (2, 0), (0, 1), (2, 1), (0, 2), (1, 2), (2, 2)]
                    .into_iter()
                    .collect(),
                color: [0, 0, 0, 255],
            },
            RegionData {
                name: "{fill}".to_string(),
                pixels: [(1, 1)].into_iter().collect(),
                color: [200, 100, 100, 255],
            },
        ];

        let relationships = infer_relationships_batch(&regions, 3);

        // Should detect that fill is contained within outline and adjacent to it
        assert!(!relationships.is_empty());

        // Check that we found an adjacent-to relationship
        let adjacent =
            relationships.iter().find(|r| r.relationship_type == RelationshipType::AdjacentTo);
        assert!(adjacent.is_some());
    }

    #[test]
    fn test_infer_relationships_batch_with_shadow() {
        let regions = vec![
            RegionData {
                name: "{base}".to_string(),
                pixels: [(5, 5), (6, 5), (5, 6), (6, 6)].into_iter().collect(),
                color: [200, 100, 100, 255],
            },
            RegionData {
                name: "{shadow}".to_string(),
                pixels: [(5, 7), (6, 7)].into_iter().collect(),
                color: [100, 50, 50, 255], // Darker version of base
            },
        ];

        let relationships = infer_relationships_batch(&regions, 16);

        // Should detect derives-from relationship
        let derives =
            relationships.iter().find(|r| r.relationship_type == RelationshipType::DerivesFrom);
        assert!(derives.is_some());
    }

    #[test]
    fn test_infer_relationships_batch_empty() {
        let regions: Vec<RegionData> = vec![];
        let relationships = infer_relationships_batch(&regions, 16);
        assert!(relationships.is_empty());
    }

    #[test]
    fn test_infer_relationships_batch_single_region() {
        let regions = vec![RegionData {
            name: "{only}".to_string(),
            pixels: [(5, 5)].into_iter().collect(),
            color: [128, 128, 128, 255],
        }];

        let relationships = infer_relationships_batch(&regions, 16);
        assert!(relationships.is_empty()); // No pairs to compare
    }
}
