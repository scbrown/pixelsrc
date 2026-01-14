# Phase 10: GitHub Migration

**Goal:** Move repo to GitHub with working CI/CD for releases, package builds, and website hosting

**Status:** Complete

**Depends on:** Existing workflows already in repo

---

## Scope

Phase 10 migrates the pixelsrc repository to GitHub and ensures all CI/CD pipelines work correctly:

- **Repository Setup** - Create GitHub repo, push code, configure settings
- **Secrets Configuration** - Set up NPM_TOKEN and any other required secrets
- **GitHub Pages** - Enable Pages for the website/WASM demo
- **Release Testing** - Verify release workflow builds all targets
- **Package Publishing** - Verify npm publishing for @pixelsrc/wasm

**Not in scope:** Custom domain (can be added later), crates.io publishing, additional package managers beyond what's configured

---

## Task Dependency Diagram

```
                          PHASE 10 TASK FLOW
═══════════════════════════════════════════════════════════════════

WAVE 1 (Setup)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   10.1 Repository Setup                                  │   │
│  │   - Create GitHub repo                                   │   │
│  │   - Push existing code                                   │   │
│  │   - Configure repo settings                              │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (Configuration - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   10.2 Secrets     │  │   10.3 Pages       │                 │
│  │   - NPM_TOKEN      │  │   - Enable Pages   │                 │
│  │   - Verify access  │  │   - Configure src  │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 3 (Validation - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   10.4 Website     │  │   10.5 WASM CI     │                 │
│  │   Deploy           │  │   Validation       │                 │
│  │   - Trigger build  │  │   - Trigger build  │                 │
│  │   - Verify Pages   │  │   - Check npm pub  │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 4 (Release)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────────────────────────────────────┐   │
│  │   10.6 Release Workflow Validation                       │   │
│  │   - Create test tag                                      │   │
│  │   - Verify all 6 platform builds                         │   │
│  │   - Verify .deb package                                  │   │
│  │   - Verify checksums                                     │   │
│  │   - Verify GitHub Release creation                       │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 5 (Post-Release - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   10.7 Homebrew    │  │   10.8 Docs &      │                 │
│  │   Tap              │  │   README           │                 │
│  │   - Verify formula │  │   - Update links   │                 │
│  │   - Test install   │  │   - Add badges     │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 10.1                 (1 task - setup)                  │
│  Wave 2: 10.2 + 10.3          (2 tasks in parallel)             │
│  Wave 3: 10.4 + 10.5          (2 tasks in parallel)             │
│  Wave 4: 10.6                 (1 task - release test)           │
│  Wave 5: 10.7 + 10.8          (2 tasks in parallel)             │
└─────────────────────────────────────────────────────────────────┘
```

---

## Pre-Migration Checklist

Before starting, ensure:

- [ ] GitHub account created/accessible
- [ ] npm account created with access to @pixelsrc scope (or create new scope)
- [ ] Decide on repo visibility (public/private)
- [ ] Decide on GitHub organization vs personal account

---

## Tasks

### Task 10.1: Repository Setup

**Wave:** 1

Create GitHub repository and push code.

**Deliverables:**

1. Create new GitHub repository:
   ```bash
   gh repo create pixelsrc --public --description "GenAI-native pixel art format and compiler"
   # Do NOT use --add-readme - GitHub would create a commit that conflicts with local history
   ```

2. Update git remotes:
   ```bash
   # Check current remotes
   git remote -v

   # Add GitHub as origin (or replace existing)
   git remote add origin git@github.com:scbrown/pixelsrc.git
   # OR if origin exists:
   git remote set-url origin git@github.com:scbrown/pixelsrc.git

   # Push all branches and tags
   git push -u origin main
   git push --tags origin
   ```

3. Configure repository settings:
   - **General**:
     - Enable "Issues"
     - Enable "Projects" (optional)
     - Disable "Wiki" (docs are in repo)
   - **Branches**:
     - Set `main` as default branch
     - Add branch protection rule for `main`:
       - Require status checks to pass
       - Require branches to be up to date
   - **Actions**:
     - Enable "Allow all actions and reusable workflows"
     - Set workflow permissions to "Read and write"
   - **Pages**:
     - Will configure in Task 10.3

