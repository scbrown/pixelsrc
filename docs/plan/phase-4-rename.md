# Phase 4: Project Rename

**Goal:** Rename project from TTP to Pixelsrc

**Status:** Complete

**Depends on:** None (can be done anytime)

---

## Scope

Rename the project from "TTP (Text To Pixel)" to "Pixelsrc" while keeping the CLI command as `pxl`.

**Rationale:**
- "Pixelsrc" clearly describes the project: a pixel art source format
- "pxl" stays as the CLI command (short, easy to type)
- Crate name becomes `pixelsrc` for crates.io publishing (since `pxl` is taken)

---

## Tasks

### Task 4.1: Update Cargo.toml
- Change package name from `pxl` to `pixelsrc`
- Update description
- Add `[[bin]]` section to keep CLI as `pxl`

### Task 4.2: Update Source Code
- Update doc comments in all source files
- Replace "TTP" with "Pixelsrc"

### Task 4.3: Update CLI Help Text
- Update clap about text
- Update command descriptions

### Task 4.4: Update Documentation
- Update all docs/*.md files
- Replace "TTP (Text To Pixel)" with "Pixelsrc"

### Task 4.5: Update Build Scripts
- Update justfile header
- Update demo.sh

### Task 4.6: Update Tests
- Update integration test assertion for help text

---

## Verification

1. `cargo build` compiles successfully
2. `cargo test` passes
3. `pxl --help` shows "Pixelsrc"
4. `grep -r "TTP" docs/` returns no results (except historical references)
