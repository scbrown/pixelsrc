# Phase 9: Package Distribution

**Goal:** Easy installation via package managers (Homebrew, Debian, AUR, etc.)

**Status:** Planning

**Depends on:** Phase 2 complete (CLI binary)

---

## Scope

Phase 9 enables installation through popular package managers:
- **Homebrew** - macOS and Linux
- **Debian/apt** - Ubuntu, Debian, and derivatives
- **AUR** - Arch Linux
- **Cargo** - Already works via `cargo install`
- **npm** - Handled by Phase 6
- **Windows** - Scoop and Chocolatey

**Not in scope:** Snap, Flatpak, AppImage, Docker

---

## Task Dependency Diagram

```
                          PHASE 9 TASK FLOW
═══════════════════════════════════════════════════════════════════

PREREQUISITE
┌─────────────────────────────────────────────────────────────────┐
│                      Phase 2 Complete                           │
│                (CLI binary builds and works)                    │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 1 (Foundation - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   9.1 Release      │  │   9.2 Binary       │                 │
│  │   Automation       │  │   Builds           │                 │
│  │   - GitHub Actions │  │   - Cross-compile  │                 │
│  │   - Tag workflow   │  │   - Multiple arch  │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
WAVE 2 (Package Managers - Parallel)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   9.3 Homebrew     │  │   9.4 Debian       │                 │
│  │   - Formula        │  │   - .deb package   │                 │
│  │   - Tap repo       │  │   - Release asset  │                 │
│  └────────────────────┘  └────────────────────┘                 │
│                                                                 │
│  ┌────────────────────┐  ┌────────────────────┐                 │
│  │   9.5 AUR          │  │   9.6 Windows      │                 │
│  │   - PKGBUILD       │  │   - Scoop manifest │                 │
│  │   - AUR submit     │  │   - Chocolatey pkg │                 │
│  └────────────────────┘  └────────────────────┘                 │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: 9.1 + 9.2        (2 tasks in parallel)                 │
│  Wave 2: 9.3 + 9.4 + 9.5 + 9.6 (4 tasks in parallel)           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Tasks

### Task 9.1: Release Automation

**Wave:** 1 (parallel with 9.2)

Set up GitHub Actions workflow for releases.

**Deliverables:**

1. Create `.github/workflows/release.yml`:
   ```yaml
   name: Release

   on:
     push:
       tags:
         - 'v*'

   permissions:
     contents: write

   jobs:
     create-release:
       runs-on: ubuntu-latest
       outputs:
         upload_url: ${{ steps.create_release.outputs.upload_url }}
       steps:
         - name: Create Release
           id: create_release
           uses: softprops/action-gh-release@v1
           with:
             draft: false
             prerelease: false
             generate_release_notes: true

     build:
       needs: create-release
       strategy:
         matrix:
           include:
             - os: ubuntu-latest
               target: x86_64-unknown-linux-gnu
               artifact: pxl-linux-x86_64
             - os: ubuntu-latest
               target: aarch64-unknown-linux-gnu
               artifact: pxl-linux-aarch64
             - os: macos-latest
               target: x86_64-apple-darwin
               artifact: pxl-macos-x86_64
             - os: macos-latest
               target: aarch64-apple-darwin
               artifact: pxl-macos-aarch64
             - os: windows-latest
               target: x86_64-pc-windows-msvc
               artifact: pxl-windows-x86_64.exe

       runs-on: ${{ matrix.os }}
       steps:
         - uses: actions/checkout@v4

         - name: Install Rust
           uses: dtolnay/rust-action@stable
           with:
             targets: ${{ matrix.target }}

         - name: Install cross-compilation tools
           if: matrix.target == 'aarch64-unknown-linux-gnu'
           run: |
             sudo apt-get update
             sudo apt-get install -y gcc-aarch64-linux-gnu

         - name: Build
           run: cargo build --release --target ${{ matrix.target }}

         - name: Rename binary (Unix)
           if: runner.os != 'Windows'
           run: mv target/${{ matrix.target }}/release/pxl ${{ matrix.artifact }}

         - name: Rename binary (Windows)
           if: runner.os == 'Windows'
           run: mv target/${{ matrix.target }}/release/pxl.exe ${{ matrix.artifact }}

         - name: Upload Release Asset
           uses: softprops/action-gh-release@v1
           with:
             files: ${{ matrix.artifact }}

     checksums:
       needs: build
       runs-on: ubuntu-latest
       steps:
         - name: Download all artifacts
           uses: actions/download-artifact@v4

         - name: Generate checksums
           run: |
             sha256sum pxl-* > checksums.txt
             cat checksums.txt

         - name: Upload checksums
           uses: softprops/action-gh-release@v1
           with:
             files: checksums.txt
   ```

2. Add release instructions to `CONTRIBUTING.md`:
   ```markdown
   ## Creating a Release

   1. Update version in `Cargo.toml`
   2. Commit: `git commit -am "Bump version to X.Y.Z"`
   3. Tag: `git tag vX.Y.Z`
   4. Push: `git push && git push --tags`
   5. GitHub Actions will build and create the release
   ```

**Verification:**
```bash
# Create test tag
git tag v0.1.0-test
git push --tags