4. Update URLs in codebase (if username differs from placeholder):
   ```bash
   # Files that may need URL updates:
   # - README.md
   # - Cargo.toml (repository field)
   # - package.json files
   # - .github/workflows/*.yml (if hardcoded)
   # - homebrew-pixelsrc/Formula/pxl.rb
   ```

**Verification:**
```bash
# Verify push succeeded
git log origin/main --oneline -5

# Verify repo is accessible
open https://github.com/scbrown/pixelsrc
```

**Dependencies:** None

---

### Task 10.2: Secrets Configuration

**Wave:** 2 (parallel with 10.3)

Configure repository secrets for CI/CD.

**Deliverables:**

1. Generate npm access token:
   - Go to https://www.npmjs.com/settings/scbrown/tokens
   - Click "Generate New Token" → "Classic Token"
   - Select "Automation" type
   - Copy the token

2. Add secrets to GitHub:
   - Go to repo → Settings → Secrets and variables → Actions
   - Add repository secret:
     - Name: `NPM_TOKEN`
     - Value: (paste npm token)

3. Verify npm scope access:
   ```bash
   # Check if @pixelsrc scope exists or if you need a different scope
   npm whoami
   npm access ls-packages

   # If @pixelsrc is not available, update wasm/package.json
   # to use a different scope like @scbrown/pixelsrc-wasm
   ```

4. If using different npm scope, update `wasm/package.json`:
   ```json
   {
     "name": "@your-scope/pixelsrc-wasm",
     ...
   }
   ```

**Verification:**
```bash
# Trigger a workflow manually to test secrets
# (will verify in Task 10.5)
```

**Dependencies:** Task 10.1

---

### Task 10.3: GitHub Pages Configuration

**Wave:** 2 (parallel with 10.2)

Enable GitHub Pages for website hosting.

**Deliverables:**

1. Configure GitHub Pages:
   - Go to repo → Settings → Pages
   - Source: "GitHub Actions"
   - This allows the `website.yml` workflow to deploy

2. Update `website/vite.config.ts` base path if needed:
   ```typescript
   import { defineConfig } from 'vite';

   export default defineConfig({
     // Use repo name as base path for GitHub Pages
     base: '/pixelsrc/',  // Adjust to match your repo name
     build: {
       outDir: 'dist',
     },
   });
   ```

3. Verify workflow permissions:
   - Go to repo → Settings → Actions → General
   - Workflow permissions: "Read and write permissions"
   - Check "Allow GitHub Actions to create and approve pull requests"

**Verification:**
```bash
# Check Pages settings
open https://github.com/scbrown/pixelsrc/settings/pages
```

**Dependencies:** Task 10.1

---

### Task 10.4: Website Deployment Validation

**Wave:** 3 (parallel with 10.5)

Trigger and verify website deployment.

**Deliverables:**

1. Trigger website workflow:
   ```bash
   # Option 1: Push a change to website/ or wasm/
   touch website/src/.trigger && git add -A && git commit -m "Trigger website deploy" && git push
   git rm website/src/.trigger && git commit -m "Cleanup trigger" && git push

   # Option 2: Manually trigger via GitHub UI
   # Go to Actions → Deploy Website → Run workflow
   ```

2. Monitor workflow:
   - Go to repo → Actions → "Deploy Website"
   - Watch for completion
   - Check for errors

3. Verify deployment:
   - Site should be live at: `https://scbrown.github.io/pixelsrc/`
   - Test features:
     - [ ] Page loads without errors
     - [ ] Editor works
     - [ ] Preview renders sprites
     - [ ] Example gallery loads
     - [ ] Download PNG works
     - [ ] Copy to clipboard works
     - [ ] URL sharing works

4. Fix any issues:
   - Check browser console for errors
   - Common issues:
     - WASM loading failure (base path wrong)
     - CORS issues
     - Missing dependencies

**Verification:**
```bash
# Open deployed site
open https://scbrown.github.io/pixelsrc/
```

**Dependencies:** Tasks 10.2, 10.3

---

### Task 10.5: WASM CI Validation

**Wave:** 3 (parallel with 10.4)

Verify WASM build and npm publishing works.

