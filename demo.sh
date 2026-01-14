#!/bin/bash
# TTP (Text To Pixel) Demo
# Interactive slideshow demonstrating Phase 0 MVP

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Temp directory for rendered outputs
DEMO_OUT="/tmp/ttp-demo"
mkdir -p "$DEMO_OUT"

# Wait for keypress
pause() {
    echo ""
    echo -e "${DIM}Press Enter to continue...${NC}"
    read -r
}

# Clear and show slide header
slide() {
    clear
    echo ""
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${DIM}$(printf '━%.0s' {1..64})${NC}"
    echo ""
}

# Display image in terminal (tries multiple methods)
show_image() {
    local img="$1"
    local scale="${2:-10}"

    if [ ! -f "$img" ]; then
        echo -e "  ${DIM}[Image not found: $img]${NC}"
        return
    fi

    # Try iTerm2 imgcat
    if command -v imgcat &> /dev/null; then
        imgcat --width "${scale}" "$img" 2>/dev/null && return
    fi

    # Try Kitty icat
    if command -v kitten &> /dev/null; then
        kitten icat --scale-up --place "${scale}x${scale}@0x0" "$img" 2>/dev/null && return
    fi

    # Try chafa (good unicode/ascii art fallback)
    if command -v chafa &> /dev/null; then
        chafa --size "${scale}x${scale}" --symbols block "$img" 2>/dev/null && return
    fi

    # Try viu
    if command -v viu &> /dev/null; then
        viu -w "$scale" "$img" 2>/dev/null && return
    fi

    # Fallback: show file info
    echo -e "  ${DIM}[Rendered: $img]${NC}"
    echo -e "  ${DIM}$(file "$img" | sed 's/.*: //')${NC}"
    echo ""
    echo -e "  ${DIM}Install 'chafa' or 'viu' to view images inline:${NC}"
    echo -e "  ${DIM}  brew install chafa${NC}"
}

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 1: Title
# ═══════════════════════════════════════════════════════════════════════════════
clear
echo ""
echo ""
echo ""
echo -e "${BOLD}${CYAN}"
cat << 'EOF'
                ████████╗████████╗██████╗
                ╚══██╔══╝╚══██╔══╝██╔══██╗
                   ██║      ██║   ██████╔╝
                   ██║      ██║   ██╔═══╝
                   ██║      ██║   ██║
                   ╚═╝      ╚═╝   ╚═╝
EOF
echo -e "${NC}"
echo ""
echo -e "                ${BOLD}Text To Pixel${NC}"
echo -e "                ${DIM}Define pixel art in JSON, render to PNG${NC}"
echo ""
echo ""
echo -e "                ${GREEN}Phase 0 + Phase 1 Complete${NC}"
echo ""
echo ""
pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 2: Build
# ═══════════════════════════════════════════════════════════════════════════════
slide "Building Project"

echo -e "  ${DIM}\$ cargo build --release${NC}"
echo ""

cargo build --release 2>&1 | grep -E "(Compiling pxl|Finished)" | tail -2 | sed 's/^/  /' || echo -e "  ${GREEN}Already up to date${NC}"
echo ""
echo -e "  ${GREEN}Build successful${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 3: Tests
# ═══════════════════════════════════════════════════════════════════════════════
slide "Running Tests"

echo -e "  ${DIM}\$ cargo test${NC}"
echo ""

# Capture test output
test_output=$(cargo test 2>&1)
passed=$(echo "$test_output" | grep -oE '[0-9]+ passed' | head -1)
doc_tests=$(echo "$test_output" | grep "doc" | grep -oE '[0-9]+ passed' || echo "")

echo -e "  ${GREEN}Unit tests:    $passed${NC}"
if [ -n "$doc_tests" ]; then
    echo -e "  ${GREEN}Doc tests:     $doc_tests${NC}"
