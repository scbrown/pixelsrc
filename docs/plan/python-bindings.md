# Python Bindings (PyO3 + Maturin)

**Goal:** Ship a native Python package (`pixelsrc`) that exposes the core pixelsrc API — parsing, rendering, validation, registry operations, and PNG import — via PyO3 FFI bindings, published to PyPI with prebuilt wheels.

**Status:** Planning

**Depends on:** None (additive — no changes to existing Rust code required)

---

## Motivation

Pixelsrc already has WASM bindings for browser/Node.js usage. Python is the dominant language in GenAI toolchains (Stable Diffusion pipelines, ComfyUI nodes, training data prep, Jupyter notebooks). Native Python bindings would let users:

- **Generate sprites programmatically** in Python scripts and notebooks
- **Integrate into GenAI pipelines** — render pixel art as part of training data generation
- **Batch process** `.pxl` files with full Rust performance
- **Use in game toolchains** — Python build scripts that produce sprite atlases
- **Access validation** — lint `.pxl` files from Python CI/CD
- **Import existing PNGs** — convert raster pixel art to `.pxl` format with semantic analysis

The WASM bindings (`src/wasm.rs`) already prove the pattern: a thin FFI layer over the core API. The Python bindings mirror that structure using PyO3 instead of `wasm-bindgen`.

---

## Architecture

```
Cargo.toml (add pyo3 dep + "python" feature)
│
├── src/python/
│   ├── mod.rs          ← #[pymodule] entry point
│   ├── types.rs        ← Python wrapper types (PySprite, PyPalette, etc.)
│   ├── parse.rs        ← Parsing functions
│   ├── render.rs       ← Rendering functions
│   ├── registry.rs     ← Registry wrappers (stateful)
│   ├── validate.rs     ← Validation functions
│   ├── color.rs        ← Color utilities
│   └── import.rs       ← PNG import + analysis
│
├── python/
│   ├── pyproject.toml  ← Maturin build config
│   ├── pixelsrc/
│   │   ├── __init__.py ← Re-exports from native module
│   │   └── py.typed    ← PEP 561 marker
│   ├── pixelsrc.pyi    ← Type stubs for IDE support
│   └── tests/
│       ├── test_parse.py
│       ├── test_render.py
│       ├── test_validate.py
│       ├── test_registry.py
│       ├── test_color.py
│       └── test_import.py
│
└── .github/workflows/
    └── python.yml      ← CI: build wheels + publish to PyPI
```

### Binding Strategy

Follow the same pattern as `src/wasm.rs`: expose high-level functions that accept strings/bytes and return simple Python-native types. Avoid leaking complex Rust lifetimes across the FFI boundary.

**Two tiers of API:**

1. **Stateless functions** (like WASM) — parse a string, render to bytes, validate
2. **Stateful registry** — build up palettes/sprites, resolve references, render resolved sprites

```
Tier 1 (Stateless):                    Tier 2 (Stateful):
┌─────────────────────┐                ┌──────────────────────────┐
│ render_to_png(pxl)  │                │ reg = Registry()         │
│ render_to_rgba(pxl) │                │ reg.load(pxl_string)     │
│ validate(pxl)       │                │ reg.load_file("hero.pxl")│
│ list_sprites(pxl)   │                │ img = reg.render("hero") │
│ parse(pxl)          │                │ reg.sprites()            │
│ format(pxl)         │                │ reg.palettes()           │
│ import_png(path)    │                │                          │
│ import_png_analyzed()│                │                          │
└─────────────────────┘                └──────────────────────────┘
```

### Type Mapping

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `String` | `str` | Direct |
| `Vec<u8>` (PNG) | `bytes` | PNG file contents |
| `Vec<u8>` (RGBA) | `bytes` | Raw pixel data |
| `RgbaImage` | `RenderResult` | width + height + pixels + warnings |
| `Vec<String>` | `list[str]` | Warning/error messages |
| `HashMap<String, String>` | `dict[str, str]` | Palette colors |
| `TtpObject` | `dict` | Serialized via serde |
| `ParseResult` | `ParseResult` | Custom Python class |
| `Warning` | `str` | Formatted message |
| `ImportResult` | `ImportResult` | Custom Python class |
| `ImportAnalysis` | `dict` | Serialized via serde (roles, relationships, symmetry, etc.) |
| `ImportOptions` | kwargs | Python keyword arguments mapped to struct fields |
| `DitherInfo` | `dict` | Detection result |
| `UpscaleInfo` | `dict` | Detection result |
| `OutlineInfo` | `dict` | Detection result |

