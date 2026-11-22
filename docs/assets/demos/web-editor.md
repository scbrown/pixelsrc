# Gallery and highlighted source editor

[Before recording](web-editor-before.webm), [after recording](web-editor-after.webm),
[before screenshot](web-editor-before.png), [after screenshot](web-editor-after.png).
Download the WebMs for playback. These small recordings were re-encoded for size;
originals remain in the task evidence. Playback was decoded and screenshots inspected.

Both are actual Chromium runs served under `/pixelsrc/`, matching the published
project path. Before: zero gallery items, six root `/examples/` 404 responses, a
JSONL placeholder and no CodeMirror. After: six usable examples, highlighted
source, a gallery selection that changes code and renders sprite pixels, and no
failed example requests. Keyboard rendering, URL-hash reload and a 390px-wide
viewport without horizontal overflow also passed.

The UI baseline is `d6382dc422ead0fa4615992da60ab31c80ca79c7` with only its stale
local-package lock synchronized so it could build. Both builds use the same
generated WASM module; its first-sprite entry point is unchanged by this UI fix.
The candidate is the source in the commit containing this caption. This is local
production-build behavior, not yet a published-page claim.

A read-only Website Check workflow builds the actual WASM/site and repeats the
project-prefix gallery, source-change, highlight and rendered-pixel checks on PRs
and main. The Pages deployment workflow retains its existing events and authority.