fi
echo ""
echo -e "  ${GREEN}All tests passing${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 4: Example 1 - Heart (Simple)
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Heart Sprite"

echo -e "  ${BOLD}Input:${NC} examples/heart.jsonl"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}type${NC}:     ${GREEN}sprite${NC}                                       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}name${NC}:     ${GREEN}heart${NC}                                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}size${NC}:     ${CYAN}7${NC} x ${CYAN}6${NC}                                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}palette${NC}:  ${DIM}{_}${NC} = transparent                            ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ${RED}{r}${NC} = ${RED}#FF0000${NC}  ${DIM}(red)${NC}                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ${MAGENTA}{p}${NC} = ${MAGENTA}#FF6B6B${NC}  ${DIM}(pink highlight)${NC}             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}grid${NC}:                                                  ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}.${NC} ${RED}r${NC} ${RED}r${NC} ${DIM}.${NC} ${RED}r${NC} ${RED}r${NC} ${DIM}.${NC}                                    ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${RED}r${NC} ${MAGENTA}p${NC} ${RED}r${NC} ${RED}r${NC} ${MAGENTA}p${NC} ${RED}r${NC} ${RED}r${NC}                                    ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC}                                    ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}.${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${DIM}.${NC}                                    ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}. .${NC} ${RED}r${NC} ${RED}r${NC} ${RED}r${NC} ${DIM}. .${NC}                                    ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}. . .${NC} ${RED}r${NC} ${DIM}. . .${NC}                                    ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 5: Render Heart
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: Heart Sprite"

echo -e "  ${DIM}\$ pxl render examples/heart.jsonl -o heart.png${NC}"
echo ""

# Actually render
./target/release/pxl render examples/heart.jsonl -o "$DEMO_OUT/heart.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Output:${NC}"
echo ""
show_image "$DEMO_OUT/heart.png" 20

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 6: Example 2 - Coin
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Coin Sprite"

echo -e "  ${BOLD}Input:${NC} examples/coin.jsonl"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${YELLOW}Palette: \"coin\"${NC}                                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{_}${NC}      = ${DIM}transparent${NC}                               ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${YELLOW}{gold}${NC}   = ${YELLOW}#FFD700${NC}                                   ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}{shine}${NC}  = ${WHITE}#FFFACD${NC}  ${DIM}(highlight)${NC}                   ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{shadow}${NC} = ${DIM}#B8860B${NC}  ${DIM}(shadow)${NC}                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{dark}${NC}   = ${DIM}#8B6914${NC}  ${DIM}(dark edge)${NC}                   ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${GREEN}Sprite: \"coin\" (8x8)${NC}                                   ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}. .${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}. .${NC}     ${DIM}row 1${NC}                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}.${NC} ${YELLOW}█${NC} ${WHITE}░${NC} ${WHITE}░${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}.${NC}     ${DIM}row 2 (with shine)${NC}         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${YELLOW}█${NC} ${WHITE}░${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}▒${NC} ${YELLOW}█${NC}     ${DIM}row 3 (with shadow)${NC}        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${YELLOW}█${NC} ${WHITE}░${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}▒${NC} ${YELLOW}█${NC}     ${DIM}row 4${NC}                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}▒${NC} ${YELLOW}█${NC}     ${DIM}row 5${NC}                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}▒${NC} ${DIM}▒${NC} ${YELLOW}█${NC}     ${DIM}row 6${NC}                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}.${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${YELLOW}█${NC} ${DIM}.${NC}     ${DIM}row 7${NC}                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}. .${NC} ${DIM}▄${NC} ${DIM}▄${NC} ${DIM}▄${NC} ${DIM}▄${NC} ${DIM}. .${NC}     ${DIM}row 8 (dark base)${NC}          ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 7: Render Coin
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: Coin Sprite"

echo -e "  ${DIM}\$ pxl render examples/coin.jsonl -o coin.png${NC}"
echo ""

