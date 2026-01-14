# Phase 0 Gaps Analysis

**Date:** 2026-01-14
**Auditor:** Mayor

## Summary

Phase 0 was marked complete but has the following gaps against `docs/spec/format.md`:

| Issue | Severity | Status |
|-------|----------|--------|
| Duplicate name warning missing | HIGH | OPEN |
| Demo.sh format differs from spec | LOW | OPEN |

---

## Gap 1: Duplicate Name Warning Missing

**Spec requirement** (from `docs/spec/format.md` - Error Handling table):
> Duplicate name → Last definition wins → Warning: "Duplicate sprite name 'X', using latest"

**Actual behavior:**
```bash
$ ./target/release/pxl render tests/fixtures/lenient/duplicate_name.jsonl -o /tmp/dupe.png
Saved: /tmp/dupe_test_dupe.png
Saved: /tmp/dupe_test_dupe.png
# NO WARNING EMITTED
```

Both sprites are rendered and overwrite each other. No warning is generated.

**Required fix:**
- Track sprite names as they're processed
- When duplicate name encountered:
  - In lenient mode: emit warning, keep latest definition only
  - In strict mode: emit error, fail

**Files to modify:**
- `src/cli.rs` - Add duplicate detection logic in sprite collection

---

## Gap 2: Demo.sh Format Differs from Spec

**Spec requirement** (from `docs/plan/phase-0-mvp.md` line 446-509):
- Uses Unicode box-drawing characters (╔═╗║╚╝)
- Includes `file` command to show dimensions
- Has specific example structure

**Actual demo.sh:**
- Uses ASCII characters (====, ----)
- Uses different section headers
- Format is functionally equivalent but visually different

**Severity:** Low - functional behavior is correct

---

## Verification Commands

After fixes, verify with:

```bash
# Gap 1: Duplicate warning
./target/release/pxl render tests/fixtures/lenient/duplicate_name.jsonl -o /tmp/dupe.png 2>&1 | grep -i warning
# Should output: Warning: Duplicate sprite name 'dupe', using latest

# Gap 1: Strict mode
./target/release/pxl render tests/fixtures/lenient/duplicate_name.jsonl --strict -o /tmp/dupe.png 2>&1
# Should fail with exit code 1

# Full test suite
cargo test
cargo test --test integration_tests
./demo.sh
```

---

## Root Cause

Polecats closed tasks without verifying against the Error Handling table in `docs/spec/format.md`. The spec explicitly lists duplicate name handling as a required behavior.

**Lesson:** Task completion must include verification against ALL spec sections, not just the task description.
