#!/usr/bin/env bash
# Install git hooks for pixelsrc development
# Points core.hooksPath at .githooks/ in the repo so hooks are
# version-controlled and shared across all worktrees automatically.
#
# Run: ./scripts/install-hooks.sh

set -euo pipefail

git config core.hooksPath .githooks
echo "Set core.hooksPath to .githooks — hooks active for this clone and all its worktrees."
