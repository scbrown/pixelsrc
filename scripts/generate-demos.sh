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
#   --inline          Inject demos into existing pages at <!-- DEMOS --> markers
#   --output-dir DIR  Output to specified directory (default: target/demos)
#   --check           Verify output matches existing files (for CI regression)
#
# Annotations supported:
#   /// @demo section/subsection#anchor
#   /// @title Demo Title
#   /// @description Description text.
#   /// @cli pxl render example.jsonl -o output.png
#
# Marker format in .md files:
#   <!-- DEMOS section/subsection#anchor -->
#   <!-- /DEMOS -->
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
BOOK_MODE=false
INLINE_MODE=false
DOCS_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --book)
            OUTPUT_DIR="$BOOK_DIR"
            BOOK_MODE=true
            shift
            ;;
        --inline)
            INLINE_MODE=true
            shift
            ;;
        --docs-dir)
            DOCS_DIR="$2"
            shift 2
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

# Default docs directory for inline mode
if [[ -z "$DOCS_DIR" ]]; then
    DOCS_DIR="$PROJECT_ROOT/docs/book/src"
fi

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

# =============================================================================
# Inline Injection Functions (DT-17)
# =============================================================================

# Get demo data for a specific path (section#anchor)
# Returns tab-separated: demo<TAB>title<TAB>description<TAB>cli<TAB>jsonl_path
get_demo_for_path() {
    local demos_data="$1"
    local target_path="$2"
    echo "$demos_data" | while IFS=$'\t' read -r marker demo title description cli jsonl_path; do
        [[ "$marker" != "DEMO" ]] && continue
        if [[ "$demo" == "$target_path" ]]; then
            printf "%s\t%s\t%s\t%s\t%s\n" "$demo" "$title" "$description" "$cli" "$jsonl_path"
            return 0
        fi
    done
}

# Generate inline demo markdown (simpler format for embedding)
generate_inline_demo_markdown() {
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

    # Extract anchor from demo path
    local anchor="${demo##*#}"

    # Generate compact inline format
    if [[ -n "$title" ]]; then
        echo "**$title**"
        echo ""
    fi

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

    if [[ -n "$cli" ]]; then
        echo ""
        echo '**CLI equivalent:**'
        echo '```bash'
        echo "$cli"
        echo '```'
    fi
}

# Find all markdown files with DEMOS markers
find_files_with_markers() {
    local docs_dir="$1"
    # Find files containing DEMOS markers
    grep -rl '<!-- DEMOS ' "$docs_dir" --include='*.md' 2>/dev/null || true
}

# Inject content between markers in a file
# Uses temp file approach to handle multi-line content safely
inject_between_markers() {
    local input_file="$1"
    local output_file="$2"
    local start_marker="$3"
    local end_marker="$4"
    local replacement_file="$5"

    awk -v start="$start_marker" -v end="$end_marker" -v repl_file="$replacement_file" '
        BEGIN { in_block = 0 }
        index($0, start) {
            print $0
            # Read and print replacement from file
            while ((getline line < repl_file) > 0) {
                print line
            }
            close(repl_file)
            in_block = 1
            next
        }
        in_block && index($0, end) {
            print $0
            in_block = 0
            next
        }
        !in_block { print }
    ' "$input_file" > "$output_file"
}