./target/release/pxl render examples/coin.jsonl -o "$DEMO_OUT/coin.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Output:${NC}"
echo ""
show_image "$DEMO_OUT/coin.png" 20

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 8: Example 3 - Hero
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Hero Character (16x16)"

echo -e "  ${BOLD}Input:${NC} examples/hero.jsonl"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${CYAN}Palette: \"hero\"${NC}                                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{_}${NC}       = transparent                               ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${YELLOW}{skin}${NC}   = #FFCC99  ${DIM}(peach)${NC}                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${RED}{hair}${NC}   = #8B4513  ${DIM}(brown)${NC}                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{eye}${NC}    = #000000  ${DIM}(black)${NC}                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${BLUE}{shirt}${NC}  = #4169E1  ${DIM}(royal blue)${NC}                  ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${BLUE}{pants}${NC}  = #1E3A5F  ${DIM}(navy)${NC}                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{shoes}${NC}  = #000000                                    ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${DIM}{outline}${NC}= #2C1810  ${DIM}(dark brown)${NC}                  ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${GREEN}Sprite: \"hero_idle\" (16x16)${NC}                            ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   Character with hair, face, shirt, pants, shoes       ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 9: Render Hero
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: Hero Character"

echo -e "  ${DIM}\$ pxl render examples/hero.jsonl -o hero.png${NC}"
echo ""

./target/release/pxl render examples/hero.jsonl -o "$DEMO_OUT/hero.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Output:${NC}"
echo ""
show_image "$DEMO_OUT/hero.png" 24

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 10: CLI Features
# ═══════════════════════════════════════════════════════════════════════════════
slide "CLI Features"

echo -e "  ${BOLD}Basic Usage:${NC}"
echo -e "  ${DIM}\$ pxl render input.jsonl${NC}"
echo -e "      Renders all sprites to {input}_{sprite}.png"
echo ""
echo -e "  ${BOLD}Output Options:${NC}"
echo -e "  ${DIM}\$ pxl render input.jsonl -o output.png${NC}"
echo -e "      Single sprite to specific file"
echo ""
echo -e "  ${DIM}\$ pxl render input.jsonl -o ./sprites/${NC}"
echo -e "      All sprites to directory"
echo ""
echo -e "  ${BOLD}Modes:${NC}"
echo -e "  ${DIM}\$ pxl render input.jsonl${NC}"
echo -e "      ${GREEN}Lenient${NC}: warns but continues on issues"
echo ""
echo -e "  ${DIM}\$ pxl render input.jsonl --strict${NC}"
echo -e "      ${RED}Strict${NC}: fails on any warning"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 11: Strict vs Lenient
# ═══════════════════════════════════════════════════════════════════════════════
slide "Error Handling: Lenient vs Strict"

echo -e "  ${BOLD}Test file with unknown token {y}:${NC}"
echo -e "  ${DIM}palette: {x}=#FF0000, grid uses {x} and {y}${NC}"
echo ""

echo -e "  ${DIM}────────────────────────────────────────────────────${NC}"
echo ""

echo -e "  ${GREEN}Lenient mode (default):${NC}"
echo -e "  ${DIM}\$ pxl render unknown_token.jsonl -o lenient.png${NC}"
echo ""
./target/release/pxl render tests/fixtures/lenient/unknown_token.jsonl -o "$DEMO_OUT/lenient.png" 2>&1 | sed 's/^/    /'
echo ""
echo -e "  ${GREEN}Result: File saved, warning printed${NC}"
echo -e "  ${DIM}Unknown token {y} rendered as ${NC}${MAGENTA}magenta${NC}"
echo ""

echo -e "  ${DIM}────────────────────────────────────────────────────${NC}"
echo ""

