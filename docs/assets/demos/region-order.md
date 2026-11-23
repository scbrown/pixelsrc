# Predictable source-order painting

Four overlapping regions use reverse alphabetical names. With default z values,
the last-defined yellow pixel must win; an explicit higher z must make red win.
The probe launches 16 separate CLI processes per case and checks decoded PNG
pixels, exposing both randomized maps and accidental alphabetical reordering.

[Terminal recording](region-order.log), [timing](region-order.time). These small
assets are retained with this fix. Download both and replay:

```
scriptreplay --log-out region-order.log --log-timing region-order.time
```

BEFORE is saved failure output on main
`d6382dc422ead0fa4615992da60ab31c80ca79c7` (green instead of yellow).
AFTER executes the source candidate in the commit containing this caption and
checks compilation with the Python feature using a supported interpreter.
This proves source-checkout behavior, not an installed release. Repeat:

```
cargo test --features lsp,wasm --test cli_region_order
```
