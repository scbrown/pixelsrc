//! Criterion benchmarks for Pixelsrc critical paths
//!
//! Benchmarks the core performance-critical operations:
//! - Tokenizer: Grid row parsing
//! - Parser: JSON/JSONL stream parsing
//! - Color: CSS color parsing (hex and functional)
//! - Renderer: Sprite to image rendering
//! - Atlas: Texture atlas packing

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use image::{Rgba, RgbaImage};
use pixelsrc::atlas::{pack_atlas, AtlasConfig, SpriteInput};
use pixelsrc::color::parse_color;
use pixelsrc::models::{PaletteRef, Sprite};
use pixelsrc::parser::{parse_line, parse_stream};
use pixelsrc::registry::ResolvedSprite;
use pixelsrc::renderer::{render_resolved, render_sprite};
use pixelsrc::tokenizer::tokenize;
use std::collections::HashMap;
use std::io::Cursor;

// =============================================================================
// Test Data Generators
// =============================================================================

/// Generate a grid row with n tokens
fn make_grid_row(n: usize) -> String {
    (0..n).map(|i| format!("{{t{}}}", i % 16)).collect()
}

/// Generate a sprite JSON line with given dimensions
fn make_sprite_json(width: usize, height: usize) -> String {
    let palette: Vec<String> = (0..16)
        .map(|i| format!("\"{{t{}}}\": \"#{:02X}{:02X}{:02X}\"", i, i * 16, i * 8, 255 - i * 16))
        .collect();

    let grid: Vec<String> = (0..height).map(|_| format!("\"{}\"", make_grid_row(width))).collect();

    format!(
        r#"{{"type": "sprite", "name": "bench_sprite", "palette": {{{}}}, "grid": [{}]}}"#,
        palette.join(", "),
        grid.join(", ")
    )
}

/// Generate JSONL content with multiple sprites
fn make_jsonl_content(sprite_count: usize, width: usize, height: usize) -> String {
    (0..sprite_count)
        .map(|i| {
            let palette: Vec<String> = (0..16)
                .map(|j| {
                    format!(
                        "\"{{t{}}}\": \"#{:02X}{:02X}{:02X}\"",
                        j,
                        (i * 16 + j) % 256,
                        j * 8,
                        255 - j * 16
                    )
                })
                .collect();

            let grid: Vec<String> =
                (0..height).map(|_| format!("\"{}\"", make_grid_row(width))).collect();

            format!(
                r#"{{"type": "sprite", "name": "sprite_{}", "palette": {{{}}}, "grid": [{}]}}"#,
                i,
                palette.join(", "),
                grid.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Create a test sprite with given dimensions
fn make_test_sprite(width: usize, height: usize) -> Sprite {
    let mut palette_colors = HashMap::new();
    for i in 0..16 {
        palette_colors.insert(format!("{{t{}}}", i), format!("#{:02X}{:02X}{:02X}", i * 16, i * 8, 255 - i * 16));
    }

    Sprite {
        name: "bench_sprite".to_string(),
        size: Some([width as u32, height as u32]),
        palette: PaletteRef::Inline(palette_colors.clone()),
        grid: (0..height).map(|_| make_grid_row(width)).collect(),
        metadata: None,
        ..Default::default()
    }
}

/// Create a ResolvedSprite for rendering benchmarks
fn make_resolved_sprite(width: usize, height: usize) -> ResolvedSprite {
    let mut palette = HashMap::new();
    for i in 0..16 {
        palette.insert(format!("{{t{}}}", i), format!("#{:02X}{:02X}{:02X}", i * 16, i * 8, 255 - i * 16));
    }

    ResolvedSprite {
        name: "bench_sprite".to_string(),
        size: Some([width as u32, height as u32]),
        grid: (0..height).map(|_| make_grid_row(width)).collect(),
        palette,
        warnings: vec![],
    }
}

/// Create sprite inputs for atlas packing benchmarks
fn make_sprite_inputs(count: usize, size: u32) -> Vec<SpriteInput> {
    let colors = [
        Rgba([255, 0, 0, 255]),
        Rgba([0, 255, 0, 255]),
        Rgba([0, 0, 255, 255]),
        Rgba([255, 255, 0, 255]),
    ];

    (0..count)
        .map(|i| SpriteInput {
            name: format!("sprite_{}", i),
            image: RgbaImage::from_pixel(size, size, colors[i % 4]),
            origin: None,
            boxes: None,
        })
        .collect()
}

// =============================================================================
// Tokenizer Benchmarks
// =============================================================================

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");

    // Benchmark different row lengths
    for size in [8, 16, 32, 64, 128].iter() {
        let row = make_grid_row(*size);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", size), &row, |b, row| {
            b.iter(|| tokenize(black_box(row)))
        });
    }

    // Benchmark with longer token names
    let long_tokens: String = (0..32).map(|i| format!("{{token_name_{:04}}}", i)).collect();
    group.bench_function("tokenize_long_names", |b| {
        b.iter(|| tokenize(black_box(&long_tokens)))
    });

    // Benchmark worst case: many warnings (characters outside tokens)
    let noisy_row = "x{a}y{b}z{c}w{d}v{e}u{f}t{g}s{h}r{i}q{j}";
    group.bench_function("tokenize_with_warnings", |b| {
        b.iter(|| tokenize(black_box(noisy_row)))
    });

    group.finish();
}

// =============================================================================
// Parser Benchmarks
// =============================================================================

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    // Single line parsing
    let small_sprite = make_sprite_json(8, 8);
    let medium_sprite = make_sprite_json(32, 32);
    let large_sprite = make_sprite_json(64, 64);

    group.bench_function("parse_line_8x8", |b| {
        b.iter(|| parse_line(black_box(&small_sprite), 1))
    });

    group.bench_function("parse_line_32x32", |b| {
        b.iter(|| parse_line(black_box(&medium_sprite), 1))
    });

    group.bench_function("parse_line_64x64", |b| {
        b.iter(|| parse_line(black_box(&large_sprite), 1))
    });

    // Stream parsing
    for (sprite_count, width, height) in [(10, 16, 16), (50, 16, 16), (10, 32, 32)].iter() {
        let content = make_jsonl_content(*sprite_count, *width, *height);
        let name = format!("parse_stream_{}x{}x{}", sprite_count, width, height);

        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_function(&name, |b| {
            b.iter(|| {
                let cursor = Cursor::new(black_box(&content));
                parse_stream(cursor)
            })
        });
    }

    group.finish();
}

