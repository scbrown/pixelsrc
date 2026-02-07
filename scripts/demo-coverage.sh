#!/usr/bin/env bash
# =============================================================================
# Demo Coverage Script (DT-19)
#
# Checks demo test coverage against the feature checklist and reports
# uncovered features. Integrates with CI to warn on missing demos.
#
# Usage: ./scripts/demo-coverage.sh [options]
#
# Options:
#   --threshold N    Minimum coverage percentage required (default: 0)
#   --json           Output in JSON format
#   --ci             CI mode: exit non-zero if below threshold
#   --verbose        Show covered features too, not just missing
#   --help           Show this help
#
# Exit codes:
#   0  Coverage meets threshold
#   1  Coverage below threshold (only with --ci flag)
#   2  Script error
#
# Threshold Recommendations:
#   35%  - Informational (current baseline after DT-29)
#   50%  - Minimum viable for release
#   70%  - Target for Wave 2 completion
#   85%  - Target for Wave 3 completion
#   95%  - Full feature coverage goal
#
# Feature Categories (81 total):
#   Core Format:    sprites(8), transforms(6), animation(6), composition(5)
#   Palette:        palette-cycling(4)
#   I/O:            imports(4), exports(10)
#   Build:          build-system(5)
#   CLI:            cli-core(3), cli-format(3), cli-analysis(3),
#                   cli-project(3), cli-info(3)
#   CSS:            css-colors(7), css-variables(4), css-timing(3),
#                   css-keyframes(4)
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TESTS_DIR="$PROJECT_ROOT/tests/demos"

# Color output (if terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    BOLD=''
    NC=''
fi

# Configuration
THRESHOLD=0
JSON_OUTPUT=false
CI_MODE=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --threshold)
            THRESHOLD="$2"
            shift 2
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --ci)
            CI_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            sed -n '2,/^# ====/p' "$0" | grep "^#" | cut -c3-
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 2
            ;;
    esac
done

# =============================================================================
# Feature Registry
#
# Maps feature names to expected @demo paths.
# Format: CATEGORY|FEATURE_NAME|DEMO_PATH_PATTERN
#
# DEMO_PATH_PATTERN uses simple wildcards:
#   * matches any single segment
#   # matches anchor part
# =============================================================================

read -r -d '' FEATURE_REGISTRY << 'EOF' || true
# Sprites
sprites|Basic sprite|format/sprite#basic
sprites|Named palette reference|format/sprite#named_palette
sprites|Inline palette definition|format/sprite#inline_palette
sprites|Multi-character color keys|format/sprite#multichar_keys
sprites|Transparency|format/sprite#transparency
sprites|Origin point|format/sprite#origin
sprites|Collision boxes|format/sprite#collision
sprites|Attachment points|format/sprite#attachments

# Transforms
transforms|Horizontal flip|format/css/transforms#flip
transforms|Vertical flip|format/css/transforms#flip
transforms|Rotation|format/css/transforms#rotate
transforms|Scale|format/css/transforms#scale
transforms|Translate|format/css/transforms#translate
transforms|Recolor (palette swap)|format/sprite#recolor

# Animation
animation|Basic frame sequence|format/animation#basic
animation|Frame timing (FPS)|format/animation#fps
animation|Frame tags|format/animation#tags
animation|Looping modes|format/animation#looping
animation|Attachment chains|format/animation#attachments
animation|Frame-specific metadata|format/animation#frame_metadata

# Composition
composition|Basic layer stacking|format/composition#basic
composition|Layer positioning|format/composition#positioning
composition|Blend modes|format/composition#blend
composition|Background fills|format/composition#fills
composition|Multi-sprite scenes|format/composition#multi_sprite

# Palette Cycling
palette-cycling|Single color cycle|format/palette#cycle_single
palette-cycling|Multiple cycle groups|format/palette#cycle_multiple
palette-cycling|Cycle timing|format/palette#cycle_timing
palette-cycling|Ping-pong mode|format/palette#cycle_pingpong

# Imports
imports|PNG to JSONL conversion|cli/import#png
imports|Palette detection|cli/import#palette_detect
imports|Multi-frame import|cli/import#multi_frame
imports|Transparent color handling|cli/import#transparency

# Exports
exports|PNG basic|export/png#basic
exports|PNG scaled|export/png#scaled
exports|GIF animated|export/gif#animated
exports|Spritesheet horizontal|export/spritesheet#horizontal
exports|Spritesheet grid|export/spritesheet#grid
exports|Spritesheet padding|export/spritesheet#padding
exports|Atlas Godot|export/atlas#godot
exports|Atlas Unity|export/atlas#unity
exports|Atlas LibGDX|export/atlas#libgdx
exports|Atlas Aseprite|export/atlas#aseprite

