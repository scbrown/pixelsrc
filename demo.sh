#!/bin/bash
# TTP Demo Script - Shows current capabilities
# Updated: Phase 0 (Complete)

set -e

echo "========================================================================"
echo "                        TTP (Text To Pixel)"
echo "                      Demo - Phase 0 MVP (Complete)"
echo "========================================================================"
echo ""

# Check if binary exists
if [ ! -f target/release/pxl ]; then
    echo "Building pxl..."
    cargo build --release --quiet
    echo ""
fi

PXL="./target/release/pxl"

echo "Phase 0 is complete! All tasks finished:"
echo "  [x] Task 0.1: Project scaffolding (Cargo.toml, module stubs)"
echo "  [x] Task 0.2: Data models (Palette, Sprite, Animation)"
echo "  [x] Task 0.3: Color parsing (#RGB, #RRGGBB, #RRGGBBAA)"
echo "  [x] Task 0.4: Token parsing ({token} extraction)"
echo "  [x] Task 0.5: JSONL parser (stream processing)"
echo "  [x] Task 0.6: Palette registry (named palette resolution)"
echo "  [x] Task 0.7: Sprite renderer (with lenient error handling)"
echo "  [x] Task 0.8: PNG output (save_png, generate_output_path)"
echo "  [x] Task 0.9: CLI implementation (clap, render command)"
echo "  [x] Task 0.10: Integration tests & demo"
echo ""

echo "-- CLI Usage --------------------------------------------------------"
echo ""
echo "  pxl render <input.jsonl>              Render all sprites"
echo "  pxl render <input.jsonl> -o out.png   Specify output file"
echo "  pxl render <input.jsonl> -o dir/      Output to directory"
echo "  pxl render <input.jsonl> --sprite X   Render only sprite X"
echo "  pxl render <input.jsonl> --strict     Treat warnings as errors"
echo ""

echo "-- Example 1: Simple Coin Sprite -----------------------------------"
echo ""
echo "Input: examples/coin.jsonl"
head -2 examples/coin.jsonl
echo "..."
echo ""
$PXL render examples/coin.jsonl -o /tmp/demo_coin.png
echo "Output: /tmp/demo_coin.png"
echo "Dimensions: $(file /tmp/demo_coin.png | grep -oE '[0-9]+ x [0-9]+')"
echo ""

echo "-- Example 2: Character Sprite -------------------------------------"
echo ""
echo "Input: examples/hero.jsonl"
$PXL render examples/hero.jsonl -o /tmp/demo_hero.png
echo "Output: /tmp/demo_hero.png"
echo "Dimensions: $(file /tmp/demo_hero.png | grep -oE '[0-9]+ x [0-9]+')"
echo ""

echo "-- Example 3: Multiple Sprites -------------------------------------"
echo ""
echo "Input: tests/fixtures/valid/multiple_sprites.jsonl"
$PXL render tests/fixtures/valid/multiple_sprites.jsonl -o /tmp/demo_multi_
ls /tmp/demo_multi_*.png 2>/dev/null | head -5
echo ""

echo "-- Example 4: Lenient Mode (with warnings) -------------------------"
echo ""
echo "Input has unknown token - lenient mode renders as magenta:"
$PXL render tests/fixtures/lenient/unknown_token.jsonl -o /tmp/demo_lenient.png 2>&1 || true
echo ""

echo "-- Example 5: Strict Mode ------------------------------------------"
echo ""
echo "Same input with --strict flag - should fail:"
$PXL render tests/fixtures/lenient/unknown_token.jsonl --strict -o /tmp/demo_strict.png 2>&1 || echo "(Expected failure - strict mode treats warnings as errors)"
echo ""

echo "========================================================================"
echo "Phase 0 Complete! Features:"
echo "  * Parse JSONL palette and sprite definitions"
echo "  * Render sprites to PNG"
echo "  * Named and inline palettes"
echo "  * Lenient mode (fill gaps, warn, continue)"
echo "  * Strict mode (fail on warnings)"
echo ""
echo "Coming in Phase 1: Built-in palettes (@gameboy, @nes, @pico8)"
echo "========================================================================"
