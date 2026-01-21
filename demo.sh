#!/bin/bash
# Pixelsrc Demo
# Interactive slideshow demonstrating all implemented features

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
DEMO_OUT="/tmp/pixelsrc-demo"
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
        ██████╗ ██╗██╗  ██╗███████╗██╗     ███████╗██████╗  ██████╗
        ██╔══██╗██║╚██╗██╔╝██╔════╝██║     ██╔════╝██╔══██╗██╔════╝
        ██████╔╝██║ ╚███╔╝ █████╗  ██║     ███████╗██████╔╝██║
        ██╔═══╝ ██║ ██╔██╗ ██╔══╝  ██║     ╚════██║██╔══██╗██║
        ██║     ██║██╔╝ ██╗███████╗███████╗███████║██║  ██║╚██████╗
        ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═╝ ╚═════╝
EOF
echo -e "${NC}"
echo ""
echo -e "                ${BOLD}Pixelsrc${NC}"
echo -e "                ${DIM}GenAI-native pixel art format${NC}"
echo ""
echo ""
echo -e "                ${GREEN}Phases 0-12, 14-16 + CSS + Build System${NC}"
echo ""
echo ""
pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 2: Build
# ═══════════════════════════════════════════════════════════════════════════════
slide "Building Project"

echo -e "  ${DIM}\$ cargo build --release${NC}"
echo ""

# Run cargo build directly without piping to preserve real-time progress output
if cargo build --release; then
    echo ""
    echo -e "  ${GREEN}Build successful${NC}"
else
    echo ""
    echo -e "  ${RED}Build failed${NC}"
    exit 1
fi

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 3: Tests
# ═══════════════════════════════════════════════════════════════════════════════
slide "Running Tests"

echo -e "  ${DIM}\$ cargo test${NC}"
echo ""

# Run tests directly without piping to preserve real-time output
if cargo test; then
    echo ""
    echo -e "  ${GREEN}All tests passing${NC}"
else
    echo ""
    echo -e "  ${RED}Tests failed${NC}"
    exit 1
fi

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
echo -e "  ${BOLD}Scaling:${NC}"
echo -e "  ${DIM}\$ pxl render input.jsonl --scale 4${NC}"
echo -e "      Scale output by integer factor (1-16)"
echo ""
echo -e "  ${BOLD}Modes:${NC}"
echo -e "  ${DIM}\$ pxl render input.jsonl${NC}"
echo -e "      ${GREEN}Lenient${NC}: warns but continues on issues"
echo ""
echo -e "  ${DIM}\$ pxl render input.jsonl --strict${NC}"
echo -e "      ${RED}Strict${NC}: fails on any warning"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 11: Output Scaling
# ═══════════════════════════════════════════════════════════════════════════════
slide "Output Scaling"

echo -e "  ${CYAN}Scale output by integer factor (1-16):${NC}"
echo ""
echo -e "  ${BOLD}Default (1x):${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl -o heart_1x.png${NC}"
echo ""
./target/release/pxl render examples/heart.jsonl -o "$DEMO_OUT/heart_1x.png" 2>&1 | sed 's/^/  /'
echo ""
show_image "$DEMO_OUT/heart_1x.png" 10

echo ""
echo -e "  ${BOLD}Scaled 4x:${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl --scale 4 -o heart_4x.png${NC}"
echo ""
./target/release/pxl render examples/heart.jsonl --scale 4 -o "$DEMO_OUT/heart_4x.png" 2>&1 | sed 's/^/  /'
echo ""
show_image "$DEMO_OUT/heart_4x.png" 20

echo ""
echo -e "  ${GREEN}Benefits:${NC}"
echo -e "  ${DIM}•${NC} Nearest-neighbor interpolation preserves pixel-art crispness"
echo -e "  ${DIM}•${NC} Great for previews and social media"
echo -e "  ${DIM}•${NC} Works with PNG, GIF, and spritesheet output"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 12: Strict vs Lenient
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
# SLIDE 15: Phase 3 - Animation Demo
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 3: Animation"

