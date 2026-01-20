---
phase: 23
title: Demo Coverage Tracking
---

# Demo Coverage Tracking

This document tracks demo coverage over time. Updated automatically by CI runs
and manually during development milestones.

## Current Coverage

| Date | Coverage | Features | Notes |
|------|----------|----------|-------|
| 2026-01-19 | 34% | 29/84 | Full registry (DT-29) - added palette-cycling, imports, build-system, CLI categories |
| 2026-01-19 | 60% | 26/43 | Initial tracking (DT-19) |

## Coverage by Category

### Well Covered (80%+)

| Category | Coverage | Notes |
|----------|----------|-------|
| CSS Colors | 100% (7/7) | All color formats covered |
| CSS Variables | 100% (4/4) | Full variable system coverage |
| CSS Timing | 100% (3/3) | All timing functions covered |
| CSS Keyframes | 100% (4/4) | All keyframe features covered |
| Transforms | 83% (5/6) | Missing: recolor/palette swap |
| Exports | 60% (6/10) | PNG/GIF/Spritesheet done; missing atlas formats |

### Needs Coverage (<80%)

| Category | Coverage | Missing |
|----------|----------|---------|
| Sprites | 0% (0/8) | All sprite features need demos |
| Animation | 0% (0/6) | All animation features need demos |
| Composition | 0% (0/5) | All composition features need demos |
| Palette Cycling | 0% (0/4) | New category - cycle effects |
| Imports | 0% (0/4) | New category - PNG import features |
| Build System | 0% (0/5) | New category - pxl.toml, watch mode |
| CLI Core | 0% (0/3) | New category - render, import, validate |
| CLI Format | 0% (0/4) | New category - fmt, show, explain, inline |
| CLI Analysis | 0% (0/5) | New category - diff, suggest, alias, grid, analyze |
| CLI Project | 0% (0/3) | New category - build, new, init |
| CLI Info | 0% (0/3) | New category - prime, prompts, palettes |

## Missing Demos

The following features from the [Phase 23 Feature Checklist](demo-tests.md#feature-coverage-checklist)
need demo coverage:

### Sprites (Priority: High)

These are fundamental features that should be covered first:

- [ ] Basic sprite (minimal valid example) - `@demo format/sprite#basic`
- [ ] Named palette reference - `@demo format/sprite#named_palette`
- [ ] Inline palette definition - `@demo format/sprite#inline_palette`
- [ ] Multi-character color keys - `@demo format/sprite#multichar_keys`
- [ ] Transparency (. character) - `@demo format/sprite#transparency`
- [ ] Origin point - `@demo format/sprite#origin`
- [ ] Collision boxes - `@demo format/sprite#collision`
- [ ] Attachment points - `@demo format/sprite#attachments`

### Animation (Priority: High)

Animation demos should demonstrate frame sequences and timing:

- [ ] Basic frame sequence - `@demo format/animation#basic`
- [ ] Frame timing (FPS) - `@demo format/animation#fps`
- [ ] Frame tags (named ranges) - `@demo format/animation#tags`
- [ ] Looping modes - `@demo format/animation#looping`
- [ ] Attachment chains - `@demo format/animation#attachments`

### Composition (Priority: Medium)

Layer composition demos:

- [ ] Basic layer stacking - `@demo format/composition#basic`
- [ ] Layer positioning (offsets) - `@demo format/composition#positioning`
- [ ] Blend modes - `@demo format/composition#blend`
- [ ] Background fills - `@demo format/composition#fills`

## Usage

### Running Coverage Check Locally

```bash
# Basic coverage report
./scripts/demo-coverage.sh

# With verbose output (shows covered features too)
./scripts/demo-coverage.sh --verbose

# JSON output for tooling
./scripts/demo-coverage.sh --json

# CI mode with threshold enforcement
./scripts/demo-coverage.sh --ci --threshold 70
```

### Adding a New Demo

1. Create the JSONL fixture in `examples/demos/<category>/`
2. Add a test with `@demo` annotation in `tests/demos/mod.rs`:
   ```rust
   /// @demo format/sprite#basic
   /// @title Basic Sprite
   /// @description The simplest valid sprite definition.
   #[test]
   fn test_sprite_basic() {
       let jsonl = include_str!("../../examples/demos/sprites/basic.jsonl");
       assert_validates(jsonl, true);
       // ...
   }
   ```
3. Run `./scripts/demo-coverage.sh` to verify coverage increased

### Feature Registry

The coverage script maintains a feature registry mapping feature names to expected
`@demo` paths. To add a new feature to track:

1. Edit `scripts/demo-coverage.sh`
2. Add entry to `FEATURE_REGISTRY` section:
   ```
   category|Feature Name|expected/demo#path
   ```

## Coverage Goals

| Milestone | Target | Notes |
|-----------|--------|-------|
| Current (DT-29) | 35% | Baseline after full registry expansion |
| Phase 23 Wave 2 | 50% | Add sprite, animation, composition demos |
| Phase 23 Wave 3 | 70% | Add export, CLI, build system demos |
| Phase 23 Wave 4 | 85% | Add palette cycling, import demos |
| Phase 23 Complete | 95% | Full feature coverage |

## Feature Registry Summary

The coverage script tracks **84 features** across **17 categories**:

| Group | Categories | Features |
|-------|------------|----------|
| Core Format | sprites, transforms, animation, composition | 25 |
| Palette | palette-cycling | 4 |
| I/O | imports, exports | 14 |
| Build | build-system | 5 |
| CLI | cli-core, cli-format, cli-analysis, cli-project, cli-info | 18 |
| CSS | css-colors, css-variables, css-timing, css-keyframes | 18 |

**Total: 84 features**

## CI Integration

Demo coverage is checked automatically in CI:

- Runs after tests pass
- Reports coverage percentage
- Currently informational (doesn't fail build)
- Future: enforce minimum threshold once coverage improves

See `.github/workflows/ci.yml` for the check step configuration.