---

## Configuration

### Cargo.toml Changes

```toml
[features]
default = ["lsp"]
lsp = ["tower-lsp", "tokio"]
wasm = ["wasm-bindgen", "console_error_panic_hook"]
python = ["pyo3"]

[dependencies]
pyo3 = { version = "0.23", features = ["extension-module"], optional = true }
```

The `python` feature follows the same pattern as `wasm` — conditional compilation with `#[cfg(feature = "python")]`.

### pyproject.toml

```toml
[build-system]
requires = ["maturin>=1.7,<2.0"]
build-backend = "maturin"

[project]
name = "pixelsrc"
requires-python = ">=3.9"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]
description = "Semantic pixel art format and compiler"
license = { text = "MIT" }

[tool.maturin]
features = ["python"]
module-name = "pixelsrc._native"
```

The native module is imported as `pixelsrc._native`, with `__init__.py` re-exporting a clean public API.

---

## Task Breakdown

### Task PY-1: Cargo.toml + Feature Flag Setup

**Wave:** 1 (Foundation)

Add the `pyo3` dependency and `python` feature flag to `Cargo.toml`. Create the `src/python/mod.rs` skeleton with `#[pymodule]` entry point. Verify that `cargo check --features python` succeeds and existing builds (`cargo build`, `cargo build --features wasm`) are unaffected.

**Deliverables:**
- `Cargo.toml` updated with `pyo3` dep and `python` feature
- `src/python/mod.rs` — module entry with empty `#[pymodule]`
- `src/lib.rs` — conditional `pub mod python;`

**Verification:**
```bash
cargo check --features python
cargo check                     # default (lsp) still works
cargo check --no-default-features --features wasm  # wasm still works
```

**Dependencies:** None

---

### Task PY-2: Python Package Scaffolding

**Wave:** 1 (Foundation — parallel with PY-1)

Create the `python/` directory with `pyproject.toml`, `__init__.py`, type stubs, and test scaffolding. Verify that `maturin develop` builds and installs the package locally.

**Deliverables:**
- `python/pyproject.toml` — maturin build config
- `python/pixelsrc/__init__.py` — public API re-exports
- `python/pixelsrc/py.typed` — PEP 561 marker
- `python/pixelsrc.pyi` — type stub skeleton
- `python/tests/` — test directory with conftest.py

**Verification:**
```bash
cd python && maturin develop --features python
python -c "import pixelsrc; print(pixelsrc.__version__)"
```

**Dependencies:** None

---

### Task PY-3: Stateless Rendering Functions

**Wave:** 2 (Core API)

Implement the stateless rendering functions mirroring `src/wasm.rs`. These are the highest-value bindings — they let Python users go from `.pxl` string to PNG bytes in one call.

**Deliverables:**
- `src/python/render.rs`:
  - `render_to_png(pxl: &str) -> PyResult<Vec<u8>>` — returns PNG bytes
  - `render_to_rgba(pxl: &str) -> PyResult<RenderResult>` — returns RGBA + metadata
- `src/python/types.rs`:
  - `RenderResult` — `#[pyclass]` with `width`, `height`, `pixels`, `warnings`

**Verification:**
```bash
cd python && maturin develop --features python
python -c "
from pixelsrc import render_to_png
png = render_to_png('''
{ type: 'sprite', name: 'dot', width: 2, height: 2,
  regions: [{ shape: 'rect', x: 0, y: 0, w: 2, h: 2, color: '#ff0000' }] }
''')
assert len(png) > 0
print(f'PNG: {len(png)} bytes')
"
```

**Dependencies:** PY-1

---

### Task PY-4: Parsing and Listing Functions

**Wave:** 2 (Core API — parallel with PY-3)

Expose parsing functions that let users inspect `.pxl` content without rendering.

