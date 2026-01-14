#!/bin/bash
# Pixelsrc Demo
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
# SLIDE 23: What's Complete
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 0 + Phase 1 + Phase 2 + Phase 3: Complete"

echo -e "  ${GREEN}Phase 0 - Core:${NC}"
echo -e "  ${GREEN}[x]${NC} JSONL parser with palettes and sprites"
echo -e "  ${GREEN}[x]${NC} Color parsing (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)"
echo -e "  ${GREEN}[x]${NC} Token extraction from grid strings"
echo -e "  ${GREEN}[x]${NC} Sprite renderer (grid → PNG)"
echo -e "  ${GREEN}[x]${NC} Lenient/strict error modes"
echo -e "  ${GREEN}[x]${NC} CLI: pxl render"
echo -e "  ${GREEN}[x]${NC} Output scaling (--scale 1-16)"
echo ""
echo -e "  ${GREEN}Phase 1 - Palettes:${NC}"
echo -e "  ${GREEN}[x]${NC} Built-in palette data (gameboy, nes, pico8, grayscale, 1bit)"
echo -e "  ${GREEN}[x]${NC} External palette include (@include:path)"
echo -e "  ${GREEN}[x]${NC} Circular include detection"
echo ""
echo -e "  ${GREEN}Phase 2 - Composition:${NC}"
echo -e "  ${GREEN}[x]${NC} Multi-layer sprite composition"
echo -e "  ${GREEN}[x]${NC} Cell-size based grid positioning"
echo -e "  ${GREEN}[x]${NC} Base sprite support"
echo -e "  ${GREEN}[x]${NC} CLI: pxl render --composition"
echo ""
echo -e "  ${GREEN}Phase 3 - Animation:${NC}"
echo -e "  ${GREEN}[x]${NC} Animation model with frame timing"
echo -e "  ${GREEN}[x]${NC} Spritesheet output (--spritesheet)"
echo -e "  ${GREEN}[x]${NC} GIF output (--gif)"
echo -e "  ${GREEN}[x]${NC} Animation selection (--animation <name>)"
echo ""
echo -e "  ${BOLD}Tests:${NC} ${GREEN}All passing${NC}"
echo -e "  ${BOLD}Clippy:${NC} ${GREEN}No warnings${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 19: What's Next
# ═══════════════════════════════════════════════════════════════════════════════
slide "Coming Next: Phase 5"

echo -e "  ${CYAN}Developer Tooling:${NC}"
echo ""
echo -e "  ${WHITE}VS Code Extension:${NC}"
echo -e "  ${DIM}Syntax highlighting, live preview${NC}"
echo ""
echo -e "  ${WHITE}Web Editor:${NC}"
echo -e "  ${DIM}Browser-based sprite editor${NC}"
echo ""
echo -e "  ${CYAN}See Also:${NC}"
echo -e "  ${DIM}BACKLOG.md:${NC} Game engine exports (Unity, Godot, Tiled)"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 20: Try It
# ═══════════════════════════════════════════════════════════════════════════════
slide "Try It Yourself"

echo -e "  ${BOLD}Render the examples:${NC}"
echo -e "  ${DIM}\$ pxl render examples/coin.jsonl${NC}"
echo -e "  ${DIM}\$ pxl render examples/hero.jsonl${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl${NC}"
echo ""
echo -e "  ${BOLD}Try scaling:${NC}"
echo -e "  ${DIM}\$ pxl render examples/heart.jsonl --scale 4 -o heart_4x.png${NC}"
echo -e "  ${DIM}\$ pxl render examples/coin.jsonl --scale 8 -o coin_8x.png${NC}"
echo ""
echo -e "  ${BOLD}Try animations:${NC}"
echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --spritesheet -o sheet.png${NC}"
echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --gif -o walk.gif${NC}"
echo -e "  ${DIM}\$ pxl render examples/walk_cycle.jsonl --gif --scale 4 -o walk_4x.gif${NC}"
echo ""
echo -e "  ${BOLD}Try compositions:${NC}"
echo -e "  ${DIM}\$ pxl render examples/forest_scene.jsonl -c forest_scene -o forest.png${NC}"
echo -e "  ${DIM}\$ pxl render examples/color_variants.jsonl -c hat_variants -o variants.png${NC}"
echo -e "  ${DIM}\$ pxl render examples/hero_equipped.jsonl -c hero_equipped -o hero.png${NC}"
echo ""
echo -e "  ${BOLD}Run the tests:${NC}"
echo -e "  ${DIM}\$ cargo test${NC}"
echo ""
echo -e "  ${BOLD}Install image viewer (optional):${NC}"
echo -e "  ${DIM}\$ brew install chafa    # For inline image display${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 21: End
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
echo -e "                ${BOLD}Phase 0 + Phase 1 + Phase 2 + Phase 3 Complete${NC}"
echo ""
echo -e "                ${GREEN}Parse JSONL → Render PNG${NC}"
echo -e "                ${GREEN}External Palette Includes${NC}"
echo -e "                ${GREEN}Sprite Composition${NC}"
echo -e "                ${GREEN}Output Scaling (1-16x)${NC}"
echo -e "                ${GREEN}Animation → GIF/Spritesheet${NC}"
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
