# Inherited palette token usage

The same named palette swap inherits its regions through `source`.
Base `22d344a413f60721269887df4dbc6cea7937c7f8` reports the used token `a`
as unused and strict validation fails. The candidate reports no warnings
and succeeds. Regression controls retain warnings for a truly unused token,
follow forward source chains and variant bases, and respect extends removal.
Source-checkout demonstration; no installed executable changed.
Candidate validator SHA256: `8836b67c0cf571517f1fc2e6b553e362b3fe5b05b2689b98d45478912d696a5b`.

[Recording/transcript](palette-source.log), [timing](palette-source.time).
Replay: `scriptreplay --log-out docs/bugs/palette-source.log --log-timing docs/bugs/palette-source.time`.
Reproduce: `PXL_BEFORE=/path/to/base/pxl PXL_AFTER=/path/to/head/pxl bash docs/bugs/record-palette-source.sh`.
This small util-linux recording is retained with its caption; no inline video
playback is claimed.