**Deliverables:**
- `src/python/parse.rs`:
  - `parse(pxl: &str) -> PyResult<Vec<PyObject>>` — returns list of parsed objects as dicts
  - `list_sprites(pxl: &str) -> PyResult<Vec<String>>` — sprite names
  - `list_palettes(pxl: &str) -> PyResult<Vec<String>>` — palette names

**Verification:**
```bash
python -c "
from pixelsrc import list_sprites, parse
names = list_sprites('{ type: \"sprite\", name: \"hero\", width: 8, height: 8, regions: [] }')
assert names == ['hero']
"
```

**Dependencies:** PY-1

---

### Task PY-5: Validation Functions

**Wave:** 2 (Core API — parallel with PY-3, PY-4)

Expose validation so Python CI pipelines can lint `.pxl` files.

**Deliverables:**
- `src/python/validate.rs`:
  - `validate(pxl: &str) -> PyResult<Vec<String>>` — returns warning/error messages
  - `validate_file(path: &str) -> PyResult<Vec<String>>` — validate from file path

**Verification:**
```bash
python -c "
from pixelsrc import validate
# Valid input
warnings = validate('{ type: \"sprite\", name: \"ok\", width: 8, height: 8, regions: [] }')
print(f'Warnings: {warnings}')
"
```

**Dependencies:** PY-1

---

### Task PY-6: Color Utilities

**Wave:** 2 (Core API — parallel with PY-3, PY-4, PY-5)

Expose color parsing and ramp generation. Useful for procedural palette generation in Python.

**Deliverables:**
- `src/python/color.rs`:
  - `parse_color(color_str: &str) -> PyResult<String>` — parse any CSS color to hex
  - `generate_ramp(from_color: &str, to_color: &str, steps: usize) -> PyResult<Vec<String>>` — color ramp

**Verification:**
```bash
python -c "
from pixelsrc import parse_color, generate_ramp
assert parse_color('red') == '#ff0000'
ramp = generate_ramp('#000000', '#ffffff', 5)
assert len(ramp) == 5
"
```

**Dependencies:** PY-1

---

### Task PY-7: PNG Import and Analysis

**Wave:** 2 (Core API — parallel with PY-3, PY-4, PY-5, PY-6)

Expose the PNG import pipeline: convert raster pixel art to `.pxl` format, with optional semantic analysis (role inference, symmetry detection, dither/upscale/outline detection, structured region extraction). This wraps `import_png_with_options()` from `src/import/mod.rs`.

**Deliverables:**
- `src/python/import.rs`:
  - `import_png(path: &str, name: Option<&str>, max_colors: Option<u32>) -> PyResult<ImportResult>` — basic import
  - `import_png_analyzed(path: &str, name: Option<&str>, max_colors: Option<u32>, confidence: Option<f64>, hints: Option<bool>, shapes: Option<bool>, detect_upscale: Option<bool>, detect_outlines: Option<bool>, dither_handling: Option<&str>) -> PyResult<ImportResult>` — import with analysis
- `src/python/types.rs` additions:
  - `ImportResult` — `#[pyclass]` with:
    - `name: String` — sprite name
    - `width: u32`, `height: u32`
    - `palette: dict[str, str]` — token-to-hex color map
    - `to_pxl() -> str` — serialize to `.pxl` format (structured JSONL)
    - `to_jsonl() -> str` — serialize to legacy JSONL
    - `analysis: Option<dict>` — analysis results if enabled
  - Analysis dict contains: `roles`, `relationships`, `symmetry`, `naming_hints`, `z_order`, `dither_patterns`, `upscale_info`, `outlines`

**Verification:**
```bash
# Create a test PNG first
python -c "
from pixelsrc import render_to_png
png = render_to_png('{ type: \"sprite\", name: \"test\", width: 8, height: 8, regions: [{ shape: \"rect\", x: 0, y: 0, w: 8, h: 8, color: \"#ff0000\" }] }')
open('/tmp/test_sprite.png', 'wb').write(png)
"

# Then import it back
python -c "
from pixelsrc import import_png, import_png_analyzed

# Basic import
result = import_png('/tmp/test_sprite.png')
print(f'Sprite: {result.name}, {result.width}x{result.height}')
print(f'Palette: {result.palette}')
print(result.to_pxl())

# Import with analysis
analyzed = import_png_analyzed('/tmp/test_sprite.png', confidence=0.7, shapes=True)
print(f'Analysis: {analyzed.analysis}')
"
```

