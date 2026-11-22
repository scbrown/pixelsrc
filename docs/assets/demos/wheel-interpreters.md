# Qualified wheel interpreters

The macOS ARM wheel job on source `e0f0dd8` discovered Python 3.15 and failed
because PyO3 0.23.5 supports through Python 3.13. The candidate workflow names
`setup-python@v5`'s exact `python-path` outputs instead of scanning the host.
The configured matrix remains Python 3.9–3.13 on Windows and Intel macOS, and
3.10–3.13 on ARM macOS (3.9 remains unavailable there).

[Terminal recording](wheel-interpreters.log), [timing](wheel-interpreters.time).
Download both and replay with `scriptreplay --log-out wheel-interpreters.log
--log-timing wheel-interpreters.time`. These small demo assets stay with the fix.

BEFORE quotes the actual failed macOS ARM job. AFTER parses and prints the
candidate workflow source in the commit containing this caption. It demonstrates
the configured selector change, not successful remote builds. Normal CI must
prove all supported platform builds before landing. YAML parsing and the
setup-python v5 `python-path` output contract were checked locally. The published
job link is in the PR description.

Release runner correction: the exact-main ARM retry was cancelled before any runner or steps. Its GitHub annotation reports `macos-14-arm64-us-default` unsupported. GitHub retired macOS 14 runners on November 2, 2026. The release matrix now uses explicit `macos-15-intel` and `macos-15` labels, matching target architectures. Source: https://github.com/actions/runner-images/blob/main/README.md . Python wheel builds on these labels passed for all supported interpreters. Release artifact publication remains gated on the normal release workflow.
