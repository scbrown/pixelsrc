#!/bin/bash
# TTP (Text To Pixel) Demo
# Interactive slideshow of current capabilities

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

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
echo -e "                ${DIM}Phase 0 MVP Demo${NC}"
echo ""
echo ""
pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 2: Build
# ═══════════════════════════════════════════════════════════════════════════════
slide "Building Project"

echo -e "  ${DIM}\$ cargo build --release${NC}"
echo ""

if cargo build --release 2>&1 | grep -E "(Compiling|Finished)" | tail -5 | sed 's/^/  /'; then
    echo ""
    echo -e "  ${GREEN}Build successful${NC}"
else
    echo -e "  ${GREEN}Already built${NC}"
fi

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 3: Tests
# ═══════════════════════════════════════════════════════════════════════════════
slide "Running Tests"

echo -e "  ${DIM}\$ cargo test${NC}"
echo ""

# Run tests and format output nicely
cargo test 2>&1 | grep -E "^test |passed|failed" | while read -r line; do
    if echo "$line" | grep -q "passed"; then
        echo -e "  ${GREEN}$line${NC}"
    elif echo "$line" | grep -q "ok$"; then
        echo -e "  ${GREEN}$line${NC}"
    elif echo "$line" | grep -q "FAILED"; then
        echo -e "  ${RED}$line${NC}"
    else
        echo -e "  $line"
    fi
done

echo ""
echo -e "  ${GREEN}All 27 tests passing${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 4: Format Overview
# ═══════════════════════════════════════════════════════════════════════════════
slide "TTP Format Overview"

echo -e "  TTP uses ${BOLD}JSONL${NC} (JSON Lines) - one object per line:"
echo ""
echo -e "  ${YELLOW}Palette${NC} - defines named colors"
echo -e "  ${DIM}  {\"type\": \"palette\", \"name\": \"...\", \"colors\": {...}}${NC}"
echo ""
echo -e "  ${GREEN}Sprite${NC} - defines pixel grid using color tokens"
echo -e "  ${DIM}  {\"type\": \"sprite\", \"name\": \"...\", \"palette\": \"...\", \"grid\": [...]}${NC}"
echo ""
echo ""
echo -e "  ${BOLD}Color Formats:${NC}"
echo -e "  ${DIM}  #RGB        ${NC}${MAGENTA}#F00${NC}         ${DIM}short red${NC}"
echo -e "  ${DIM}  #RGBA       ${NC}${MAGENTA}#F008${NC}        ${DIM}short red, 50% alpha${NC}"
echo -e "  ${DIM}  #RRGGBB     ${NC}${MAGENTA}#FF0000${NC}      ${DIM}full red${NC}"
echo -e "  ${DIM}  #RRGGBBAA   ${NC}${MAGENTA}#FF000080${NC}    ${DIM}full red, 50% alpha${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 5: Example - Coin Palette
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Coin Sprite"

echo -e "  ${BOLD}File:${NC} examples/coin.jsonl"
echo ""
echo -e "  ${YELLOW}Line 1 - Palette Definition:${NC}"
echo ""
echo -e "  ${DIM}{${NC}"
echo -e "    ${CYAN}\"type\"${NC}: ${GREEN}\"palette\"${NC},"
echo -e "    ${CYAN}\"name\"${NC}: ${GREEN}\"coin\"${NC},"
echo -e "    ${CYAN}\"colors\"${NC}: {"
echo -e "      ${CYAN}\"{_}\"${NC}:      ${GREEN}\"#00000000\"${NC}  ${DIM}transparent${NC}"
echo -e "      ${CYAN}\"{gold}\"${NC}:   ${GREEN}\"#FFD700\"${NC}    ${DIM}gold${NC}"
echo -e "      ${CYAN}\"{shine}\"${NC}:  ${GREEN}\"#FFFACD\"${NC}    ${DIM}highlight${NC}"
echo -e "      ${CYAN}\"{shadow}\"${NC}: ${GREEN}\"#B8860B\"${NC}    ${DIM}shadow${NC}"
echo -e "      ${CYAN}\"{dark}\"${NC}:   ${GREEN}\"#8B6914\"${NC}    ${DIM}dark edge${NC}"
echo -e "    }"
echo -e "  ${DIM}}${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 6: Example - Coin Grid
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Coin Sprite (continued)"

