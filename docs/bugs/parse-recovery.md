# Object parse recovery

A malformed composition precedes a valid composition named `good`.
Base `22d344a413f60721269887df4dbc6cea7937c7f8` exits 1 and hides `good`.
The candidate saves the 4x4 image, warns with the malformed object's type,
name and line, and exits 0. `--strict` still exits nonzero without rendering.
Source checkout demonstration; no installed executable was changed.
Candidate parser/render combined SHA256: `5e124ab4a5d89d44a51c9e0b823d7f71e16b4a2c93880a17eb81cb78e209753e`.

[Recording/transcript](parse-recovery.log), [timing](parse-recovery.time).
Replay with `scriptreplay --log-out docs/bugs/parse-recovery.log --log-timing docs/bugs/parse-recovery.time`.
Reproduce with `PXL_BEFORE=/path/to/base/pxl PXL_AFTER=/path/to/head/pxl bash docs/bugs/record-parse-recovery.sh`.
The small util-linux recording is retained with this caption; terminal replay
is supported, no inline video playback is claimed.
