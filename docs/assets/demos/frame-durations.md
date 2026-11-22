# Uneven GIF frame timing

`frame_durations: [40, null, "250ms"]` with `duration: 100` must produce GIF
delays of 40ms, 100ms and 250ms. The probe decodes the actual output GIF and also
checks the old uniform fallback, invalid timing diagnostics, composition frames,
and timing alignment when a missing frame is skipped in lenient mode.

[Terminal recording](frame-durations.log), [timing](frame-durations.time).
These small assets are retained with the change. Download both and replay:

```
scriptreplay --log-out frame-durations.log --log-timing frame-durations.time
```

BEFORE is saved regression output from base
`d6382dc422ead0fa4615992da60ab31c80ca79c7`: all delays were 100ms and zero was
silently accepted. AFTER executes the source candidate in the commit containing
this caption, including the decoded delays. This proves source-checkout behavior,
not a released binary. Repeat the probe:

```
cargo test --features lsp,wasm --test cli_frame_durations -- --nocapture
```
