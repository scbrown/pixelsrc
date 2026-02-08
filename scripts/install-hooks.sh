#!/usr/bin/env bash
# Install git hooks for pixelsrc development
# Run: ./scripts/install-hooks.sh

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing git hooks..."

# Pre-commit: fast checks (fmt + clippy)
cat > "$HOOKS_DIR/pre-commit" << 'HOOK'
#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Checking formatting..."
cargo fmt --all -- --check

echo "==> Running clippy..."
cargo clippy --all-targets --features lsp,wasm -- -D warnings

echo "==> Pre-commit checks passed."
HOOK
chmod +x "$HOOKS_DIR/pre-commit"

echo "Installed pre-commit hook (fmt + clippy)"
echo "Done."