// =============================================================================
// Color Parsing Benchmarks
// =============================================================================

fn bench_color(c: &mut Criterion) {
    let mut group = c.benchmark_group("color");

    // Hex color formats (fast path)
    group.bench_function("parse_hex_3", |b| b.iter(|| parse_color(black_box("#F00"))));

    group.bench_function("parse_hex_4", |b| b.iter(|| parse_color(black_box("#F00F"))));

    group.bench_function("parse_hex_6", |b| b.iter(|| parse_color(black_box("#FF0000"))));

    group.bench_function("parse_hex_8", |b| b.iter(|| parse_color(black_box("#FF0000FF"))));

    // CSS functional formats (uses lightningcss)
    group.bench_function("parse_rgb", |b| {
        b.iter(|| parse_color(black_box("rgb(255, 0, 0)")))
    });

    group.bench_function("parse_rgba", |b| {
        b.iter(|| parse_color(black_box("rgba(255, 0, 0, 0.5)")))
    });

    group.bench_function("parse_hsl", |b| {
        b.iter(|| parse_color(black_box("hsl(0, 100%, 50%)")))
    });

    group.bench_function("parse_named", |b| b.iter(|| parse_color(black_box("red"))));

    // Modern CSS syntax
    group.bench_function("parse_rgb_modern", |b| {
        b.iter(|| parse_color(black_box("rgb(255 0 0 / 50%)")))
    });

    // Complex: color-mix
    group.bench_function("parse_color_mix", |b| {
        b.iter(|| parse_color(black_box("color-mix(in oklch, red 70%, blue)")))
    });

    // Batch parsing (simulates parsing a palette)
    let colors = [
        "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF", "#FFFFFF", "#000000",
        "#F0F0F0", "#0F0F0F", "#123456", "#ABCDEF", "#FEDCBA", "#654321", "#AABBCC", "#CCBBAA",
    ];
    group.bench_function("parse_palette_16_hex", |b| {
        b.iter(|| {
            for color in &colors {
                let _ = parse_color(black_box(*color));
            }
        })
    });

    group.finish();
}

// =============================================================================
// Renderer Benchmarks
// =============================================================================