echo -e "  ${CYAN}Multi-frame animations with timing:${NC}"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}walk_cycle.jsonl:${NC}                                      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}type${NC}:   ${GREEN}animation${NC}                                   ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}name${NC}:   ${GREEN}walk${NC}                                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}frames${NC}: ${CYAN}[\"walk_1\", \"walk_2\", \"walk_3\", \"walk_4\"]${NC}      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}duration${NC}: ${YELLOW}150${NC}  ${DIM}(ms per frame)${NC}                     ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}loop${NC}:   ${YELLOW}true${NC}                                        ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 16: Phase 3 - Spritesheet Output
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 3: Spritesheet Output"

echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --spritesheet -o sheet.png${NC}"
echo ""

./target/release/pxl render examples/walk_cycle.jsonl --spritesheet -o "$DEMO_OUT/walk_sheet.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Result:${NC} Horizontal strip of all animation frames"
echo ""
show_image "$DEMO_OUT/walk_sheet.png" 32

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 17: Phase 3 - GIF Output
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 3: GIF Output"

echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --gif -o walk.gif${NC}"
echo ""

./target/release/pxl render examples/walk_cycle.jsonl --gif -o "$DEMO_OUT/walk.gif" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Features:${NC}"
echo -e "  ${DIM}•${NC} Frame duration from animation (150ms)"
echo -e "  ${DIM}•${NC} Loop setting respected"
echo -e "  ${DIM}•${NC} Select specific animation with --animation <name>"
echo ""
echo -e "  ${BOLD}Result:${NC} $DEMO_OUT/walk.gif"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 17b: CSS Variables and color-mix()
# ═══════════════════════════════════════════════════════════════════════════════
slide "CSS Variables & color-mix()"

echo -e "  ${CYAN}Define base colors once, derive variants automatically:${NC}"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}CSS Variables:${NC}                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"--gold\"${NC}: ${YELLOW}\"#FFD700\"${NC}                               ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"{gold}\"${NC}: ${CYAN}\"var(--gold)\"${NC}                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}color-mix() for shadows:${NC}                                ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"{shadow}\"${NC}: ${MAGENTA}\"color-mix(in oklch, var(--gold) 70%, black)\"${NC}${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}color-mix() for highlights:${NC}                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"{shine}\"${NC}: ${MAGENTA}\"color-mix(in oklch, var(--gold) 60%, white)\"${NC}${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${GREEN}Benefits:${NC}"
echo -e "  ${DIM}•${NC} Define base colors once, auto-derive shadow/highlight"
echo -e "  ${DIM}•${NC} OKLCH color space for perceptually uniform blending"
echo -e "  ${DIM}•${NC} Easy theming: change --base, variants update automatically"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 17c: CSS Keyframes Animation
# ═══════════════════════════════════════════════════════════════════════════════
slide "CSS Keyframes Animation"

echo -e "  ${CYAN}Percentage-based animations with CSS timing:${NC}"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}type${NC}:   ${GREEN}animation${NC}                                     ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}name${NC}:   ${GREEN}coin_spin${NC}                                     ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}keyframes${NC}:                                              ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${CYAN}\"0%\"${NC}:   {\"sprite\": \"coin\"}                            ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${CYAN}\"50%\"${NC}:  {\"sprite\": \"coin\", \"transform\": \"scale(0.3,1)\"} ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${CYAN}\"100%\"${NC}: {\"sprite\": \"coin\"}                            ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}duration${NC}:       ${YELLOW}\"600ms\"${NC}                               ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}timing_function${NC}: ${YELLOW}\"ease-in-out\"${NC}                        ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${GREEN}Supported properties:${NC}"
echo -e "  ${DIM}•${NC} ${WHITE}sprite${NC}    - Change sprite at keyframe"
echo -e "  ${DIM}•${NC} ${WHITE}opacity${NC}   - Fade effects (0.0 to 1.0)"
echo -e "  ${DIM}•${NC} ${WHITE}transform${NC} - scale, rotate, translate, flip"
echo -e "  ${DIM}•${NC} ${WHITE}offset${NC}    - Position offset [x, y]"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 17d: Sprite Transforms (Derived Sprites)
# ═══════════════════════════════════════════════════════════════════════════════
slide "Sprite Transforms (Derived Sprites)"