# Build System
build-system|Basic pxl.toml configuration|cli/build#basic
build-system|Multi-target builds|cli/build#multi_target
build-system|Incremental rebuilds|cli/build#incremental
build-system|Watch mode|cli/build#watch
build-system|Build variants|cli/build#variants

# CLI Core
cli-core|render command|cli/core#render
cli-core|import command|cli/core#import
cli-core|validate command|cli/core#validate

# CLI Format
cli-format|fmt command|cli/format#fmt
cli-format|show command|cli/format#show
cli-format|explain command|cli/format#explain

# CLI Analysis
cli-analysis|diff command|cli/analysis#diff
cli-analysis|suggest command|cli/analysis#suggest
cli-analysis|analyze command|cli/analysis#analyze

# CLI Project
cli-project|build command|cli/project#build
cli-project|new command|cli/project#new
cli-project|init command|cli/project#init

# CLI Info
cli-info|prime command|cli/info#prime
cli-info|prompts command|cli/info#prompts
cli-info|palettes command|cli/info#palettes

# CSS Colors
css-colors|Hex colors|format/css/colors#hex
css-colors|RGB colors|format/css/colors#rgb
css-colors|HSL colors|format/css/colors#hsl
css-colors|OKLCH colors|format/css/colors#oklch
css-colors|HWB colors|format/css/colors#hwb
css-colors|Named colors|format/css/colors#named
css-colors|Color-mix|format/css/colors#color_mix

# CSS Variables
css-variables|Variable definition|format/css/variables#definition
css-variables|Variable resolution|format/css/variables#resolution
css-variables|Variable fallbacks|format/css/variables#fallbacks
css-variables|Variable chaining|format/css/variables#chaining

# CSS Timing
css-timing|Named timing functions|format/css/timing#named
css-timing|Cubic-bezier|format/css/timing#cubic_bezier
css-timing|Steps|format/css/timing#steps

# CSS Keyframes
css-keyframes|Percentage keyframes|format/css/keyframes#percentage
css-keyframes|From/to keyframes|format/css/keyframes#from_to
css-keyframes|Sprite changes|format/css/keyframes#sprite_changes
css-keyframes|Transform keyframes|format/css/keyframes#transforms
EOF

# =============================================================================
# Functions
# =============================================================================

# Extract all @demo paths from test files
get_existing_demos() {
    grep -rh '/// @demo ' "$TESTS_DIR" 2>/dev/null | \
        sed 's/.*@demo //' | \
        tr -d ' ' | \
        sort -u
}

# Check if a demo path exists (exact match or pattern match)
demo_exists() {
    local pattern="$1"
    local demos="$2"

    # Direct exact match
    if echo "$demos" | grep -qx "$pattern"; then
        return 0
    fi

    # For flip, check if any flip demo exists (covers both horizontal and vertical)
    if [[ "$pattern" == *"#flip"* ]]; then
        if echo "$demos" | grep -q "transforms#flip"; then
            return 0
        fi
    fi

    return 1
}

