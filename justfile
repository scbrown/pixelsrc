# TTP: Pixel Art DSL
# Run `just --list` to see available recipes

# Quiet by default to save context; use verbose=true for full output
verbose := "false"

# Default recipe - show available commands
default:
    @just --list

# === Development ===

# Run tests (quiet by default)
test:
    #!/usr/bin/env bash
    if [ -d "src" ] && [ -f "pyproject.toml" ]; then
        python -m pytest {{ if verbose == "true" { "-v" } else { "-q" } }}
    else
        echo "Test infrastructure not yet set up"
    fi

# Run linter
lint:
    #!/usr/bin/env bash
    if [ -d "src" ]; then
        ruff check .
    else
        echo "Lint infrastructure not yet set up"
    fi

# Format code
fmt:
    #!/usr/bin/env bash
    if [ -d "src" ]; then
        ruff format .
    else
        echo "Format infrastructure not yet set up"
    fi

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