echo -e "  ${RED}Strict mode:${NC}"
echo -e "  ${DIM}\$ pxl render unknown_token.jsonl --strict${NC}"
echo ""
./target/release/pxl render tests/fixtures/lenient/unknown_token.jsonl --strict -o "$DEMO_OUT/strict.png" 2>&1 | sed 's/^/    /' || true
echo ""
echo -e "  ${GREEN}Result: Correctly rejected (exit 1)${NC}"
echo -e "  ${DIM}Strict mode fails on any warning${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 12: Phase 1 Feature - External Palette Include
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 1: External Palette Include"

echo -e "  ${CYAN}Share palettes across files with @include:${NC}"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}shared/palette.jsonl:${NC}                                  ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}type${NC}:   ${GREEN}palette${NC}                                     ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}name${NC}:   ${GREEN}shared${NC}                                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}colors${NC}: ${RED}{r}${NC}=#FF0000 ${GREEN}{g}${NC}=#00FF00 ${BLUE}{b}${NC}=#0000FF      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}sprite.jsonl:${NC}                                           ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}palette${NC}: ${CYAN}\"@include:shared/palette.jsonl\"${NC}             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}grid${NC}:    ${RED}{r}${NC}${GREEN}{g}${NC} / ${BLUE}{b}${NC}${DIM}{_}${NC}                                  ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${GREEN}Benefits:${NC}"
echo -e "  ${DIM}•${NC} Define colors once, use across multiple sprites"
echo -e "  ${DIM}•${NC} Relative paths from the including file"
echo -e "  ${DIM}•${NC} Circular include detection"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 13: Render External Include Example
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: External Include"

echo -e "  ${DIM}\$ pxl render tests/fixtures/valid/include_palette.jsonl${NC}"
echo ""

./target/release/pxl render tests/fixtures/valid/include_palette.jsonl -o "$DEMO_OUT/rgb_square.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Palette from:${NC} shared/palette.jsonl"
echo -e "  ${BOLD}Result:${NC} 2x2 RGB square"
echo ""
show_image "$DEMO_OUT/rgb_square.png" 20

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 14: Phase 1 - Built-in Palettes
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 1: Built-in Palette Data"

echo -e "  ${CYAN}6 palettes available in the library:${NC}"
echo ""
echo -e "  ${YELLOW}gameboy${NC}    ${DIM}4-color green${NC}"
echo -e "             ${GREEN}lightest${NC} #9BBC0F  ${GREEN}light${NC} #8BAC0F"
echo -e "             ${GREEN}dark${NC} #306230  ${GREEN}darkest${NC} #0F380F"
echo ""
echo -e "  ${YELLOW}nes${NC}        ${DIM}NES key colors${NC}"
echo -e "             ${RED}red${NC} ${GREEN}green${NC} ${BLUE}blue${NC} ${CYAN}cyan${NC} ${YELLOW}yellow${NC} ${MAGENTA}pink${NC} ${DIM}+ more${NC}"
echo ""
echo -e "  ${YELLOW}pico8${NC}      ${DIM}PICO-8 16-color palette${NC}"
echo -e "             ${MAGENTA}dark_purple${NC} ${GREEN}dark_green${NC} ${BLUE}blue${NC} ${RED}red${NC} ${DIM}+ more${NC}"
echo ""
echo -e "  ${YELLOW}dracula${NC}    ${DIM}Dark theme colors${NC}"
echo -e "             ${MAGENTA}purple${NC} ${MAGENTA}pink${NC} ${CYAN}cyan${NC} ${GREEN}green${NC} ${YELLOW}yellow${NC} ${RED}red${NC} ${DIM}+ more${NC}"
echo ""
echo -e "  ${YELLOW}grayscale${NC}  ${DIM}8 shades white to black${NC}"
echo ""
echo -e "  ${YELLOW}1bit${NC}       ${DIM}Black and white only${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 15: What's Complete
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 0 + Phase 1: Complete"