# Process inline demos - inject generated content into markdown files at markers
process_inline_demos() {
    local demos_data="$1"
    local docs_dir="$2"
    local check_mode="$3"
    local dry_run="$4"

    local has_changes=false
    local has_errors=false

    # Create temp directory for working files
    local work_dir
    work_dir=$(mktemp -d)
    trap "rm -rf '$work_dir'" RETURN

    # Find all files with DEMOS markers
    local files_with_markers
    files_with_markers=$(find_files_with_markers "$docs_dir")

    if [[ -z "$files_with_markers" ]]; then
        log_warn "No files with <!-- DEMOS --> markers found in $docs_dir"
        return 0
    fi

    # Process each file
    while IFS= read -r md_file; do
        [[ -z "$md_file" ]] && continue

        log_info "Processing: $md_file"

        # Copy original to working file
        local work_file="$work_dir/current.md"
        local temp_file="$work_dir/temp.md"
        cp "$md_file" "$work_file"

        # Find all DEMOS markers in the file
        # Pattern: <!-- DEMOS section#anchor -->
        local markers
        markers=$(grep -o '<!-- DEMOS [^>]*-->' "$md_file" | sed 's/<!-- DEMOS //;s/ -->//' || true)

        if [[ -z "$markers" ]]; then
            continue
        fi

        local file_changed=false

        # Process each marker
        while IFS= read -r marker_path; do
            [[ -z "$marker_path" ]] && continue

            # Get demo data for this path
            local demo_info
            demo_info=$(get_demo_for_path "$demos_data" "$marker_path")

            if [[ -z "$demo_info" ]]; then
                log_warn "No demo found for marker: $marker_path in $md_file"
                continue
            fi

            # Parse demo info
            local demo title description cli jsonl_path
            IFS=$'\t' read -r demo title description cli jsonl_path <<< "$demo_info"

            # Generate replacement content to temp file
            local repl_file="$work_dir/replacement.md"
            if ! generate_inline_demo_markdown "$demo" "$title" "$description" "$cli" "$jsonl_path" > "$repl_file"; then
                has_errors=true
                continue
            fi

            # Build the full replacement block
            local start_marker="<!-- DEMOS $marker_path -->"
            local end_marker="<!-- /DEMOS -->"

            # Inject content using temp files
            inject_between_markers "$work_file" "$temp_file" "$start_marker" "$end_marker" "$repl_file"
            mv "$temp_file" "$work_file"
            file_changed=true

        done <<< "$markers"

        # Check if content changed from original
        if [[ "$file_changed" == "true" ]] && ! diff -q "$md_file" "$work_file" > /dev/null 2>&1; then
            has_changes=true

            if [[ "$dry_run" == "true" ]]; then
                log_info "[DRY-RUN] Would update: $md_file"
            elif [[ "$check_mode" == "true" ]]; then
                log_error "File needs update: $md_file"
                # Show diff preview
                diff -u "$md_file" "$work_file" | head -30 || true
            else
                # Write updated content
                cp "$work_file" "$md_file"
                log_success "Updated: $md_file"
            fi
        else
            log_success "Up to date: $md_file"
        fi

    done <<< "$files_with_markers"

    if [[ "$check_mode" == "true" && "$has_changes" == "true" ]]; then
        log_error "Some files need updating. Run './scripts/generate-demos.sh --inline' to regenerate."
        return 1
    fi

    if [[ "$has_errors" == "true" ]]; then
        log_warn "Some demos could not be processed"
    fi

    return 0
}

# Main
main() {
    log_info "Demo Generator Script (DT-16, DT-17, DT-18)"
    log_info "Project root: $PROJECT_ROOT"

    if [[ "$INLINE_MODE" == "true" ]]; then
        log_info "Mode: inline injection"
        log_info "Docs directory: $DOCS_DIR"
    else
        log_info "Mode: standalone generation"
        log_info "Output directory: $OUTPUT_DIR"
    fi

    # In check mode for standalone generation, generate to temp directory
    local actual_output_dir="$OUTPUT_DIR"
    local temp_dir=""
    if [[ "$CHECK_MODE" == "true" && "$INLINE_MODE" == "false" ]]; then
        temp_dir=$(mktemp -d)
        actual_output_dir="$temp_dir"
        log_info "Check mode: generating to temp directory"
        trap "rm -rf '$temp_dir'" EXIT
    fi

    # Ensure output directory exists (for standalone mode)
    if [[ "$DRY_RUN" == "false" && "$INLINE_MODE" == "false" ]]; then
        mkdir -p "$actual_output_dir"
    fi

    # Find all test files
    local test_files=()
    while IFS= read -r -d '' file; do
        test_files+=("$file")
    done < <(find "$TESTS_DIR" -name "*.rs" -print0 2>/dev/null | sort -z || true)

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

    # Handle inline mode
    if [[ "$INLINE_MODE" == "true" ]]; then
        log_info ""
        log_info "Processing inline demos..."

        if ! process_inline_demos "$all_demos" "$DOCS_DIR" "$CHECK_MODE" "$DRY_RUN"; then
            exit 1
        fi

        if [[ "$CHECK_MODE" == "true" ]]; then
            log_success "All inline demos are up to date!"
        else
            log_success "Done! Inline demos injected into: $DOCS_DIR"
        fi
        exit 0
    fi

    # Standalone mode: generate separate files

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
