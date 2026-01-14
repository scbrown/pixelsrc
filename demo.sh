#!/bin/bash
# TTP Demo Script - Shows current capabilities
# Updated: Phase 0 (Complete)

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    TTP (Text To Pixel)                       ║"
echo "║                Demo - Phase 0 MVP (Complete)                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if binary exists
if [ ! -f target/release/pxl ]; then
    echo "Building pxl..."
    cargo build --release
    echo ""
fi

echo "Phase 0 is complete. Basic JSONL -> PNG rendering works."
echo ""
echo "Completed tasks:"
echo "  [x] Task 0.1: Project scaffolding (Cargo.toml, module stubs)"
echo "  [x] Task 0.2: Data models (Palette, Sprite, Animation)"
echo "  [x] Task 0.3: Color parsing (#RGB, #RRGGBB, #RRGGBBAA)"
echo "  [x] Task 0.4: Token parsing ({token} extraction)"
echo "  [x] Task 0.5: JSONL parser (stream processing)"
echo "  [x] Task 0.6: Palette registry (named palette resolution)"
echo "  [x] Task 0.7: Sprite renderer (with lenient error handling)"
echo "  [x] Task 0.8: PNG output (save_png, generate_output_path)"
echo "  [x] Task 0.9: CLI implementation (clap, render command)"
echo "  [ ] Task 0.10: Integration tests & demo"
echo ""

echo "── CLI Usage ─────────────────────────────────────────────────"
echo ""
echo "  pxl render <input.jsonl>              Render all sprites"
echo "  pxl render <input.jsonl> -o out.png   Specify output file"
echo "  pxl render <input.jsonl> -o dir/      Output to directory"
echo "  pxl render <input.jsonl> --sprite X   Render only sprite X"
echo "  pxl render <input.jsonl> --strict     Treat warnings as errors"
echo ""

echo "── Demo: Rendering examples/coin.jsonl ──────────────────────"
echo ""
echo "Input file:"
cat examples/coin.jsonl
echo ""
echo ""
echo "Running: pxl render examples/coin.jsonl -o /tmp/coin_demo.png"
./target/release/pxl render examples/coin.jsonl -o /tmp/coin_demo.png
echo ""
echo "Output: /tmp/coin_demo.png (8x8 pixel coin sprite)"
echo ""

echo "── Demo: Multiple sprites from examples/hero.jsonl ──────────"
echo ""
echo "Running: pxl render examples/hero.jsonl -o /tmp/hero_"
./target/release/pxl render examples/hero.jsonl -o /tmp/hero_.png 2>&1 || true
echo ""

echo "── Planned Features ─────────────────────────────────────────"
echo ""
echo "  Phase 1: Built-in palettes (@gameboy, @nes, @pico8)"
echo "  Phase 2: Animation and spritesheet export"
echo "  Phase 3: Game engine integration (Unity, Godot, Tiled)"
echo "  Phase 4: VS Code extension, web previewer, emoji output"
echo ""
echo "══════════════════════════════════════════════════════════════"
