#!/usr/bin/env bash
set -eu
scratch=$(mktemp -d /tmp/pxl-nested-demo.XXXXXX)
trap 'rm -rf "$scratch"' EXIT
for mode in before after; do
    if [[ $mode == before ]]; then pxl=$PXL_BEFORE; else pxl=$PXL_AFTER; fi
    printf '%s: render scene containing a repeated nested pair\n' "$mode"
    "$pxl" render tests/fixtures/valid/nested_structured.pxl -c scene -o "$scratch/scene.png"
    python3 - "$scratch/scene.png" <<'PY'
from PIL import Image
import sys
image = Image.open(sys.argv[1]).convert('RGBA')
print('canvas=%dx%d visible_pixels=%d' % (*image.size, sum(image.getchannel('A').histogram()[1:])))
PY
done