**Deliverables:**

1. Trigger WASM workflow:
   ```bash
   # Push a change to src/ or wasm/
   # Or trigger manually via GitHub UI
   ```

2. Monitor workflow:
   - Go to repo → Actions → "WASM Build"
   - Verify build job passes:
     - [ ] Rust compile succeeds
     - [ ] wasm-pack build succeeds
     - [ ] wasm-pack tests pass
     - [ ] npm tests pass
   - Verify publish job:
     - [ ] npm publish succeeds (or "already published" message)

3. If publish fails with scope error:
   - Option A: Create the npm organization/scope
   - Option B: Update package name to use personal scope

4. Verify on npm:
   ```bash
   npm view @pixelsrc/wasm
   # OR
   npm view @your-scope/pixelsrc-wasm
   ```

**Verification:**
```bash
# Check npm package exists
npm view @pixelsrc/wasm
```

**Dependencies:** Task 10.2

---

### Task 10.6: Release Workflow Validation

**Wave:** 4

Test the full release workflow.

**Deliverables:**

1. Create a test release tag:
   ```bash
   # Ensure all changes are committed
   git status

   # Create test tag
   git tag v0.1.0-rc1
   git push origin v0.1.0-rc1
   ```

2. Monitor release workflow:
   - Go to repo → Actions → "Release"
   - Verify all build jobs complete:
     - [ ] x86_64-unknown-linux-gnu
     - [ ] aarch64-unknown-linux-gnu
     - [ ] x86_64-apple-darwin
     - [ ] aarch64-apple-darwin
     - [ ] x86_64-pc-windows-msvc
     - [ ] aarch64-pc-windows-msvc
   - Verify Debian package builds
   - Verify checksums generated

3. Verify GitHub Release:
   - Go to repo → Releases
   - Check release was created with:
     - [ ] 6 platform binaries (.tar.gz or .zip)
     - [ ] 1 Debian package (.deb)
     - [ ] SHA256SUMS.txt
     - [ ] Auto-generated release notes

4. Download and test a binary:
   ```bash
   # Download for your platform
   curl -LO https://github.com/scbrown/pixelsrc/releases/download/v0.1.0-rc1/pxl-v0.1.0-rc1-aarch64-apple-darwin.tar.gz
   tar xzf pxl-v0.1.0-rc1-aarch64-apple-darwin.tar.gz
   ./pxl --version
   ```

5. Cleanup (if this was just a test):
   ```bash
   # Delete test release and tag
   gh release delete v0.1.0-rc1 --yes
   git tag -d v0.1.0-rc1
   git push --delete origin v0.1.0-rc1
   ```

**Verification:**
```bash
# Verify release exists
gh release view v0.1.0-rc1

# Test downloaded binary
./pxl --version
```

**Dependencies:** Tasks 10.4, 10.5

---

### Task 10.7: Homebrew Tap Validation

**Wave:** 5 (parallel with 10.8)

Verify Homebrew formula updates on release.

**Deliverables:**

1. Check homebrew formula was updated:
   - After release workflow completes, the `homebrew.yml` workflow should trigger
   - Go to repo → Actions → "Update Homebrew Formula"
   - Verify it committed updated formula

2. Verify formula in repo:
   ```bash
   cat homebrew-pixelsrc/Formula/pxl.rb
   # Should have updated version and SHA256 hashes
   ```

3. Test Homebrew installation (optional, requires Mac):
   ```bash
   # Add tap (using your username)
   brew tap scbrown/pixelsrc ./homebrew-pixelsrc

   # Install
   brew install pxl

   # Verify
   pxl --version
   ```

**Note:** If the homebrew-pixelsrc directory is meant to be a separate repo (common pattern), you'll need to:
1. Create separate `homebrew-pixelsrc` repository
2. Update `homebrew.yml` to push to that repo instead
3. Generate a PAT with repo access for cross-repo commits

**Verification:**
```bash
# Check formula file
cat homebrew-pixelsrc/Formula/pxl.rb
```

**Dependencies:** Task 10.6

---

### Task 10.8: Documentation & Badges

**Wave:** 5 (parallel with 10.7)

Update documentation with GitHub-specific links and badges.

**Deliverables:**

