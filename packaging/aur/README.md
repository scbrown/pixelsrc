# AUR Packages for Pixelsrc

This directory contains PKGBUILD files for the Arch User Repository (AUR).

## Packages

### pixelsrc
Build from source using cargo. Requires Rust toolchain.

```bash
yay -S pixelsrc
# or
paru -S pixelsrc
```

### pixelsrc-bin
Prebuilt binary from GitHub releases. No build dependencies needed.

```bash
yay -S pixelsrc-bin
# or
paru -S pixelsrc-bin
```

## Before Submitting to AUR

1. **Update the URL**: Replace `OWNER` in the URLs with your actual GitHub username/organization:
   - In `pixelsrc/PKGBUILD` and `pixelsrc/.SRCINFO`
   - In `pixelsrc-bin/PKGBUILD` and `pixelsrc-bin/.SRCINFO`

2. **Update maintainer info**: Add your name and email in the PKGBUILD header.

3. **Update checksums**: Once you have a release:
   ```bash
   cd pixelsrc
   updpkgsums  # Updates sha256sums in PKGBUILD
   makepkg --printsrcinfo > .SRCINFO
   ```

4. **Test locally**:
   ```bash
   makepkg -si  # Build and install
   pxl --version  # Verify installation
   ```

## Submitting to AUR

1. Create an AUR account at https://aur.archlinux.org/

2. Generate SSH key and add to your AUR account

3. Clone your (empty) AUR package:
   ```bash
   git clone ssh://aur@aur.archlinux.org/pixelsrc.git
   ```

4. Copy PKGBUILD and .SRCINFO into the cloned directory

5. Commit and push:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Initial upload: pixelsrc 0.1.0"
   git push
   ```

6. Repeat for pixelsrc-bin

## Version Updates

When releasing a new version:

1. Update `pkgver` in PKGBUILD
2. Run `updpkgsums` to update checksums
3. Regenerate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
4. Commit and push to AUR
