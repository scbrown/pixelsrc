#!/bin/bash
# TTP Demo Script - Shows current capabilities
# Updated: Phase 0 (In Progress)

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    TTP (Text To Pixel)                       ║"
echo "║              Demo - Phase 0 MVP (In Progress)                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Phase 0 is in progress. Project scaffolding complete."
echo ""
echo "Current status: Phase 0 in progress"
echo ""
echo "Completed:"
echo "  [x] Task 0.1: Project scaffolding (Cargo.toml, module stubs)"
echo ""
echo "In progress:"
echo "  [ ] Task 0.2: Data models"
echo "  [ ] Task 0.3: Color parsing"
echo "  [ ] Task 0.4: Token parsing"
echo "  [ ] Task 0.5: JSONL parser"
echo "  [ ] Task 0.6: Palette registry"
echo "  [ ] Task 0.7: Sprite renderer"
echo "  [ ] Task 0.8: PNG output"
echo "  [ ] Task 0.9: CLI implementation"
echo "  [ ] Task 0.10: Integration tests"
echo ""
echo "Planned features:"
echo "  Phase 0: Parse JSONL -> PNG rendering"
echo "  Phase 1: Built-in palettes (@gameboy, @nes, @pico8)"
echo "  Phase 2: Animation and spritesheet export"
echo "  Phase 3: Game engine integration (Unity, Godot, Tiled)"
echo "  Phase 4: VS Code extension, web previewer, emoji output"
echo ""
echo "To get started, see: docs/plan/README.md"
echo ""

# Show example input format
echo "── Example TTP Format ─────────────────────────────────────────"
echo ""
if [ -f examples/coin.jsonl ]; then
    cat examples/coin.jsonl
else
    echo '{"type": "palette", "name": "coin", "colors": {"{_}": "#00000000", "{gold}": "#FFD700"}}'
    echo '{"type": "sprite", "name": "coin", "size": [8, 8], "palette": "coin", "grid": ["{_}{gold}{gold}{_}",...]}'
fi
echo ""
echo "══════════════════════════════════════════════════════════════"