**Dependencies:** PY-1

---

### Task PY-8: Stateful Registry

**Wave:** 3 (Advanced API)

Implement a `Registry` class that mirrors the Rust `PaletteRegistry` + `SpriteRegistry` pattern. This is the power-user API for working with multiple sprites and palettes that reference each other.

**Deliverables:**
- `src/python/registry.rs`:
  - `PyRegistry` — `#[pyclass]` wrapping owned registries
  - Methods:
    - `load(pxl: &str)` — parse and register all objects
    - `load_file(path: &str)` — load from file
    - `sprites() -> Vec<String>` — list registered sprites
    - `palettes() -> Vec<String>` — list registered palettes
    - `render(name: &str) -> RenderResult` — render a named sprite
    - `render_to_png(name: &str) -> Vec<u8>` — render to PNG bytes
    - `render_all() -> dict[str, bytes]` — render all sprites to PNG

**Verification:**
```bash
python -c "
from pixelsrc import Registry
reg = Registry()
reg.load('''
{ type: 'palette', name: 'warm', colors: { primary: '#ff4400', bg: '#331100' } }
{ type: 'sprite', name: 'dot', width: 4, height: 4, palette: 'warm',
  regions: [{ shape: 'circle', cx: 2, cy: 2, r: 2, color: 'primary' }] }
''')
assert 'dot' in reg.sprites()
assert 'warm' in reg.palettes()
png = reg.render_to_png('dot')
assert len(png) > 0
"
```

**Dependencies:** PY-3, PY-4

---

### Task PY-9: Formatting Function

**Wave:** 3 (Advanced API — parallel with PY-8)

Expose the `.pxl` formatter so Python tools can auto-format pixel art source.

**Deliverables:**
- Add to `src/python/parse.rs` or new file:
  - `format_pxl(pxl: &str) -> PyResult<String>` — format `.pxl` source

**Verification:**
```bash
python -c "
from pixelsrc import format_pxl
formatted = format_pxl('{type:\"sprite\",name:\"x\",width:8,height:8,regions:[]}')
print(formatted)
"
```

**Dependencies:** PY-1

---

### Task PY-10: Type Stubs and Documentation

**Wave:** 4 (Polish)

Complete the `.pyi` type stubs and add docstrings to all exported functions and classes. Write a `python/README.md` for PyPI.

**Deliverables:**
- `python/pixelsrc.pyi` — complete type stubs with docstrings
- `python/README.md` — PyPI package description
- Docstrings on all `#[pyfunction]` and `#[pyclass]` items

**Verification:**
```bash
# Type checking
cd python && pip install mypy
mypy --strict -c "
import pixelsrc
result = pixelsrc.render_to_png('...')
reveal_type(result)  # bytes
"
```

**Dependencies:** PY-3, PY-4, PY-5, PY-6, PY-7, PY-8, PY-9

---

### Task PY-11: Python Test Suite

**Wave:** 4 (Polish — parallel with PY-10)

Comprehensive Python-side tests using pytest. Mirror the patterns from the existing Rust integration tests.

**Deliverables:**
- `python/tests/test_render.py` — rendering roundtrips
- `python/tests/test_parse.py` — parsing and listing
- `python/tests/test_validate.py` — validation messages
- `python/tests/test_registry.py` — stateful registry operations
- `python/tests/test_color.py` — color parsing and ramps
- `python/tests/test_import.py` — PNG import and analysis roundtrips
- `python/tests/conftest.py` — shared fixtures (sample .pxl strings, test PNGs)

**Verification:**
```bash
cd python && maturin develop --features python && pytest -v
```

**Dependencies:** PY-3, PY-4, PY-5, PY-6, PY-7, PY-8

---

### Task PY-12: CI/CD and Wheel Builds

**Wave:** 5 (Release)

Set up GitHub Actions to build wheels for all target platforms using maturin's CI action, and publish to PyPI.

