# Composition antialiasing

Replay: `scriptreplay --log-timing composition-aa.time --log-out composition-aa.log`.

The before segment contains saved failures from main d6382dc: xBR4x output stayed 4×4 and animation AA flags were silently accepted. The after segment runs actual CLI renders, checks xBR4x/HQ4x output against the selected library algorithm, verifies xBR edge interpolation, selected/batch parity, Scale2x, additional nearest scaling, and an unsupported GIF diagnostic.

PNG AA runs on the composed bitmap before additional `--scale`. Region and anchor metadata are unavailable at this stage. An additional xBR fix separates edge-vote magnitudes from color-distance thresholds so default-strength interpolation can occur. The full format, clippy, lsp/wasm tests and documentation gates passed under the HOME escape guard.
