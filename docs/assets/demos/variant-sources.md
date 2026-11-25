# Addressing variants directly and through a source

The same four-line [fixture](variant-sources.pxl) is rendered on base
`4ca2f042c311f5437bb52ea0a6a67c8e9388de55` and source candidate
`d2718be46327462c4bcf30af022d2595cd680fea`. These are source builds,
not an installed release.

Before, selecting `dot_blue` reports no such sprite and deriving `flip` reports
an unknown source. Both exit 1. After, both exit 0. Decoded PNGs are 4×4:
`dot_blue` has blue at the top left, and the derived mirror has blue at the top
right with transparency at the opposite corner. Omitting the source palette
inherits the variant colors; a separate CLI control verifies explicit palette
replacement. Mixed source/base cycles are diagnosed.

This is an output-only terminal recording, not a video or inline player.
Download the [output](variant-sources.log) and [timing](variant-sources.time), then:

```sh
scriptreplay --log-out variant-sources.log --log-timing variant-sources.time
```

Repeat the visible commands with the fixture on each source revision. Timing and
raw output are retained unchanged. This addresses variant selection and source
resolution; it does not claim to fix the separately observed intermediate
image-transform inheritance limitation in longer source chains.