**Deliverables:**
- `.github/workflows/python.yml`:
  - Build matrix: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
  - Python versions: 3.9, 3.10, 3.11, 3.12, 3.13
  - manylinux for Linux wheels
  - Publish to PyPI on tag push
- `python/MANIFEST.in` if needed

**Verification:**
```bash
# Local test of wheel build
cd python && maturin build --release --features python
ls target/wheels/
```

**Dependencies:** PY-10, PY-11

---

### Task PY-13: Optional NumPy/Pillow Integration

**Wave:** 5 (Release — parallel with PY-12)

Add optional convenience methods for interop with NumPy arrays and Pillow images — the two most common image representations in Python.

**Deliverables:**
- `python/pixelsrc/__init__.py` additions:
  - `RenderResult.to_numpy()` — returns `numpy.ndarray` with shape `(h, w, 4)`
  - `RenderResult.to_pil()` — returns `PIL.Image.Image`
- These are pure Python wrappers (no Rust changes) that convert the raw RGBA bytes

**Verification:**
```bash
python -c "
from pixelsrc import render_to_rgba
result = render_to_rgba('{ type: \"sprite\", name: \"x\", width: 4, height: 4, regions: [{shape: \"rect\", x:0, y:0, w:4, h:4, color: \"#ff0000\"}] }')
img = result.to_pil()
print(f'PIL Image: {img.size}')
arr = result.to_numpy()
print(f'NumPy: {arr.shape}')
"
```

**Dependencies:** PY-3

---

## Task Dependency Diagram

```
Wave 1 (Foundation):
  PY-1 ─────────┐
  PY-2           │
                 │
Wave 2 (Core):   │
  PY-3 ◄────────┤  (render)
  PY-4 ◄────────┤  (parse)
  PY-5 ◄────────┤  (validate)
  PY-6 ◄────────┤  (color)
  PY-7 ◄────────┘  (import)

Wave 3 (Advanced):
  PY-8 ◄──── PY-3, PY-4  (registry)
  PY-9 ◄──── PY-1        (format)

Wave 4 (Polish):
  PY-10 ◄──── PY-3..PY-9  (type stubs)
  PY-11 ◄──── PY-3..PY-8  (tests)

Wave 5 (Release):
  PY-12 ◄──── PY-10, PY-11  (CI/wheels)
  PY-13 ◄──── PY-3          (numpy/pillow)
```

| Wave | Tasks | Parallelism |
|------|-------|-------------|
| 1 | PY-1, PY-2 | 2 tasks in parallel |
| 2 | PY-3, PY-4, PY-5, PY-6, PY-7 | 5 tasks in parallel |
| 3 | PY-8, PY-9 | 2 tasks in parallel |
| 4 | PY-10, PY-11 | 2 tasks in parallel |
| 5 | PY-12, PY-13 | 2 tasks in parallel |

---

## Design Decisions

1. **Package name:** `pixelsrc` on PyPI. Matches the project name and is the obvious choice. If taken, fall back to `pixelsrc-python`.
2. **Minimum Python version:** 3.9. It's the oldest non-EOL version and PyO3 0.23 supports it. No reason to go higher and exclude users.
3. **PyPy support:** Deferred. PyO3 supports it but the CI matrix cost isn't justified until there's demand.
4. **Async support:** Deferred. Synchronous file I/O is appropriate for a pixel art compiler. Revisit if someone needs async rendering in a web server context.
5. **NumPy/Pillow:** Optional extras — `pip install pixelsrc[images]` pulls in Pillow, `pip install pixelsrc[numpy]` pulls in NumPy. Core package has zero Python dependencies.

---

## Success Criteria

- [ ] `pip install pixelsrc` works on Linux, macOS, Windows
- [ ] Stateless API: `render_to_png()`, `validate()`, `list_sprites()`, `import_png()` work from Python
- [ ] Stateful API: `Registry` class supports multi-object workflows
- [ ] Type stubs provide full IDE autocomplete and mypy compatibility
- [ ] Python test suite passes with pytest
- [ ] Prebuilt wheels for Python 3.9–3.13 on all major platforms
- [ ] Package published to PyPI
- [ ] No changes required to existing Rust code (pure addition)