# Check GitHub Actions for release workflow
# Delete test tag after verification
git tag -d v0.1.0-test
git push --delete origin v0.1.0-test
```

**Dependencies:** Phase 2 complete

---

### Task 9.2: Binary Builds

**Wave:** 1 (parallel with 9.1)

Configure cross-compilation for multiple platforms.

**Deliverables:**

1. Add cross-compilation config `.cargo/config.toml`:
   ```toml
   [target.aarch64-unknown-linux-gnu]
   linker = "aarch64-linux-gnu-gcc"

   [target.x86_64-unknown-linux-gnu]
   linker = "x86_64-linux-gnu-gcc"
   ```

2. Update `Cargo.toml` with profile settings:
   ```toml
   [profile.release]
   lto = true
   strip = true
   codegen-units = 1
   panic = "abort"
   ```

3. Test local builds:
   ```bash
   # Native build
   cargo build --release

   # Check binary size
   ls -lh target/release/pxl

   # Verify it runs
   ./target/release/pxl --version
   ```

**Verification:**
```bash
# Build for current platform
cargo build --release

# Check binary
file target/release/pxl
./target/release/pxl --version
```

**Dependencies:** Phase 2 complete

---

### Task 9.3: Homebrew Formula

**Wave:** 2 (parallel with 9.4, 9.5, 9.6)

Create Homebrew tap and formula.

**Deliverables:**

1. Create separate repo `homebrew-pixelsrc` with `Formula/pxl.rb`:
   ```ruby
   class Pxl < Formula
     desc "GenAI-native pixel art format and renderer"
     homepage "https://github.com/user/pixelsrc"
     version "0.1.0"
     license "MIT"

     on_macos do
       on_arm do
         url "https://github.com/user/pixelsrc/releases/download/v#{version}/pxl-macos-aarch64"
         sha256 "PLACEHOLDER_SHA256_ARM"
       end
       on_intel do
         url "https://github.com/user/pixelsrc/releases/download/v#{version}/pxl-macos-x86_64"
         sha256 "PLACEHOLDER_SHA256_INTEL"
       end
     end

     on_linux do
       on_arm do
         url "https://github.com/user/pixelsrc/releases/download/v#{version}/pxl-linux-aarch64"
         sha256 "PLACEHOLDER_SHA256_LINUX_ARM"
       end
       on_intel do
         url "https://github.com/user/pixelsrc/releases/download/v#{version}/pxl-linux-x86_64"
         sha256 "PLACEHOLDER_SHA256_LINUX_INTEL"
       end
     end

     def install
       bin.install "pxl-macos-aarch64" => "pxl" if OS.mac? && Hardware::CPU.arm?
       bin.install "pxl-macos-x86_64" => "pxl" if OS.mac? && Hardware::CPU.intel?
       bin.install "pxl-linux-aarch64" => "pxl" if OS.linux? && Hardware::CPU.arm?
       bin.install "pxl-linux-x86_64" => "pxl" if OS.linux? && Hardware::CPU.intel?
     end

     test do
       assert_match "pxl", shell_output("#{bin}/pxl --version")
     end
   end
   ```

2. Alternative: Build from source formula:
   ```ruby
   class Pxl < Formula
     desc "GenAI-native pixel art format and renderer"
     homepage "https://github.com/user/pixelsrc"
     url "https://github.com/user/pixelsrc/archive/refs/tags/v0.1.0.tar.gz"
     sha256 "PLACEHOLDER_SHA256"
     license "MIT"

     depends_on "rust" => :build

     def install
       system "cargo", "install", *std_cargo_args
     end

     test do
       assert_match "pxl", shell_output("#{bin}/pxl --version")
     end
   end
   ```

3. Create GitHub Action to update formula on release:
   ```yaml
   # In main repo: .github/workflows/homebrew.yml
   name: Update Homebrew

   on:
     release:
       types: [published]

   jobs:
     update-formula:
       runs-on: ubuntu-latest
       steps:
         - name: Update Homebrew formula
           uses: mislav/bump-homebrew-formula-action@v3
           with:
             formula-name: pxl
             homebrew-tap: user/homebrew-pixelsrc
             download-url: https://github.com/user/pixelsrc/archive/refs/tags/${{ github.ref_name }}.tar.gz
           env:
             COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
   ```

**Verification:**
```bash
# Add tap
brew tap user/pixelsrc