echo -e "  ${CYAN}Create sprite variants from a source with transforms:${NC}"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}Source sprite:${NC}                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"name\"${NC}: ${GREEN}\"face_right\"${NC}                               ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}Derived sprites with transforms:${NC}                       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"source\"${NC}: ${GREEN}\"face_right\"${NC}                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"transform\"${NC}: ${CYAN}[\"mirror-h\"]${NC}        → face_left       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"transform\"${NC}: ${CYAN}[\"rotate:90\"]${NC}       → face_down       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"transform\"${NC}: ${CYAN}[\"scale:2,2\"]${NC}       → face_big        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}\"transform\"${NC}: ${CYAN}[{\"op\": \"sel-out\"}]${NC} → auto-outline    ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${GREEN}Transform operations:${NC}"
echo -e "  ${DIM}•${NC} ${WHITE}mirror-h${NC}  - Flip horizontally"
echo -e "  ${DIM}•${NC} ${WHITE}mirror-v${NC}  - Flip vertically"
echo -e "  ${DIM}•${NC} ${WHITE}rotate:N${NC}  - Rotate 90°, 180°, or 270°"
echo -e "  ${DIM}•${NC} ${WHITE}scale:x,y${NC} - Scale by factors"
echo -e "  ${DIM}•${NC} ${WHITE}sel-out${NC}   - Auto-outline based on fill colors"
echo -e "  ${DIM}•${NC} ${WHITE}dither${NC}    - Apply dither patterns"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 17e: Render Transforms Demo
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: Sprite Transforms"

echo -e "  ${DIM}\$ pxl render examples/transforms_demo.jsonl -s face_right -o face_right.png${NC}"
echo -e "  ${DIM}\$ pxl render examples/transforms_demo.jsonl -s face_left -o face_left.png${NC}"
echo ""

./target/release/pxl render examples/transforms_demo.jsonl -s face_right -o "$DEMO_OUT/face_right.png" 2>&1 | sed 's/^/  /'
./target/release/pxl render examples/transforms_demo.jsonl -s face_left -o "$DEMO_OUT/face_left.png" 2>&1 | sed 's/^/  /'
./target/release/pxl render examples/transforms_demo.jsonl -s face_outlined -o "$DEMO_OUT/face_outlined.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Original (face_right):${NC}"
show_image "$DEMO_OUT/face_right.png" 8
echo ""
echo -e "  ${BOLD}Mirrored (face_left):${NC}"
show_image "$DEMO_OUT/face_left.png" 8
echo ""
echo -e "  ${BOLD}Outlined (face_outlined):${NC}"
show_image "$DEMO_OUT/face_outlined.png" 8

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 18: Phase 2 - Composition
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 2: Composition"

echo -e "  ${CYAN}Layer sprites onto a canvas:${NC}"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${BOLD}Composition features:${NC}                                  ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}base${NC}:      Base sprite (rendered first)              ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}cell_size${NC}: Grid cell dimensions [w, h]              ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}sprites${NC}:   Map characters to sprite names           ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}layers${NC}:    Stack of grids (bottom to top)           ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${GREEN}Use cases:${NC}                                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   • Equip items on characters                           ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   • Build tile-based scenes                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   • Create color variants                               ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 19: Forest Scene Composition
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Forest Scene (Tile-Based Composition)"

echo -e "  ${BOLD}Input:${NC} examples/forest_scene.jsonl"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}cell_size${NC}: ${CYAN}[8, 8]${NC} - each character = 8x8 pixels       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}sprites${NC}:   ${GREEN}G${NC}=grass ${BLUE}W${NC}=water ${YELLOW}P${NC}=path ${GREEN}T${NC}=tree ${MAGENTA}F${NC}=flower ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} ${WHITE}layers${NC}:                                                ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   terrain: ${GREEN}G${NC}${GREEN}G${NC}${GREEN}G${NC}${BLUE}W${NC}${BLUE}W${NC}  ─┐                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ${GREEN}G${NC}${GREEN}G${NC}${YELLOW}P${NC}${BLUE}W${NC}${BLUE}W${NC}   │ 5×4 tile grid              ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ${GREEN}G${NC}${GREEN}G${NC}${YELLOW}P${NC}${GREEN}G${NC}${BLUE}W${NC}   │                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ${GREEN}G${NC}${MAGENTA}F${NC}${YELLOW}P${NC}${GREEN}G${NC}${GREEN}G${NC}  ─┘                             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   objects: ${GREEN}T${NC}..${GREEN}T${NC}.   (trees overlaid on terrain)       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ..${GREEN}T${NC}..                                       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            .....                                        ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}            ${GREEN}T${NC}...${GREEN}T${NC}                                       ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 20: Render Forest Scene
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: Forest Scene"

