//! Analyze Command Demo Tests
//!
//! Demonstrates the `pxl analyze` command functionality for extracting
//! corpus metrics from pixelsrc files.

use pixelsrc::analyze::{
    AnalysisReport, CoOccurrenceMatrix, CompressionEstimator, DimensionStats, TokenCounter,
};
use pixelsrc::models::{PaletteRef, Sprite};
use std::collections::{HashMap, HashSet};

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
// Token Counter Tests
// ============================================================================
#[test]
fn test_analyze_token_frequency() {
    let mut counter = TokenCounter::new();

    // Simulate analyzing a sprite with tokens
    counter.add("{skin}");
    counter.add("{skin}");
    counter.add("{skin}");
    counter.add("{hair}");
    counter.add("{hair}");
    counter.add("{shirt}");

    assert_eq!(counter.get("{skin}"), 3);
    assert_eq!(counter.get("{hair}"), 2);
    assert_eq!(counter.get("{shirt}"), 1);
    assert_eq!(counter.total(), 6);
}
#[test]
fn test_analyze_unique_tokens() {
    let mut counter = TokenCounter::new();

    counter.add("{a}");
    counter.add("{a}");
    counter.add("{b}");
    counter.add("{c}");
    counter.add("{c}");
    counter.add("{c}");

    assert_eq!(counter.unique_count(), 3, "Should have 3 unique tokens");
}
#[test]
fn test_analyze_token_percentage() {
    let mut counter = TokenCounter::new();

    // 50 uses of {a}, 50 uses of {b} = 50% each
    for _ in 0..50 {
        counter.add("{a}");
        counter.add("{b}");
    }

    let percentage_a = counter.percentage("{a}");
    let percentage_b = counter.percentage("{b}");

    assert!((percentage_a - 50.0).abs() < 0.1, "Token {{a}} should be ~50%");
    assert!((percentage_b - 50.0).abs() < 0.1, "Token {{b}} should be ~50%");
}
#[test]
fn test_analyze_top_tokens() {
    let mut counter = TokenCounter::new();

    counter.add_count("{common}", 100);
    counter.add_count("{medium}", 50);
    counter.add_count("{rare}", 10);
    counter.add_count("{very_rare}", 1);

    let top_2 = counter.top_n(2);

    assert_eq!(top_2.len(), 2);
    assert_eq!(top_2[0].0, "{common}");
    assert_eq!(top_2[1].0, "{medium}");
}

// ============================================================================
// Co-Occurrence Matrix Tests
// ============================================================================
#[test]
fn test_analyze_cooccurrence() {
    let mut matrix = CoOccurrenceMatrix::new();

    // Sprite 1 has {skin} and {hair}
    let sprite1_tokens: HashSet<String> =
        vec!["{skin}".to_string(), "{hair}".to_string()].into_iter().collect();
    matrix.record_sprite(&sprite1_tokens);

    // Sprite 2 also has {skin} and {hair}
    let sprite2_tokens: HashSet<String> =
        vec!["{skin}".to_string(), "{hair}".to_string()].into_iter().collect();
    matrix.record_sprite(&sprite2_tokens);

    // Sprite 3 has only {skin}
    let sprite3_tokens: HashSet<String> = vec!["{skin}".to_string()].into_iter().collect();
    matrix.record_sprite(&sprite3_tokens);

    // {skin} and {hair} co-occur in 2 sprites
    let cooccur_count = matrix.get("{skin}", "{hair}");
    assert_eq!(cooccur_count, 2, "skin+hair should co-occur in 2 sprites");
}
#[test]
fn test_analyze_top_pairs() {
    let mut matrix = CoOccurrenceMatrix::new();

    // Create sprites where certain pairs co-occur frequently
    for _ in 0..10 {
        let tokens: HashSet<String> =
            vec!["{a}".to_string(), "{b}".to_string()].into_iter().collect();
        matrix.record_sprite(&tokens);
    }
    for _ in 0..5 {
        let tokens: HashSet<String> =
            vec!["{c}".to_string(), "{d}".to_string()].into_iter().collect();
        matrix.record_sprite(&tokens);
    }

    let top_pairs = matrix.top_n(2);

    assert!(!top_pairs.is_empty());
    // {a},{b} should be the most common pair
    let ((token1, token2), count) = &top_pairs[0];
    assert_eq!(*count, 10);
    assert!(
        (*token1 == "{a}" && *token2 == "{b}") || (*token1 == "{b}" && *token2 == "{a}"),
        "Top pair should be a,b"
    );
}

// ============================================================================
// Dimension Stats Tests
// ============================================================================
#[test]
fn test_analyze_dimensions() {
    let mut stats = DimensionStats::new();

    // Record various sprite dimensions
    stats.add(8, 8);
    stats.add(8, 8);
    stats.add(16, 16);
    stats.add(16, 16);
    stats.add(16, 16);
    stats.add(32, 32);

    assert_eq!(stats.total(), 6);
}
#[test]
fn test_analyze_common_sizes() {
    let mut stats = DimensionStats::new();

    stats.add(8, 8);
    stats.add(16, 16);
    stats.add(16, 16);
    stats.add(16, 16);
    stats.add(32, 32);

    let sorted = stats.sorted_by_frequency();

    assert!(!sorted.is_empty());
    assert_eq!(sorted[0].0, (16, 16), "16x16 should be most common");
    assert_eq!(sorted[0].1, 3, "16x16 should appear 3 times");
}

// ============================================================================
// Compression Analysis Tests
// ============================================================================
/// @title Row Repetition Analysis
/// @description Detects duplicate rows within sprites.// ============================================================================
// Analysis Report Tests
// ============================================================================
/// @title Format Analysis as Text
/// @description Analysis can be output as human-readable text report.// ============================================================================
// Edge Cases
// ============================================================================
#[test]
fn test_analyze_empty_corpus() {
    let report = AnalysisReport::new();

    assert_eq!(report.total_sprites, 0);
    assert_eq!(report.token_counter.total(), 0);
    assert_eq!(report.token_counter.unique_count(), 0);
}
