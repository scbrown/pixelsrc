# README art: actual GitHub before and after

The README on base `6aa65808b4f879fe9f1a9937fa1a33d4f3e9bec0` used the old
144×64 banner and two unpaired JSON5 examples. Both examples failed strict
rendering with released pxl 0.4.0 because they used the old background shorthand.

Source head `6438f74973847e69284a8da07bfcf603dbb4969a` replaces the banner with
a semantic, composable pixelsrc logo: a cyan/purple/pink lowercase p with a
brush-stroke descender. Native64 and scaled256 PNGs come from its editable source.
Coin and hero now use supported geometry and have192px nearest-neighbor previews
immediately below their generated source blocks.

These recordings show GitHub's actual rendered README at the two exact commits,
not a simulated Markdown page. After controls verified all three images loaded,
the expected dimensions, both source blocks and the displayed previews.

- [Before recording](readme-art-before.webm) · [before screenshot](readme-art-before.png)
- [After recording](readme-art-after.webm) · [logo screenshot](readme-art-after.png)
- [Coin and source](readme-art-coin.png) · [hero and source](readme-art-hero.png)

The WebM files decode successfully; download them if inline playback is unavailable.

Reproduce generation and the CI drift check with released pxl0.4.0:

```sh
just readme-assets /path/to/pxl
just readme-assets-check /path/to/pxl
```

Regeneration produced identical bytes. Independent controls reject changed PNGs,
edited README snippets, undeclared JSON5 examples and changed source; check mode
leaves the files untouched. Every scaled pixel matches its native source pixel.
The CI job installs the pinned release into an isolated runner directory and runs
the same check. The existing old banner remains available to other consumers.