echo -e "  ${DIM}\$ pxl render examples/forest_scene.jsonl -c forest_scene -o forest.png${NC}"
echo ""

./target/release/pxl render examples/forest_scene.jsonl -c forest_scene -o "$DEMO_OUT/forest.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Output:${NC} 40×32 pixels (5×4 grid × 8×8 cell_size)"
echo ""
show_image "$DEMO_OUT/forest.png" 24

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 21: Color Variants
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Color Variants"

echo -e "  ${BOLD}Input:${NC} examples/color_variants.jsonl"
echo ""
echo -e "  ${DIM}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "  ${DIM}│${NC} Same sprite design with different palettes:             ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${RED}hat_red${NC}    palette: main=#CC0000, trim=#FFD700       ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${BLUE}hat_blue${NC}   palette: main=#0044CC, trim=#C0C0C0      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${GREEN}hat_green${NC}  palette: main=#228B22, trim=#FFD700      ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}                                                         ${DIM}│${NC}"
echo -e "  ${DIM}│${NC} Composition arranges them side by side:                 ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}cell_size${NC}: [12, 8]                                  ${DIM}│${NC}"
echo -e "  ${DIM}│${NC}   ${WHITE}map${NC}:       \"${RED}R${NC}${BLUE}B${NC}${GREEN}G${NC}\" (3 variants in a row)            ${DIM}│${NC}"
echo -e "  ${DIM}└─────────────────────────────────────────────────────────┘${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 22: Render Color Variants
# ═══════════════════════════════════════════════════════════════════════════════
slide "Rendering: Color Variants"

echo -e "  ${DIM}\$ pxl render examples/color_variants.jsonl -c hat_variants -o variants.png${NC}"
echo ""

./target/release/pxl render examples/color_variants.jsonl -c hat_variants -o "$DEMO_OUT/variants.png" 2>&1 | sed 's/^/  /'

echo ""
echo -e "  ${BOLD}Output:${NC} Three hat variants in a row"
echo ""
show_image "$DEMO_OUT/variants.png" 24

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 23: Phase 5 - CLI Extras
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 5: CLI Extras"

echo -e "  ${CYAN}PNG Import:${NC}"
echo -e "  ${DIM}\$ pxl import image.png -o sprite.jsonl${NC}"
echo -e "  ${DIM}Convert existing pixel art to Pixelsrc format${NC}"
echo ""
echo -e "  ${CYAN}Emoji Preview:${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl --emoji${NC}"
echo ""
./target/release/pxl render examples/heart.jsonl --emoji 2>&1 | sed 's/^/  /'
echo ""
echo -e "  ${CYAN}GenAI Prompts:${NC}"
echo -e "  ${DIM}\$ pxl prompts${NC}"
echo -e "  ${DIM}Show templates for AI-assisted sprite generation${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 24: Phase 14 - Corpus Analysis
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 14: Corpus Analysis"

