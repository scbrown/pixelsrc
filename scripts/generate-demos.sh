#!/usr/bin/env bash
# =============================================================================
# Demo Generator Script (DT-16, DT-17, DT-18)
#
# Parses @demo annotations from tests/demos/**/*.rs, extracts JSONL content,
# and generates markdown documentation fragments.
#
# Usage: ./scripts/generate-demos.sh [OPTIONS]
#
# Options:
#   --dry-run         Show what would be generated without writing files
#   --book            Output to docs/book/src/demos/ (for mdbook integration)
#   --output-dir DIR  Output to specified directory (default: target/demos)
#   --check           Verify output matches existing files (for CI regression)
#
# Annotations supported:
#   /// @demo section/subsection#anchor
#   /// @title Demo Title
#   /// @description Description text.
#   /// @cli pxl render example.jsonl -o output.png
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TESTS_DIR="$PROJECT_ROOT/tests/demos"
EXAMPLES_DIR="$PROJECT_ROOT/examples/demos"
OUTPUT_DIR="$PROJECT_ROOT/target/demos"
BOOK_DIR="$PROJECT_ROOT/docs/book/src/demos"

# Color output (if terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

DRY_RUN=false
CHECK_MODE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --book)
            OUTPUT_DIR="$BOOK_DIR"
            shift
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --check)
            CHECK_MODE=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Parse a single test file and extract demo metadata
# Output: TAB-separated records for each demo found
# Uses external AWK script for BSD awk compatibility
parse_test_file() {
    local file="$1"
    awk -f "$SCRIPT_DIR/parse_demos.awk" "$file"
}

# Generate markdown for a demo
generate_demo_markdown() {
    local demo="$1"
    local title="$2"
    local description="$3"
    local cli="$4"
    local jsonl_path="$5"

    # Replace <EMPTY> placeholders with empty string
    [[ "$title" == "<EMPTY>" ]] && title=""
    [[ "$description" == "<EMPTY>" ]] && description=""
    [[ "$cli" == "<EMPTY>" ]] && cli=""

    local jsonl_file="$PROJECT_ROOT/$jsonl_path"

    if [[ ! -f "$jsonl_file" ]]; then
        log_warn "JSONL file not found: $jsonl_path"
        return 1
    fi

    # Extract anchor from demo path (format: section/subsection#anchor)
    local anchor="${demo##*#}"
    local section="${demo%#*}"

    echo "## $title"
    echo ""
    if [[ -n "$description" ]]; then
        echo "$description"
        echo ""
    fi

    echo '<div class="demo-source">'
    echo ""
    echo '```jsonl'
    cat "$jsonl_file"
    echo '```'
    echo ""
    echo '</div>'
    echo ""

    echo "<div class=\"demo-container\" data-demo=\"$anchor\">"
    echo '</div>'
    echo ""

    if [[ -n "$cli" ]]; then
        echo '**CLI equivalent:**'
        echo '```bash'
        echo "$cli"
        echo '```'
        echo ""
    fi
}

# Get unique sections from demo data
get_sections() {
    local demos_data="$1"
    echo "$demos_data" | while IFS=$'\t' read -r marker demo title description cli jsonl_path; do
        [[ "$marker" != "DEMO" ]] && continue
        # Extract section (everything before #anchor)
        echo "${demo%#*}"
    done | sort -u
}

# Get demos for a specific section
# Note: Keeps <EMPTY> placeholders intact for bash read compatibility
get_demos_for_section() {
    local demos_data="$1"
    local target_section="$2"
    echo "$demos_data" | while IFS=$'\t' read -r marker demo title description cli jsonl_path; do
        [[ "$marker" != "DEMO" ]] && continue
        local section="${demo%#*}"
        if [[ "$section" == "$target_section" ]]; then
            printf "%s\t%s\t%s\t%s\t%s\n" "$demo" "$title" "$description" "$cli" "$jsonl_path"
        fi
    done
}

# Map section path to book-friendly filename
get_book_filename() {
    local section="$1"
    case "$section" in
        format/css/colors)     echo "colors.md" ;;
        format/css/variables)  echo "variables.md" ;;
        format/css/timing)     echo "timing.md" ;;
        format/css/keyframes)  echo "keyframes.md" ;;
        format/css/transforms) echo "transforms.md" ;;
        export/spritesheet)    echo "spritesheets.md" ;;
        *)                     echo "$(echo "$section" | tr '/' '_').md" ;;
    esac
}

