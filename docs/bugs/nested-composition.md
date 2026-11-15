# Nested composition rendering

The same 8x8 scene contains two copies of a composition containing two dots.
Base `22d344a413f60721269887df4dbc6cea7937c7f8` emits missing-sprite and grid
warnings and saves an empty image (0 visible pixels). The candidate renders
all four dots (16 visible pixels), without those warnings. CLI regressions
compare every RGBA pixel and cover cycles, nested bases/animation frames,
strict size validation and sprite-name precedence.
Source-checkout demonstration; no installed executable changed.
Candidate CLI-render SHA256: `74d8db4475172daa709bc024a701c0379610d880b0b83b1d506e6db99b64ea79`.

[Recording/transcript](nested-composition.log), [timing](nested-composition.time).
Replay: `scriptreplay --log-out docs/bugs/nested-composition.log --log-timing docs/bugs/nested-composition.time`.
Reproduce: `PXL_BEFORE=/path/to/base/pxl PXL_AFTER=/path/to/head/pxl bash docs/bugs/record-nested-composition.sh`.
The demo's pixel counter needs Pillow; rendering needs only the CLI. This
small util-linux recording is retained with its caption; no inline video is claimed.
