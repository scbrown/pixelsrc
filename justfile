# Pixelsrc: Pixel Art Format
# Run `just --list` to see available recipes

# Quiet by default to save context; use verbose=true for full output
verbose := "false"

# Default recipe - show available commands
default:
    @just --list

# === Development ===

# Build the project
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Run tests
test:
    cargo test {{ if verbose == "true" { "--verbose" } else { "" } }}

# Run linter (clippy)
lint:
    cargo clippy --all-targets

# Format code
fmt:
    cargo fmt --all

# Check formatting without changing files
fmt-check:
    cargo fmt --all -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# === CLI ===

# Render an example (e.g., just render coin)
render name:
    cargo run -- render examples/{{name}}.jsonl -o /tmp/{{name}}.png && echo "Saved to /tmp/{{name}}.png"

# List built-in palettes
palettes:
    cargo run -- palettes list

# Run the demo
demo:
    ./demo.sh

# === Issue Tracking ===

# Show ready issues
ready:
    bd ready

# Show all open issues
issues:
    bd list --status open

# Show blocked issues
blocked:
    bd blocked

# Sync issues with remote
sync:
    bd sync

# === Git Helpers ===

# Status check
status:
    @git status
    @echo ""
    @bd list --status in_progress 2>/dev/null || true