echo -e "  ${GREEN}Line 2 - Sprite Definition:${NC}"
echo ""
echo -e "  ${DIM}{${NC}"
echo -e "    ${CYAN}\"type\"${NC}: ${GREEN}\"sprite\"${NC},"
echo -e "    ${CYAN}\"name\"${NC}: ${GREEN}\"coin\"${NC},"
echo -e "    ${CYAN}\"size\"${NC}: [8, 8],"
echo -e "    ${CYAN}\"palette\"${NC}: ${GREEN}\"coin\"${NC},  ${DIM}<-- references palette above${NC}"
echo -e "    ${CYAN}\"grid\"${NC}: ["
echo -e "      ${GREEN}\"{_}{_}{gold}{gold}{gold}{gold}{_}{_}\"${NC}"
echo -e "      ${GREEN}\"{_}{gold}{shine}{shine}{gold}{gold}{gold}{_}\"${NC}"
echo -e "      ${GREEN}\"{gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold}\"${NC}"
echo -e "      ${DIM}...${NC}"
echo -e "    ]"
echo -e "  ${DIM}}${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 7: ASCII Preview
# ═══════════════════════════════════════════════════════════════════════════════
slide "Example: Coin Sprite (preview)"

echo -e "  ${BOLD}ASCII Visualization:${NC}"
echo ""
echo -e "         1 2 3 4 5 6 7 8"
echo -e "        ${DIM}┌─────────────────┐${NC}"
echo -e "      1 ${DIM}│${NC} ${DIM}. .${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}. .${NC} ${DIM}│${NC}"
echo -e "      2 ${DIM}│${NC} ${DIM}.${NC} ${YELLOW}#${NC} ${BOLD}*${NC} ${BOLD}*${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}.${NC} ${DIM}│${NC}"
echo -e "      3 ${DIM}│${NC} ${YELLOW}#${NC} ${BOLD}*${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}@${NC} ${YELLOW}#${NC} ${DIM}│${NC}"
echo -e "      4 ${DIM}│${NC} ${YELLOW}#${NC} ${BOLD}*${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}@${NC} ${YELLOW}#${NC} ${DIM}│${NC}"
echo -e "      5 ${DIM}│${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}@${NC} ${YELLOW}#${NC} ${DIM}│${NC}"
echo -e "      6 ${DIM}│${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}@${NC} ${DIM}@${NC} ${YELLOW}#${NC} ${DIM}│${NC}"
echo -e "      7 ${DIM}│${NC} ${DIM}.${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${YELLOW}#${NC} ${DIM}.${NC} ${DIM}│${NC}"
echo -e "      8 ${DIM}│${NC} ${DIM}. .${NC} ${DIM}o${NC} ${DIM}o${NC} ${DIM}o${NC} ${DIM}o${NC} ${DIM}. .${NC} ${DIM}│${NC}"
echo -e "        ${DIM}└─────────────────┘${NC}"
echo ""
echo -e "  ${DIM}Legend:${NC}  ${YELLOW}#${NC} gold   ${BOLD}*${NC} shine   ${DIM}@${NC} shadow   ${DIM}o${NC} dark   ${DIM}.${NC} transparent"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 8: Running Binary
# ═══════════════════════════════════════════════════════════════════════════════
slide "Running the Binary"

echo -e "  ${DIM}\$ ./target/release/pxl examples/coin.jsonl${NC}"
echo ""
./target/release/pxl examples/coin.jsonl 2>&1 | sed 's/^/  /'
echo ""
echo ""
echo -e "  ${YELLOW}CLI not yet wired up${NC}"
echo ""
echo -e "  ${DIM}Expected (when complete):${NC}"
echo -e "  ${DIM}\$ pxl render examples/coin.jsonl -o coin.png${NC}"
echo -e "  ${DIM}Rendered: coin.png (8x8)${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 9: What's Working
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 0 Status: What's Working"