1. Add CI badges to README.md:
   ```markdown
   # Pixelsrc

   [![Release](https://img.shields.io/github/v/release/scbrown/pixelsrc)](https://github.com/scbrown/pixelsrc/releases)
   [![Build](https://github.com/scbrown/pixelsrc/actions/workflows/wasm.yml/badge.svg)](https://github.com/scbrown/pixelsrc/actions/workflows/wasm.yml)
   [![npm](https://img.shields.io/npm/v/@pixelsrc/wasm)](https://www.npmjs.com/package/@pixelsrc/wasm)
   [![License](https://img.shields.io/github/license/scbrown/pixelsrc)](LICENSE)

   ...
   ```

2. Update Cargo.toml repository field:
   ```toml
   [package]
   repository = "https://github.com/scbrown/pixelsrc"
   ```

3. Update all package.json files:
   ```json
   {
     "repository": {
       "type": "git",
       "url": "https://github.com/scbrown/pixelsrc.git"
     },
     "bugs": {
       "url": "https://github.com/scbrown/pixelsrc/issues"
     },
     "homepage": "https://github.com/scbrown/pixelsrc#readme"
   }
   ```

4. Add "Try it online" link:
   ```markdown
   ## Try It Online

   [Launch the web editor](https://scbrown.github.io/pixelsrc/) - no installation required!
   ```

5. Update installation instructions with new URLs:
   - Homebrew: `brew tap scbrown/pixelsrc && brew install pxl`
   - Binary downloads: Link to releases page
   - npm: `npm install @pixelsrc/wasm`

**Verification:**
```bash
# Verify badges render correctly
open https://github.com/scbrown/pixelsrc
```

**Dependencies:** Tasks 10.6, 10.7

---

## Post-Migration Optional Enhancements

### Custom Domain

To use a custom domain (e.g., pixelsrc.dev):

1. Purchase domain
2. Configure DNS:
   - CNAME record: `www` → `scbrown.github.io`
   - A records for apex domain:
     - 185.199.108.153
     - 185.199.109.153
     - 185.199.110.153
     - 185.199.111.153
3. Add `website/public/CNAME` file:
   ```
   pixelsrc.dev
   ```
4. Update base path in `vite.config.ts`:
   ```typescript
   base: '/',  // Root path for custom domain
   ```
5. Configure in GitHub Pages settings

### Separate Homebrew Tap Repository

For cleaner tap management:

1. Create `homebrew-pixelsrc` as separate repo
2. Update `homebrew.yml` to:
   - Clone the tap repo
   - Update formula
   - Push to tap repo
   - Requires PAT with repo scope

### Crates.io Publishing

To publish to crates.io:

1. Login: `cargo login`
2. Verify Cargo.toml metadata is complete
3. Dry run: `cargo publish --dry-run`
4. Publish: `cargo publish`
5. Add to CI (optional)

---

## Verification Summary

```bash
# Wave 1: Repo setup
git remote -v  # Should show GitHub URL
open https://github.com/scbrown/pixelsrc

# Wave 2: Configuration
# Check GitHub Settings → Secrets (NPM_TOKEN exists)
# Check GitHub Settings → Pages (enabled)

# Wave 3: CI validation
# Website live at https://scbrown.github.io/pixelsrc/
# npm package at https://www.npmjs.com/package/@pixelsrc/wasm

# Wave 4: Release validation
gh release list  # Should show releases
./pxl --version  # Downloaded binary works

# Wave 5: Post-release
cat homebrew-pixelsrc/Formula/pxl.rb  # Updated formula
# Badges render on README
```

---

## Troubleshooting

### Website 404

- Check base path in `vite.config.ts` matches repo name
- Ensure GitHub Pages source is "GitHub Actions"
- Check workflow completed successfully

### npm Publish Fails

- Verify NPM_TOKEN secret is set
- Check npm scope exists and you have access
- Try publishing manually first to debug

### Release Builds Fail

- Check for platform-specific code issues
- Verify cross-compilation setup for Linux ARM
- Check `cross` tool is installed correctly

### Homebrew Formula Not Updating

- Verify release was published (not draft)
- Check `homebrew.yml` workflow runs
- Verify git permissions for pushing
