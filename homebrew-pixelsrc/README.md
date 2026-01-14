# Homebrew Tap for Pixelsrc

This tap contains the Homebrew formula for `pxl`, the Pixelsrc CLI.

## Installation

```bash
# Add the tap
brew tap stiwi/pixelsrc

# Install pxl
brew install pxl
```

## Available Formulas

### pxl (binary)

Pre-built binary for fast installation:

```bash
brew install pxl
```

Supports:
- macOS (Intel and Apple Silicon)
- Linux (x86_64 and ARM64)

### pxl-src (build from source)

Build from source (requires Rust):

```bash
brew install pxl-src
```

## Updating

```bash
brew update
brew upgrade pxl
```

## Verification

```bash
pxl --version
```

## About Pixelsrc

Pixelsrc is a GenAI-native pixel art format and compiler. See the main repository for documentation.