# Install
brew install pxl

# Verify
pxl --version
```

**Dependencies:** Task 9.1

---

### Task 9.4: Debian Package

**Wave:** 2 (parallel with 9.3, 9.5, 9.6)

Create .deb package for Debian/Ubuntu.

**Deliverables:**

1. Create `debian/` directory in main repo:
   ```
   debian/
   ├── control
   ├── rules
   ├── changelog
   ├── copyright
   └── pxl.install
   ```

2. `debian/control`:
   ```
   Source: pixelsrc
   Section: graphics
   Priority: optional
   Maintainer: Your Name <you@example.com>
   Build-Depends: debhelper-compat (= 13), cargo, rustc
   Standards-Version: 4.6.0
   Homepage: https://github.com/user/pixelsrc

   Package: pxl
   Architecture: any
   Depends: ${shlibs:Depends}, ${misc:Depends}
   Description: GenAI-native pixel art format and renderer
    PixelSrc (pxl) is a text-based pixel art format designed for
    AI generation. It converts JSONL sprite definitions to PNG images.
   ```

3. `debian/rules`:
   ```makefile
   #!/usr/bin/make -f

   %:
   	dh $@

   override_dh_auto_build:
   	cargo build --release

   override_dh_auto_install:
   	install -D -m 755 target/release/pxl debian/pxl/usr/bin/pxl
   ```

4. Add to release workflow:
   ```yaml
   # In .github/workflows/release.yml, add job:
   build-deb:
     needs: create-release
     runs-on: ubuntu-latest
     steps:
       - uses: actions/checkout@v4

       - name: Install dependencies
         run: |
           sudo apt-get update
           sudo apt-get install -y devscripts debhelper

       - name: Install Rust
         uses: dtolnay/rust-action@stable

       - name: Build .deb
         run: |
           cargo build --release
           mkdir -p pkg/DEBIAN pkg/usr/bin
           cp target/release/pxl pkg/usr/bin/
           cat > pkg/DEBIAN/control << EOF
           Package: pxl
           Version: ${GITHUB_REF_NAME#v}
           Architecture: amd64
           Maintainer: Your Name <you@example.com>
           Description: GenAI-native pixel art format and renderer
           EOF
           dpkg-deb --build pkg pxl_${GITHUB_REF_NAME#v}_amd64.deb

       - name: Upload .deb
         uses: softprops/action-gh-release@v1
         with:
           files: pxl_*.deb
   ```

**Verification:**
```bash
# Download .deb from release
wget https://github.com/user/pixelsrc/releases/download/v0.1.0/pxl_0.1.0_amd64.deb

# Install
sudo dpkg -i pxl_0.1.0_amd64.deb

# Verify
pxl --version
```

**Dependencies:** Task 9.1

---

### Task 9.5: AUR Package

**Wave:** 2 (parallel with 9.3, 9.4, 9.6)

Create Arch Linux AUR package.

**Deliverables:**

1. Create `PKGBUILD`:
   ```bash
   # Maintainer: Your Name <you@example.com>
   pkgname=pixelsrc
   pkgver=0.1.0
   pkgrel=1
   pkgdesc="GenAI-native pixel art format and renderer"
   arch=('x86_64' 'aarch64')
   url="https://github.com/user/pixelsrc"
   license=('MIT')
   makedepends=('cargo')
   source=("$pkgname-$pkgver.tar.gz::https://github.com/user/pixelsrc/archive/v$pkgver.tar.gz")
   sha256sums=('PLACEHOLDER')

   build() {
     cd "$pkgname-$pkgver"
     cargo build --release --locked
   }

   package() {
     cd "$pkgname-$pkgver"
     install -Dm755 "target/release/pxl" "$pkgdir/usr/bin/pxl"
     install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
   }
   ```

2. Create `.SRCINFO`:
   ```
   pkgbase = pixelsrc
   	pkgdesc = GenAI-native pixel art format and renderer
   	pkgver = 0.1.0
   	pkgrel = 1
   	url = https://github.com/user/pixelsrc
   	arch = x86_64
   	arch = aarch64
   	license = MIT
   	makedepends = cargo
   	source = pixelsrc-0.1.0.tar.gz::https://github.com/user/pixelsrc/archive/v0.1.0.tar.gz
   	sha256sums = PLACEHOLDER

   pkgname = pixelsrc
   ```

3. Submit to AUR:
   - Create account on https://aur.archlinux.org
   - Clone AUR package repo
   - Push PKGBUILD and .SRCINFO

4. Alternative: Binary package (pixelsrc-bin):
   ```bash
   pkgname=pixelsrc-bin
   pkgver=0.1.0
   pkgrel=1
   pkgdesc="GenAI-native pixel art format and renderer (prebuilt binary)"
   arch=('x86_64' 'aarch64')
   url="https://github.com/user/pixelsrc"
   license=('MIT')
   provides=('pixelsrc')
   conflicts=('pixelsrc')
   source_x86_64=("pxl-$pkgver-x86_64::https://github.com/user/pixelsrc/releases/download/v$pkgver/pxl-linux-x86_64")
   source_aarch64=("pxl-$pkgver-aarch64::https://github.com/user/pixelsrc/releases/download/v$pkgver/pxl-linux-aarch64")
   sha256sums_x86_64=('PLACEHOLDER')
   sha256sums_aarch64=('PLACEHOLDER')

   package() {
     if [[ "$CARCH" == "x86_64" ]]; then
       install -Dm755 "$srcdir/pxl-$pkgver-x86_64" "$pkgdir/usr/bin/pxl"
     else
       install -Dm755 "$srcdir/pxl-$pkgver-aarch64" "$pkgdir/usr/bin/pxl"
     fi
   }
   ```

**Verification:**
```bash
# Using yay
yay -S pixelsrc

# Or using paru
paru -S pixelsrc

# Verify
pxl --version
```

**Dependencies:** Task 9.1

---

### Task 9.6: Windows Packages

**Wave:** 2 (parallel with 9.3, 9.4, 9.5)

Create Scoop and Chocolatey packages for Windows.

**Deliverables:**

#### Scoop

1. Create `scoop/pxl.json` manifest:
   ```json
   {
     "version": "0.1.0",
     "description": "GenAI-native pixel art format and renderer",
     "homepage": "https://github.com/user/pixelsrc",
     "license": "MIT",
     "architecture": {
       "64bit": {
         "url": "https://github.com/user/pixelsrc/releases/download/v0.1.0/pxl-windows-x86_64.exe",
         "hash": "PLACEHOLDER_SHA256"
       }
     },
     "bin": [["pxl-windows-x86_64.exe", "pxl"]],
     "checkver": {
       "github": "https://github.com/user/pixelsrc"
     },
     "autoupdate": {
       "architecture": {
         "64bit": {
           "url": "https://github.com/user/pixelsrc/releases/download/v$version/pxl-windows-x86_64.exe"
         }
       }
     }
   }
   ```

2. Submit to Scoop extras bucket or create own bucket

#### Chocolatey

1. Create `chocolatey/` directory:
   ```
   chocolatey/
   ├── pxl.nuspec
   └── tools/
       └── chocolateyinstall.ps1
   ```

2. `chocolatey/pxl.nuspec`:
   ```xml
   <?xml version="1.0" encoding="utf-8"?>
   <package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
     <metadata>
       <id>pxl</id>
       <version>0.1.0</version>
       <title>PixelSrc (pxl)</title>
       <authors>Your Name</authors>
       <projectUrl>https://github.com/user/pixelsrc</projectUrl>
       <licenseUrl>https://github.com/user/pixelsrc/blob/main/LICENSE</licenseUrl>
       <requireLicenseAcceptance>false</requireLicenseAcceptance>
       <description>GenAI-native pixel art format and renderer</description>
       <tags>pixel-art cli renderer genai</tags>
     </metadata>
     <files>
       <file src="tools\**" target="tools" />
     </files>
   </package>
   ```

3. `chocolatey/tools/chocolateyinstall.ps1`:
   ```powershell
   $ErrorActionPreference = 'Stop'
   $toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
   $url64 = 'https://github.com/user/pixelsrc/releases/download/v0.1.0/pxl-windows-x86_64.exe'

   $packageArgs = @{
     packageName   = $env:ChocolateyPackageName
     fileFullPath  = "$toolsDir\pxl.exe"
     url64bit      = $url64
     checksum64    = 'PLACEHOLDER_SHA256'
     checksumType64= 'sha256'
   }

   Get-ChocolateyWebFile @packageArgs
   ```

4. Add to release workflow:
   ```yaml
   update-packages:
     needs: build
     runs-on: ubuntu-latest
     steps:
       - uses: actions/checkout@v4

       - name: Update Scoop manifest
         run: |
           # Update version and hash in scoop/pxl.json
           # Commit and push to bucket repo

       - name: Update Chocolatey package
         run: |
           # Update version and hash
           # Push to Chocolatey
   ```

**Verification:**
```powershell
# Scoop
scoop bucket add pixelsrc https://github.com/user/scoop-pixelsrc
scoop install pxl
pxl --version

# Chocolatey
choco install pxl
pxl --version
```

**Dependencies:** Task 9.1

---

## Installation Summary

After Phase 9, users can install via:

| Platform | Method | Command |
|----------|--------|---------|
| Any | Cargo | `cargo install pixelsrc` |
| Any | npm | `npm install -g @pixelsrc/wasm` |
| macOS | Homebrew | `brew install user/pixelsrc/pxl` |
| Linux (Debian) | apt | `sudo dpkg -i pxl_*.deb` |
| Linux (Arch) | AUR | `yay -S pixelsrc` |
| Windows | Scoop | `scoop install pxl` |
| Windows | Chocolatey | `choco install pxl` |

---

## Verification Summary

```bash
# 1. Release workflow creates binaries
git tag v0.1.0 && git push --tags
# Check GitHub releases page

# 2. Homebrew works
brew tap user/pixelsrc && brew install pxl

# 3. Debian package works
sudo dpkg -i pxl_0.1.0_amd64.deb

# 4. AUR package builds
yay -S pixelsrc

# 5. Windows packages work (on Windows)
scoop install pxl
choco install pxl
```

---

## Future Considerations

Features considered but not included in Phase 9:

| Feature | Rationale for Deferral |
|---------|------------------------|
| Snap package | Ubuntu prioritizes .deb; Snap has size overhead |
| Flatpak | Desktop apps focus; pxl is CLI tool |
| Docker image | Can add later if requested |
| Nix package | Smaller user base; can add later |
| APT repository | Start with .deb in releases; PPA is more complex |
