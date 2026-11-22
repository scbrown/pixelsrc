# One loaded browser project, several shared files

[Browser recording](shared-project.webm) and [final screenshot](shared-project.png)
show an actual Chromium run of the two files in `tests/fixtures/wasm_project`.
Download the WebM to play it in a compatible browser/player. Both small artifacts
are retained with this change; playback was decoded successfully and the final
screenshot was inspected.

The loaded Project parses two source files, resolves a supplied palette include,
and renders a nested named composition three times. Every RGBA byte matches a
PNG produced by `pxl render` from the concatenated files. An unavailable include
fails explicitly instead of becoming a silent fallback. The screenshot records
those checks. A checked-in cross-browser CI probe repeats this comparison against
a freshly generated CLI reference.

Base `d6382dc422ead0fa4615992da60ab31c80ca79c7` has only source-string entry points;
it has no loaded Project API. The candidate in the commit containing this
caption adds `Project.load(JSON.stringify([{path,text}, ...]))` and
`project.render(name)`. This is a new API, so no same-call before recording exists.
The recording proves source-checkout browser behavior, not a published package.

Validation also includes four native Project controls, actual generated-WASM Node
assertions, WASM compilation, full feature tests/clippy and HOME-escape checks.
The Node smoke fixtures use the current regions format and real throwing
assertions, so a failed assertion now fails CI.
