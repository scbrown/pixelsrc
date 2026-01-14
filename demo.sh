#!/bin/bash
# TTP Demo Script - Shows current capabilities
# Updated: Pre-Phase 0 (Planning)

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    TTP (Text To Pixel)                       ║"
echo "║              Demo - Planning Phase (Pre-MVP)                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "TTP is not yet implemented. This demo will be updated as each"
echo "phase is completed."
echo ""
echo "Current status: Planning"
echo ""
echo "Planned features:"
echo "  Phase 0: Parse JSONL → PNG rendering"
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
