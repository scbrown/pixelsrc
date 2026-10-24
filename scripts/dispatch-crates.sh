#!/usr/bin/env bash
# Dispatch a separate workflow so crates.io sees crates.yml as the OIDC caller.
set -euo pipefail

if [[ ${GITHUB_REPOSITORY:-} != scbrown/pixelsrc ]]; then
  echo 'Refusing crates dispatch outside the publishing repository' >&2
  exit 2
fi
if [[ ! ${RELEASE_TAG:-} =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo 'Refusing missing or invalid release tag' >&2
  exit 2
fi

# workflow_dispatch is an explicit GITHUB_TOKEN recursion exception. The job
# has actions:write; no PAT, token argument or publisher account change is needed.
gh workflow run crates.yml --repo "$GITHUB_REPOSITORY" --ref main \
  -f "tag=$RELEASE_TAG" -f dry_run=false
