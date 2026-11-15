#!/usr/bin/env bash
set -eu
scratch=$(mktemp -d /tmp/pxl-parse-demo.XXXXXX)
trap 'rm -rf "$scratch"' EXIT
for mode in before after; do
    if [[ $mode == before ]]; then pxl=$PXL_BEFORE; else pxl=$PXL_AFTER; fi
    printf '%s: render good composition after malformed composition\n' "$mode"
    if "$pxl" render docs/bugs/parse-recovery.pxl -c good -o "$scratch/result.png"; then
        echo 'exit=0, image saved'
    else
        rc=$?; printf 'exit=%s\n' "$rc"
    fi
done
