# Enclosed concave fill

The original U-shaped outline must fill both arms and the bottom, leaving the
notch transparent. `seed: [1,1]` must render that same interior; `seed: [4,4]`
selects the exterior and must fail in strict mode.

[Terminal recording](inside-fill.log) and [timing](inside-fill.time) are retained
with this change as small repository demo assets. Download both and replay with:

```
scriptreplay --log-out inside-fill.log --log-timing inside-fill.time
```

The BEFORE section is saved output from tests added to base
`d6382dc422ead0fa4615992da60ab31c80ca79c7`; it is labelled as saved output.
The AFTER section runs the candidate's CLI tests. These tests exercise the
checked-in `tests/cli_inside_fill.rs` input and assert actual PNG pixels and
strict-mode refusal. This is source-checkout behavior, not an installed release.
The demo corresponds to the code in the commit containing this caption.

Repeat the candidate probe: `cargo test --features lsp,wasm --test cli_inside_fill`.