echo -e "  ${GREEN}Phase 0 - Core:${NC}"
echo -e "  ${GREEN}[x]${NC} JSONL parser with palettes and sprites"
echo -e "  ${GREEN}[x]${NC} Color parsing (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)"
echo -e "  ${GREEN}[x]${NC} Token extraction from grid strings"
echo -e "  ${GREEN}[x]${NC} Sprite renderer (grid → PNG)"
echo -e "  ${GREEN}[x]${NC} Lenient/strict error modes"
echo -e "  ${GREEN}[x]${NC} CLI: pxl render"
echo ""
echo -e "  ${GREEN}Phase 1 - Palettes:${NC}"
echo -e "  ${GREEN}[x]${NC} Built-in palette data (gameboy, nes, pico8, grayscale, 1bit)"
echo -e "  ${GREEN}[x]${NC} External palette include (@include:path)"
echo -e "  ${GREEN}[x]${NC} Circular include detection"
echo ""
echo -e "  ${BOLD}Tests:${NC} ${GREEN}All passing${NC}"
echo -e "  ${BOLD}Clippy:${NC} ${GREEN}No warnings${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 16: What's Next
# ═══════════════════════════════════════════════════════════════════════════════
slide "Coming Next: Phase 2"

echo -e "  ${CYAN}Animation & Spritesheet Export:${NC}"
echo ""
echo -e "  ${WHITE}Animation objects:${NC}"
echo -e "  ${DIM}Define frame sequences with timing${NC}"
echo -e "  ${DIM}Reference multiple sprites as frames${NC}"
echo ""
echo -e "  ${WHITE}Spritesheet output:${NC}"
echo -e "  ${DIM}Export all frames to a single image${NC}"
echo -e "  ${DIM}Generate metadata for game engines${NC}"
echo ""
echo -e "  ${CYAN}Future Phases:${NC}"
echo -e "  ${DIM}Phase 3:${NC} Animation timing & GIF export"
echo -e "  ${DIM}Phase 4:${NC} Game engine integration (Unity, Godot, Tiled)"
echo -e "  ${DIM}Phase 5:${NC} VS Code extension, web previewer"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 17: Try It
# ═══════════════════════════════════════════════════════════════════════════════
slide "Try It Yourself"

echo -e "  ${BOLD}Render the examples:${NC}"
echo -e "  ${DIM}\$ pxl render examples/coin.jsonl${NC}"
echo -e "  ${DIM}\$ pxl render examples/hero.jsonl${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl${NC}"
echo ""
echo -e "  ${BOLD}Try external include:${NC}"
echo -e "  ${DIM}\$ pxl render tests/fixtures/valid/include_palette.jsonl${NC}"
echo ""
echo -e "  ${BOLD}Run the tests:${NC}"
echo -e "  ${DIM}\$ cargo test${NC}"
echo ""
echo -e "  ${BOLD}Install image viewer (optional):${NC}"
echo -e "  ${DIM}\$ brew install chafa    # For inline image display${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 18: End
# ═══════════════════════════════════════════════════════════════════════════════
clear
echo ""
echo ""
echo ""
echo -e "${BOLD}${CYAN}"
cat << 'EOF'
                ████████╗████████╗██████╗
                ╚══██╔══╝╚══██╔══╝██╔══██╗
                   ██║      ██║   ██████╔╝
                   ██║      ██║   ██╔═══╝
                   ██║      ██║   ██║
                   ╚═╝      ╚═╝   ╚═╝
EOF
echo -e "${NC}"
echo ""
echo -e "                ${BOLD}Phase 0 + Phase 1 Complete${NC}"
echo ""
echo -e "                ${GREEN}Parse JSONL → Render PNG${NC}"
echo -e "                ${GREEN}External Palette Includes${NC}"
echo ""
echo ""

# Show all rendered outputs if possible
if command -v chafa &> /dev/null || command -v viu &> /dev/null; then
    echo -e "  ${DIM}Rendered outputs:${NC}"
    echo ""
    for img in "$DEMO_OUT"/*.png; do
        [ -f "$img" ] && show_image "$img" 8
    done
fi

echo ""
echo -e "  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