echo -e "  ${CYAN}Analyze pixelsrc files for usage patterns:${NC}"
echo ""
echo -e "  ${DIM}\$ pxl analyze examples/*.jsonl${NC}"
echo ""
./target/release/pxl analyze examples/*.jsonl 2>&1 | head -20 | sed 's/^/  /'
echo ""
echo -e "  ${GREEN}Use cases:${NC}"
echo -e "  ${DIM}•${NC} Understand token frequency across a corpus"
echo -e "  ${DIM}•${NC} Identify common patterns"
echo -e "  ${DIM}•${NC} Data-driven format optimization"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 25: Phase 15 - AI Tools
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 15: AI Assistance Tools"

echo -e "  ${CYAN}pxl prime${NC} - Format guide for AI context injection"
echo -e "  ${DIM}\$ pxl prime --brief${NC}"
echo ""
./target/release/pxl prime --brief 2>&1 | head -10 | sed 's/^/  /'
echo ""
echo -e "  ${CYAN}pxl validate${NC} - Check for common mistakes"
echo -e "  ${DIM}\$ pxl validate sprite.jsonl${NC}"
echo ""
echo -e "  ${CYAN}pxl explain${NC} - Human-readable sprite explanation"
echo -e "  ${DIM}\$ pxl explain examples/heart.jsonl${NC}"
echo ""
./target/release/pxl explain examples/heart.jsonl 2>&1 | head -8 | sed 's/^/  /'

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 26: Phase 15 - More AI Tools
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 15: More AI Tools"

echo -e "  ${CYAN}pxl suggest${NC} - Suggest fixes for incomplete sprites"
echo -e "  ${DIM}\$ pxl suggest incomplete.jsonl${NC}"
echo ""
echo -e "  ${CYAN}pxl diff${NC} - Compare sprites semantically"
echo -e "  ${DIM}\$ pxl diff sprite1.jsonl sprite2.jsonl${NC}"
echo ""
echo -e "  ${GREEN}All tools support:${NC}"
echo -e "  ${DIM}•${NC} --json for machine-readable output"
echo -e "  ${DIM}•${NC} --stdin for piped input"
echo -e "  ${DIM}•${NC} Multiple file arguments"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 27: Terminal Display Commands
# ═══════════════════════════════════════════════════════════════════════════════
slide "Terminal Display Commands"

echo -e "  ${CYAN}pxl show${NC} - Display sprite with ANSI colors in terminal"
echo -e "  ${DIM}\$ pxl show examples/heart.jsonl -s heart${NC}"
echo ""
./target/release/pxl show examples/heart.jsonl -s heart 2>&1 | head -15 | sed 's/^/  /'
echo ""
echo -e "  ${CYAN}pxl grid${NC} - Display grid with row/column coordinates"
echo -e "  ${DIM}\$ pxl grid examples/coin.jsonl -s coin${NC}"
echo ""
./target/release/pxl grid examples/coin.jsonl -s coin 2>&1 | head -12 | sed 's/^/  /'

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 28: Editing & Formatting Commands
# ═══════════════════════════════════════════════════════════════════════════════
slide "Editing & Formatting Commands"

echo -e "  ${CYAN}pxl alias${NC} - Extract repeated patterns into single-letter aliases"
echo -e "  ${DIM}\$ pxl alias examples/heart.jsonl -s heart${NC}"
echo ""
./target/release/pxl alias examples/heart.jsonl -s heart 2>&1 | head -10 | sed 's/^/  /'
echo ""
echo -e "  ${CYAN}pxl inline${NC} - Expand grid with column-aligned spacing"
echo -e "  ${DIM}\$ pxl inline examples/coin.jsonl -s coin${NC}"
echo ""
./target/release/pxl inline examples/coin.jsonl -s coin 2>&1 | head -10 | sed 's/^/  /'

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 29: Transform & Palette Commands
# ═══════════════════════════════════════════════════════════════════════════════
slide "Transform & Palette Commands"

echo -e "  ${CYAN}pxl transform${NC} - Transform sprites (mirror, rotate, scale)"
echo -e "  ${DIM}\$ pxl transform examples/heart.jsonl -s heart --mirror-h${NC}"
echo ""
./target/release/pxl transform examples/heart.jsonl -s heart --mirror-h 2>&1 | head -8 | sed 's/^/  /'
echo ""
echo -e "  ${CYAN}pxl palette list${NC} - List built-in palettes"
echo -e "  ${DIM}\$ pxl palette list${NC}"
echo ""
./target/release/pxl palette list 2>&1 | head -10 | sed 's/^/  /'
echo ""
echo -e "  ${CYAN}pxl palette show${NC} - Show palette details"
echo -e "  ${DIM}\$ pxl palette show gameboy${NC}"
echo ""
./target/release/pxl palette show gameboy 2>&1 | head -8 | sed 's/^/  /'

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 30: Phase 16 - .pxl Format
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 16: .pxl Format & Formatting"

echo -e "  ${CYAN}.pxl file extension${NC} - More readable multi-line format"
echo -e "  ${DIM}Both .pxl and .jsonl are supported${NC}"
echo ""
echo -e "  ${CYAN}pxl fmt${NC} - Auto-format pixelsrc files"
echo -e "  ${DIM}\$ pxl fmt sprite.pxl${NC}           ${DIM}# Format in place${NC}"
echo -e "  ${DIM}\$ pxl fmt --check sprite.pxl${NC}   ${DIM}# Check only${NC}"
echo -e "  ${DIM}\$ pxl fmt --stdout sprite.pxl${NC}  ${DIM}# Print to stdout${NC}"
echo ""
echo -e "  ${GREEN}Benefits:${NC}"
echo -e "  ${DIM}•${NC} Improved readability with visual grid alignment"
echo -e "  ${DIM}•${NC} Consistent formatting across files"
echo -e "  ${DIM}•${NC} CI-friendly --check mode"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 28: What's Complete
# ═══════════════════════════════════════════════════════════════════════════════
slide "Implementation Status"

echo -e "  ${GREEN}Complete:${NC}"
echo -e "  ${GREEN}[x]${NC} Phase 0-5:   Core, Palettes, Composition, Animation, CLI Extras"
echo -e "  ${GREEN}[x]${NC} Phase 6-10:  WASM, Website, Obsidian, Packages, GitHub"
echo -e "  ${GREEN}[x]${NC} Phase 11:    Website Improvements (Dracula theme, a11y)"
echo -e "  ${GREEN}[x]${NC} Phase 12:    Composition Tiling"
echo -e "  ${GREEN}[x]${NC} Phase 14:    Corpus Analysis (pxl analyze)"
echo -e "  ${GREEN}[x]${NC} Phase 15:    AI Tools (prime, validate, suggest, diff, explain)"
echo -e "  ${GREEN}[x]${NC} Phase 16:    .pxl Format (pxl fmt)"
echo -e "  ${GREEN}[x]${NC} CSS:         Variables, color-mix(), keyframes, timing functions"
echo -e "  ${GREEN}[x]${NC} Build:       Project discovery, parallel builds, game engine exports"
echo ""
echo -e "  ${YELLOW}In Progress:${NC}"
echo -e "  ${YELLOW}[ ]${NC} Phase 13:    Theming & Branding (favicon, banners, social preview)"
echo -e "  ${YELLOW}[ ]${NC} Demo tests:  Example-driven test coverage"
echo ""
echo -e "  ${BOLD}Tests:${NC} ${GREEN}All passing${NC}"
echo -e "  ${BOLD}Clippy:${NC} ${GREEN}No warnings${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 29: What's Next
# ═══════════════════════════════════════════════════════════════════════════════
slide "Coming Next"

echo -e "  ${CYAN}Phase 13 - Branding:${NC}"
echo -e "  ${DIM}•${NC} Favicon (multiple sizes)"
echo -e "  ${DIM}•${NC} Social preview / Open Graph image"
echo -e "  ${DIM}•${NC} README and marketing banners"
echo ""
echo -e "  ${CYAN}Demo Tests:${NC}"
echo -e "  ${DIM}•${NC} Example-driven test coverage"
echo -e "  ${DIM}•${NC} CSS variables, timing, keyframes demos"
echo -e "  ${DIM}•${NC} Export format demos (Godot, Unity)"
echo ""
echo -e "  ${CYAN}Future Ideas:${NC}"
echo -e "  ${DIM}•${NC} VS Code Extension"
echo -e "  ${DIM}•${NC} Token Efficiency Optimizations"
echo -e "  ${DIM}•${NC} Palette Inheritance"
echo -e "  ${DIM}•${NC} More Built-in Palettes (Synthwave)"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 30: Try It
# ═══════════════════════════════════════════════════════════════════════════════
slide "Try It Yourself"

echo -e "  ${BOLD}Render sprites:${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl --scale 4 -o heart.png${NC}"
echo -e "  ${DIM}\$ pxl render examples/hero.jsonl --emoji${NC}"
echo ""
echo -e "  ${BOLD}Animations:${NC}"
echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --gif -o walk.gif${NC}"
echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --spritesheet -o sheet.png${NC}"
echo ""
echo -e "  ${BOLD}AI Tools:${NC}"
echo -e "  ${DIM}\$ pxl prime                    # Format guide for AI${NC}"
echo -e "  ${DIM}\$ pxl validate sprite.jsonl    # Check for mistakes${NC}"
echo -e "  ${DIM}\$ pxl explain examples/*.jsonl # Describe sprites${NC}"
echo -e "  ${DIM}\$ pxl analyze examples/        # Corpus metrics${NC}"
echo -e "  ${DIM}\$ pxl suggest sprite.jsonl     # Suggest fixes${NC}"
echo -e "  ${DIM}\$ pxl diff a.jsonl b.jsonl     # Compare sprites${NC}"
echo ""
echo -e "  ${BOLD}Display & Formatting:${NC}"
echo -e "  ${DIM}\$ pxl show sprite.jsonl -s x   # Terminal display (ANSI)${NC}"
echo -e "  ${DIM}\$ pxl grid sprite.jsonl -s x   # Grid with coordinates${NC}"
echo -e "  ${DIM}\$ pxl inline sprite.jsonl      # Column-aligned grid${NC}"
echo -e "  ${DIM}\$ pxl alias sprite.jsonl       # Extract aliases${NC}"
echo -e "  ${DIM}\$ pxl fmt sprite.pxl           # Auto-format${NC}"
echo -e "  ${DIM}\$ pxl fmt --check *.jsonl      # CI check${NC}"
echo ""
echo -e "  ${BOLD}Transform & Palettes:${NC}"
echo -e "  ${DIM}\$ pxl transform sprite.jsonl --mirror-h  # Transform${NC}"
echo -e "  ${DIM}\$ pxl palette list             # List built-in palettes${NC}"
echo -e "  ${DIM}\$ pxl palette show gameboy     # Show palette details${NC}"
echo ""
echo -e "  ${BOLD}Build System:${NC}"
echo -e "  ${DIM}\$ pxl build                    # Build all sprites in project${NC}"
echo -e "  ${DIM}\$ pxl build --parallel         # Parallel builds${NC}"
echo -e "  ${DIM}\$ pxl build --watch            # Watch for changes${NC}"
echo ""
echo -e "  ${BOLD}Website:${NC}"
echo -e "  ${DIM}https://scbrown.github.io/pixelsrc/${NC}"
echo ""
echo -e "  ${BOLD}Run tests:${NC}"
echo -e "  ${DIM}\$ cargo test${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 31: End
# ═══════════════════════════════════════════════════════════════════════════════
clear
echo ""
echo ""
echo ""
echo -e "${BOLD}${CYAN}"
cat << 'EOF'
        ██████╗ ██╗██╗  ██╗███████╗██╗     ███████╗██████╗  ██████╗
        ██╔══██╗██║╚██╗██╔╝██╔════╝██║     ██╔════╝██╔══██╗██╔════╝
        ██████╔╝██║ ╚███╔╝ █████╗  ██║     ███████╗██████╔╝██║
        ██╔═══╝ ██║ ██╔██╗ ██╔══╝  ██║     ╚════██║██╔══██╗██║
        ██║     ██║██╔╝ ██╗███████╗███████╗███████║██║  ██║╚██████╗
        ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═╝ ╚═════╝
EOF
echo -e "${NC}"
echo ""
echo -e "                ${BOLD}Phases 0-12, 14-16 + CSS + Build System Complete${NC}"
echo ""
echo -e "                ${GREEN}Core: Parse .pxl/.jsonl → Render PNG/GIF${NC}"
echo -e "                ${GREEN}CSS: Variables, color-mix(), keyframes, timing${NC}"
echo -e "                ${GREEN}Build: Project discovery, parallel builds, exports${NC}"
echo -e "                ${GREEN}AI: prime, validate, explain, analyze, suggest${NC}"
echo -e "                ${GREEN}Web: Website + Obsidian Plugin + WASM${NC}"
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