# Group demos by section and generate output files
process_demos() {
    local demos_data="$1"

    # Get unique sections
    local sections
    sections=$(get_sections "$demos_data")

    # Generate output for each section
    while IFS= read -r section; do
        [[ -z "$section" ]] && continue

        # Convert section path to file path
        local output_file
        if [[ "$BOOK_MODE" == "true" ]]; then
            output_file=$(get_book_filename "$section")
        else
            output_file=$(echo "$section" | tr '/' '_').md
        fi
        local output_path="$OUTPUT_DIR/$output_file"

        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "[DRY-RUN] Would generate: $output_path"
            continue
        fi

        log_info "Generating: $output_path"

        # Generate page title based on section
        local page_title
        case "$section" in
            format/css/colors)     page_title="CSS Color Demos" ;;
            format/css/variables)  page_title="CSS Variable Demos" ;;
            format/css/timing)     page_title="CSS Timing Function Demos" ;;
            format/css/keyframes)  page_title="CSS Keyframe Demos" ;;
            format/css/transforms) page_title="CSS Transform Demos" ;;
            export/spritesheet)    page_title="Spritesheet Demos" ;;
            *)                     page_title="${section##*/} Demos" ;;
        esac

        {
            echo "<!-- Generated by scripts/generate-demos.sh -->"
            echo "<!-- Do not edit manually - regenerate with: ./scripts/generate-demos.sh -->"
            echo ""
            echo "# $page_title"
            echo ""

            get_demos_for_section "$demos_data" "$section" | while IFS=$'\t' read -r demo title description cli jsonl_path; do
                generate_demo_markdown "$demo" "$title" "$description" "$cli" "$jsonl_path"
            done

        } > "$output_path"

        log_success "Generated: $output_path"
    done <<< "$sections"
}

# Compare two files and report differences
compare_files() {
    local expected="$1"
    local actual="$2"
    local name="$3"

    if [[ ! -f "$expected" ]]; then
        log_error "Missing file: $name (expected at $expected)"
        return 1
    fi

    if ! diff -q "$actual" "$expected" > /dev/null 2>&1; then
        log_error "File differs: $name"
        log_info "Run './scripts/generate-demos.sh --book' to regenerate"
        diff -u "$expected" "$actual" | head -50 || true
        return 1
    fi

    log_success "Verified: $name"
    return 0
}

# Main
main() {
    log_info "Demo Generator Script (DT-16, DT-18)"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Output directory: $OUTPUT_DIR"

    # In check mode, generate to temp directory and compare
    local actual_output_dir="$OUTPUT_DIR"
    local temp_dir=""
    if [[ "$CHECK_MODE" == "true" ]]; then
        temp_dir=$(mktemp -d)
        actual_output_dir="$temp_dir"
        log_info "Check mode: generating to temp directory"
        trap "rm -rf '$temp_dir'" EXIT
    fi

    # Ensure output directory exists
    if [[ "$DRY_RUN" == "false" ]]; then
        mkdir -p "$actual_output_dir"
    fi

    # Find all test files
    local test_files=()
    while IFS= read -r -d '' file; do
        test_files+=("$file")
    done < <(find "$TESTS_DIR" -name "*.rs" -print0 2>/dev/null || true)

    if [[ ${#test_files[@]} -eq 0 ]]; then
        log_warn "No test files found in $TESTS_DIR"
        exit 0
    fi

    log_info "Found ${#test_files[@]} test file(s)"

    # Parse all test files
    local all_demos=""
    local demo_count=0

    for file in "${test_files[@]}"; do
        local file_demos
        file_demos=$(parse_test_file "$file")
        if [[ -n "$file_demos" ]]; then
            all_demos+="$file_demos"$'\n'
            local count
            count=$(echo "$file_demos" | grep -c "^DEMO" || true)
            demo_count=$((demo_count + count))
            log_info "Parsed $(basename "$file"): $count demo(s)"
        fi
    done

    if [[ $demo_count -eq 0 ]]; then
        log_warn "No @demo annotations found"
        exit 0
    fi

    log_info "Total demos found: $demo_count"

    # Temporarily override OUTPUT_DIR for generation
    local saved_output_dir="$OUTPUT_DIR"
    OUTPUT_DIR="$actual_output_dir"

    # Process and generate output
    process_demos "$all_demos"

    OUTPUT_DIR="$saved_output_dir"

    # In check mode, compare generated files against committed files
    if [[ "$CHECK_MODE" == "true" ]]; then
        log_info "Comparing generated files against committed files..."
        local has_diff=false

        # Check all generated files
        for generated_file in "$temp_dir"/*.md; do
            [[ -f "$generated_file" ]] || continue
            local filename
            filename=$(basename "$generated_file")
            local expected_file="$OUTPUT_DIR/$filename"

            if ! compare_files "$expected_file" "$generated_file" "$filename"; then
                has_diff=true
            fi
        done

        # Check for extra committed files that shouldn't exist
        if [[ -d "$OUTPUT_DIR" ]]; then
            for committed_file in "$OUTPUT_DIR"/*.md; do
                [[ -f "$committed_file" ]] || continue
                local filename
                filename=$(basename "$committed_file")
                if [[ ! -f "$temp_dir/$filename" ]]; then
                    log_error "Extra file in committed docs: $filename"
                    has_diff=true
                fi
            done
        fi

        if [[ "$has_diff" == "true" ]]; then
            log_error "Demo documentation is out of date!"
            log_info "Run './scripts/generate-demos.sh --book' to regenerate"
            exit 1
        fi

        log_success "All demo documentation is up to date!"
        exit 0
    fi

    log_success "Done! Output in: $OUTPUT_DIR"
}

main "$@"
