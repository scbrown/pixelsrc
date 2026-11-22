# Positioned composition layers

Replay with `scriptreplay --log-timing positioned-layers.time --log-out positioned-layers.log`.

Before: saved main d6382dc CLI failures for direct `{sprite,x,y}` placement: missing `sprites`. After: actual strict CLI renders verify positive placement, negative clipping, inferred bounds, nested composition placement, layer mirror/opacity, and formatter preservation. Existing direct derived-sprite and layer-transform PNG/GIF regressions remain green. The parse-recovery fixture now uses an invalid coordinate type because positioned layers are valid.

Full formatting, clippy, lsp/wasm tests and documentation checks passed under the HOME escape guard. Source-chain intermediate transform inheritance is a separate observed limitation; this change adds positioned layers and does not claim that inheritance fix.
