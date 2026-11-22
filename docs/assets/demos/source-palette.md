# Inherited source palettes

A source-only sprite inherits the base palette without repeating it. The probe
renders named-palette source chains and a composition, checks explicit inline
recoloring, preserves the direct-source mirror transform, and confirms that a
direct sprite still needs a palette.

[Terminal recording](source-palette.log), [timing](source-palette.time).
These small assets are retained with the change. Download both and replay:

```
scriptreplay --log-out source-palette.log --log-timing source-palette.time
```

BEFORE is saved failure output from new probes on main
`d6382dc422ead0fa4615992da60ab31c80ca79c7` (empty palette not found).
AFTER executes `tests/cli_source_palette.rs` on the source candidate in the
commit containing this caption. Tests assert actual PNG colors and validation
output. Source-chain transform inheritance is separate from this palette fix.
This demonstrates source-checkout behavior, not an installed release.

Repeat: `cargo test --features lsp,wasm --test cli_source_palette`.