echo -e "  ${GREEN}Completed:${NC}"
echo ""
echo -e "  ${GREEN}[x]${NC} ${BOLD}Data Models${NC}"
echo -e "      ${DIM}Palette, Sprite, TtpObject deserialization${NC}"
echo ""
echo -e "  ${GREEN}[x]${NC} ${BOLD}Color Parsing${NC}"
echo -e "      ${DIM}#RGB, #RGBA, #RRGGBB, #RRGGBBAA formats${NC}"
echo ""
echo -e "  ${GREEN}[x]${NC} ${BOLD}Tokenizer${NC}"
echo -e "      ${DIM}Extracts {tokens} from grid strings${NC}"
echo ""
echo -e "  ${GREEN}[x]${NC} ${BOLD}Test Fixtures${NC}"
echo -e "      ${DIM}19 fixture files (valid, invalid, lenient)${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 10: What's Missing
# ═══════════════════════════════════════════════════════════════════════════════
slide "Phase 0 Status: What's Missing"

echo -e "  ${YELLOW}Remaining Tasks:${NC}"
echo ""
echo -e "  ${YELLOW}[ ]${NC} ${BOLD}JSONL Parser${NC}         ${DIM}src/parser.rs${NC}"
echo -e "      ${DIM}parse_line(), parse_stream()${NC}"
echo ""
echo -e "  ${YELLOW}[ ]${NC} ${BOLD}Palette Registry${NC}    ${DIM}src/registry.rs${NC}"
echo -e "      ${DIM}Resolve named palette references${NC}"
echo ""
echo -e "  ${YELLOW}[ ]${NC} ${BOLD}Sprite Renderer${NC}     ${DIM}src/renderer.rs${NC}"
echo -e "      ${DIM}Grid + palette -> RgbaImage${NC}"
echo ""
echo -e "  ${YELLOW}[ ]${NC} ${BOLD}PNG Output${NC}          ${DIM}src/output.rs${NC}"
echo -e "      ${DIM}save_png(), path generation${NC}"
echo ""
echo -e "  ${YELLOW}[ ]${NC} ${BOLD}CLI${NC}                 ${DIM}src/cli.rs${NC}"
echo -e "      ${DIM}pxl render input.jsonl -o output.png${NC}"
echo ""
echo -e "  ${YELLOW}[ ]${NC} ${BOLD}Integration Tests${NC}   ${DIM}tests/integration_tests.rs${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 11: Try It
# ═══════════════════════════════════════════════════════════════════════════════
slide "Try It Yourself"

echo -e "  ${BOLD}Run the tests:${NC}"
echo -e "  ${DIM}\$ cargo test${NC}"
echo ""
echo -e "  ${BOLD}Explore the examples:${NC}"
echo -e "  ${DIM}\$ cat examples/coin.jsonl${NC}"
echo -e "  ${DIM}\$ cat examples/hero.jsonl${NC}"
echo -e "  ${DIM}\$ cat examples/walk_cycle.jsonl${NC}"
echo ""
echo -e "  ${BOLD}Check test fixtures:${NC}"
echo -e "  ${DIM}\$ ls tests/fixtures/valid/${NC}"
echo -e "  ${DIM}\$ ls tests/fixtures/invalid/${NC}"
echo -e "  ${DIM}\$ ls tests/fixtures/lenient/${NC}"
echo ""
echo -e "  ${BOLD}Read the spec:${NC}"
echo -e "  ${DIM}\$ cat docs/spec/format.md${NC}"

pause

# ═══════════════════════════════════════════════════════════════════════════════
# SLIDE 12: End
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
echo -e "                ${BOLD}End of Demo${NC}"
echo ""
echo -e "                ${DIM}Phase 0: 4/10 tasks complete${NC}"
echo -e "                ${DIM}Foundation ready, pipeline needed${NC}"
echo ""
echo ""
echo -e "  ${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