# Generate coverage report
generate_report() {
    local demos
    demos=$(get_existing_demos)

    local total=0
    local covered=0
    local missing=()
    local found=()

    # Process feature registry
    while IFS= read -r line; do
        # Skip comments and empty lines
        [[ -z "$line" || "$line" == \#* ]] && continue

        IFS='|' read -r category feature pattern <<< "$line"
        total=$((total + 1))

        if demo_exists "$pattern" "$demos"; then
            covered=$((covered + 1))
            found+=("$category|$feature|$pattern")
        else
            missing+=("$category|$feature|$pattern")
        fi
    done <<< "$FEATURE_REGISTRY"

    # Calculate percentage
    local percentage=0
    if [[ $total -gt 0 ]]; then
        percentage=$((covered * 100 / total))
    fi

    # Output
    if [[ "$JSON_OUTPUT" == "true" ]]; then
        output_json "$total" "$covered" "$percentage" "${missing[@]}" "${found[@]}"
    else
        output_text "$total" "$covered" "$percentage" "${missing[@]}" "${found[@]}"
    fi

    # Return exit code
    if [[ "$CI_MODE" == "true" && $percentage -lt $THRESHOLD ]]; then
        return 1
    fi
    return 0
}

output_text() {
    local total="$1"
    local covered="$2"
    local percentage="$3"
    shift 3

    # Split remaining args into missing and found
    local missing=()
    local found=()
    local in_found=false
    for arg in "$@"; do
        if [[ "$arg" == "---FOUND---" ]]; then
            in_found=true
            continue
        fi
        if [[ "$in_found" == "true" ]]; then
            found+=("$arg")
        else
            missing+=("$arg")
        fi
    done

    echo -e "${BOLD}Demo Coverage Report${NC}"
    echo "===================="
    echo ""

    # Summary
    local color="$GREEN"
    if [[ $percentage -lt 50 ]]; then
        color="$RED"
    elif [[ $percentage -lt 80 ]]; then
        color="$YELLOW"
    fi

    echo -e "Coverage: ${color}${BOLD}${percentage}%${NC} (${covered}/${total} features)"
    echo ""

    # Missing demos by category
    if [[ ${#missing[@]} -gt 0 ]]; then
        echo -e "${YELLOW}Missing Demo Coverage:${NC}"
        echo ""

        local current_category=""
        for item in "${missing[@]}"; do
            IFS='|' read -r category feature pattern <<< "$item"
            if [[ "$category" != "$current_category" ]]; then
                current_category="$category"
                echo -e "  ${CYAN}${category}${NC}"
            fi
            echo -e "    ${RED}✗${NC} $feature"
            echo -e "      ${BLUE}→ Expected: @demo $pattern${NC}"
        done
        echo ""
    fi

    # Covered demos (verbose mode)
    if [[ "$VERBOSE" == "true" && ${#found[@]} -gt 0 ]]; then
        echo -e "${GREEN}Covered Features:${NC}"
        echo ""

        local current_category=""
        for item in "${found[@]}"; do
            IFS='|' read -r category feature pattern <<< "$item"
            if [[ "$category" != "$current_category" ]]; then
                current_category="$category"
                echo -e "  ${CYAN}${category}${NC}"
            fi
            echo -e "    ${GREEN}✓${NC} $feature"
        done
        echo ""
    fi

    # Threshold warning
    if [[ $THRESHOLD -gt 0 ]]; then
        if [[ $percentage -ge $THRESHOLD ]]; then
            echo -e "${GREEN}✓ Coverage meets threshold (${THRESHOLD}%)${NC}"
        else
            echo -e "${RED}✗ Coverage below threshold (${percentage}% < ${THRESHOLD}%)${NC}"
        fi
    fi
}

output_json() {
    local total="$1"
    local covered="$2"
    local percentage="$3"
    shift 3

    # Collect missing and found
    local missing_json="["
    local found_json="["
    local first_missing=true
    local first_found=true
    local in_found=false

    for arg in "$@"; do
        if [[ "$arg" == "---FOUND---" ]]; then
            in_found=true
            continue
        fi

        IFS='|' read -r category feature pattern <<< "$arg"
        local item="{\"category\":\"$category\",\"feature\":\"$feature\",\"expected_demo\":\"$pattern\"}"

        if [[ "$in_found" == "true" ]]; then
            if [[ "$first_found" == "true" ]]; then
                first_found=false
            else
                found_json+=","
            fi
            found_json+="$item"
        else
            if [[ "$first_missing" == "true" ]]; then
                first_missing=false
            else
                missing_json+=","
            fi
            missing_json+="$item"
        fi
    done

    missing_json+="]"
    found_json+="]"

    cat << EOF
{
  "total_features": $total,
  "covered_features": $covered,
  "coverage_percentage": $percentage,
  "threshold": $THRESHOLD,
  "passes_threshold": $([ $percentage -ge $THRESHOLD ] && echo "true" || echo "false"),
  "missing": $missing_json,
  "covered": $found_json
}
EOF
}

# Fix the generate_report to pass markers between missing and found
generate_report() {
    local demos
    demos=$(get_existing_demos)

    local total=0
    local covered=0
    local missing=()
    local found=()

    # Process feature registry
    while IFS= read -r line; do
        # Skip comments and empty lines
        [[ -z "$line" || "$line" == \#* ]] && continue

        IFS='|' read -r category feature pattern <<< "$line"
        total=$((total + 1))

        if demo_exists "$pattern" "$demos"; then
            covered=$((covered + 1))
            found+=("$category|$feature|$pattern")
        else
            missing+=("$category|$feature|$pattern")
        fi
    done <<< "$FEATURE_REGISTRY"

    # Calculate percentage
    local percentage=0
    if [[ $total -gt 0 ]]; then
        percentage=$((covered * 100 / total))
    fi

    # Output with marker between arrays
    if [[ "$JSON_OUTPUT" == "true" ]]; then
        output_json "$total" "$covered" "$percentage" "${missing[@]}" "---FOUND---" "${found[@]}"
    else
        output_text "$total" "$covered" "$percentage" "${missing[@]}" "---FOUND---" "${found[@]}"
    fi

    # Return exit code
    if [[ "$CI_MODE" == "true" && $percentage -lt $THRESHOLD ]]; then
        return 1
    fi
    return 0
}

# =============================================================================
# Main
# =============================================================================

main() {
    if [[ ! -d "$TESTS_DIR" ]]; then
        echo "Error: Tests directory not found: $TESTS_DIR" >&2
        exit 2
    fi

    generate_report
}

main "$@"
