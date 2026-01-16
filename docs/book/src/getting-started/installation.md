# Installation

Pixelsrc provides the `pxl` command-line tool for working with pixel art files.

## From Source (Rust)

If you have Rust installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/scbrown/pixelsrc.git
cd pixelsrc

# Build and install
cargo install --path .

# Verify installation
pxl --version
```

## From Releases

Pre-built binaries are available on the [GitHub Releases](https://github.com/scbrown/pixelsrc/releases) page for:

- Linux (x86_64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

Download the appropriate binary for your platform and add it to your PATH.

## Verify Installation

After installation, verify that `pxl` is available:

```bash
pxl --version
```

You should see output like:

```
pxl 0.1.0
```

## Next Steps

Once installed, head to the [Quick Start](quick-start.md) guide to create your first sprite.
