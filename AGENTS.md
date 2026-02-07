# Agent Instructions

This project is managed by **Gas Town** (`gt`). Full context is injected at session start.

> **Recovery**: Run `gt prime` after compaction, clear, or new session.

## For Humans

If you're working manually (not as a gastown agent), use the justfile:

```bash
just ready            # Find available work
just issues           # List open issues
just status           # Git + issue status
```


## Required Reading

Before working on this codebase, read **CONTRIBUTING.md**. It documents CI requirements
including demo documentation regeneration, clippy rules, and feature flags.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
