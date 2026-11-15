#!/usr/bin/env bash
set -eu
for mode in before after; do
    if [[ $mode == before ]]; then pxl=$PXL_BEFORE; else pxl=$PXL_AFTER; fi
    printf '%s: validate palette swap inherited through source, strict mode\n' "$mode"
    if "$pxl" validate tests/fixtures/valid/source_palette_swap.pxl --strict; then
        echo 'exit=0'
    else
        rc=$?; printf 'exit=%s\n' "$rc"
    fi
done
