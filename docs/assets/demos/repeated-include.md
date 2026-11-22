# Reusing a palette include across GIF frames

Two frames of the same sprite both load `@include:base.pxl`. Both decoded GIF
frames must have the same red pixel, with no false circular-include or unknown
token warning. The probe runs in lenient and strict mode. Include unit controls
also reject a file already on the active stack and unwind failed lookups without
removing caller-owned active entries.

[Terminal recording](repeated-include.log), [timing](repeated-include.time).
These small assets are retained with the change. Download both and replay:

```
scriptreplay --log-out repeated-include.log --log-timing repeated-include.time
```

BEFORE is saved failing regression output on main
`d6382dc422ead0fa4615992da60ab31c80ca79c7`; the temporary fixture path is redacted.
AFTER runs the source candidate in the commit containing this caption. This is
source-checkout behavior, not an installed release. Repeat:

```
cargo test --features lsp,wasm --test cli_repeated_include -- --nocapture
cargo test --features lsp,wasm --lib include::tests
```