fn bench_renderer(c: &mut Criterion) {
    let mut group = c.benchmark_group("renderer");

    // Test different sprite sizes
    for size in [8, 16, 32, 64, 128].iter() {
        let sprite = make_test_sprite(*size, *size);
        let palette: HashMap<String, String> = match &sprite.palette {
            PaletteRef::Inline(p) => p.clone(),
            _ => HashMap::new(),
        };

        group.throughput(Throughput::Elements((*size * *size) as u64));
        group.bench_with_input(
            BenchmarkId::new("render_sprite", format!("{}x{}", size, size)),
            &(sprite.clone(), palette.clone()),
            |b, (sprite, palette)| b.iter(|| render_sprite(black_box(sprite), black_box(palette))),
        );
    }

    // Benchmark render_resolved (used in build pipeline)
    for size in [16, 32, 64].iter() {
        let resolved = make_resolved_sprite(*size, *size);

        group.throughput(Throughput::Elements((*size * *size) as u64));
        group.bench_with_input(
            BenchmarkId::new("render_resolved", format!("{}x{}", size, size)),
            &resolved,
            |b, resolved| b.iter(|| render_resolved(black_box(resolved))),
        );
    }

    // Benchmark with many unique tokens (stresses color cache)
    let mut wide_palette = HashMap::new();
    for i in 0..256 {
        wide_palette.insert(format!("{{c{}}}", i), format!("#{:02X}{:02X}{:02X}", i, i, i));
    }
    let wide_grid: Vec<String> = (0..16)
        .map(|row| (0..16).map(|col| format!("{{c{}}}", row * 16 + col)).collect())
        .collect();

    let wide_sprite = Sprite {
        name: "wide_palette".to_string(),
        size: Some([16, 16]),
        palette: PaletteRef::Inline(wide_palette.clone()),
        grid: wide_grid,
        metadata: None,
        ..Default::default()
    };

    group.bench_function("render_256_colors", |b| {
        b.iter(|| render_sprite(black_box(&wide_sprite), black_box(&wide_palette)))
    });

    group.finish();
}

// =============================================================================
// Atlas Packing Benchmarks
// =============================================================================

fn bench_atlas(c: &mut Criterion) {
    let mut group = c.benchmark_group("atlas");

    let config = AtlasConfig::default();

    // Test different sprite counts
    for count in [10, 50, 100, 200].iter() {
        let sprites = make_sprite_inputs(*count, 16);

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("pack_16x16", count), &sprites, |b, sprites| {
            b.iter(|| pack_atlas(black_box(sprites), black_box(&config), "bench"))
        });
    }

    // Test different sprite sizes
    for size in [8, 16, 32, 64].iter() {
        let sprites = make_sprite_inputs(50, *size);

        group.bench_with_input(
            BenchmarkId::new("pack_50_sprites", format!("{}x{}", size, size)),
            &sprites,
            |b, sprites| b.iter(|| pack_atlas(black_box(sprites), black_box(&config), "bench")),
        );
    }

    // Test with power-of-two enabled
    let pot_config = AtlasConfig { power_of_two: true, ..Default::default() };
    let sprites = make_sprite_inputs(50, 16);

    group.bench_function("pack_50_power_of_two", |b| {
        b.iter(|| pack_atlas(black_box(&sprites), black_box(&pot_config), "bench"))
    });

    // Test with padding
    let padded_config = AtlasConfig { padding: 2, ..Default::default() };

    group.bench_function("pack_50_with_padding", |b| {
        b.iter(|| pack_atlas(black_box(&sprites), black_box(&padded_config), "bench"))
    });

    // Test mixed sizes (common real-world scenario)
    let mixed_sprites: Vec<SpriteInput> = (0..50)
        .map(|i| {
            let size = match i % 4 {
                0 => 8,
                1 => 16,
                2 => 24,
                _ => 32,
            };
            SpriteInput {
                name: format!("mixed_{}", i),
                image: RgbaImage::from_pixel(size, size, Rgba([255, 0, 0, 255])),
                origin: None,
                boxes: None,
            }
        })
        .collect();

    group.bench_function("pack_mixed_sizes", |b| {
        b.iter(|| pack_atlas(black_box(&mixed_sprites), black_box(&config), "bench"))
    });

    group.finish();
}

// =============================================================================
// Criterion Configuration
// =============================================================================

criterion_group!(
    benches,
    bench_tokenizer,
    bench_parser,
    bench_color,
    bench_renderer,
    bench_atlas
);

criterion_main!(benches);
