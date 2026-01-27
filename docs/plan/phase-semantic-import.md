# Phase: Semantic Import Enhancement

**Goal:** Automatically identify and name discrete visual components during PNG → pxlsrc conversion using pre-trained vision models and vector embeddings

**Status:** Planning

**Depends on:** Phase 5 (PNG Import exists), Phase 15 (AI Tools foundation)

---

## Motivation

The current import pipeline (`pxl import`) converts PNG images to pxlsrc format with:
- Color-based region extraction using geometric primitives (rect, points, union, etc.)
- Role inference (background, outline, fill, etc.)
- Structural relationships (containment, adjacency)
- Generic token names (`c1`, `c2`, etc.)

**The gap:** Tokens are named by color index, not by what they visually represent. A sword is `c3`, not `sword`. A helmet is `c7`, not `helmet`.

**The goal:** Leverage pre-trained vision models to automatically identify semantic components:
- "This region looks like a sword" → `sword`
- "This region looks like armor" → `armor`
- "This cluster of regions forms a character" → grouped semantically

This enables:
1. **Meaningful editing:** When you import a PNG and later edit the pxlsrc, you're editing `sword` not `c3` - the token names communicate intent
2. **Composability:** Semantically-named components can be mixed/matched across sprites
3. **Searchability:** Find all sprites containing weapon-type components via grep/search
4. **AI generation guidance:** "Generate a sprite with helmet, sword, shield regions" - AI understands the vocabulary

---

## Personas

This feature benefits all personas, with complexity scaling appropriately:

| Persona | Benefit | Typical Usage |
|---------|---------|---------------|
| **Sketcher** | Import existing art, get meaningful names automatically | `pxl import sprite.png --semantic` |
| **Pixel Artist** | Better organization when importing reference art | Default semantic on, custom vocabulary for project |
| **Animator** | Consistent naming across imported frame sequences | Batch import with shared vocabulary |
| **Motion Designer** | Semantic regions enable intelligent transform targeting | "Apply glow to all `weapon` regions" |
| **Game Developer** | Metadata-rich imports ready for engine integration | TOML-configured vocabulary per asset type |

**Complexity by persona:**
- Sketcher: Zero config, just add `--semantic` flag
- Pixel Artist+: Custom vocabulary in `pxl.toml`
- Game Developer: Full control via TOML + CI integration

---

## Approach: Off-the-Shelf Models

Rather than training custom models, we leverage existing pre-trained models:

### Segmentation: Segment Anything Model (SAM)
- **What:** Meta's universal segmentation model
- **Why:** Zero-shot segmentation works on any image type including pixel art
- **How:** Generates candidate region masks without requiring training

### Classification: CLIP
- **What:** OpenAI's vision-language model
- **Why:** Aligns images with text descriptions in shared embedding space
- **How:** Given a region + candidate labels, returns similarity scores

### Alternative: Vision-Language Models (VLMs)
- **What:** GPT-4V, Claude Vision, LLaVA
- **Why:** Can describe image contents in natural language
- **How:** "What is this region?" → "A golden sword with a curved blade"

---

## Architecture

```
PNG Image
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│  Stage 1: Region Extraction                              │
│  ┌─────────────────┐    ┌────────────────────────────┐  │
│  │ Current Import  │ OR │ SAM Segmentation           │  │
│  │ (color-based)   │    │ (mask-based)               │  │
│  └─────────────────┘    └────────────────────────────┘  │
│                    ▼                                     │
│            Candidate Regions                             │
└─────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│  Stage 2: Semantic Classification                        │
│  ┌─────────────────┐    ┌────────────────────────────┐  │
│  │ CLIP Embedding  │ OR │ VLM Description            │  │
│  │ + Label Match   │    │ (GPT-4V/Claude)            │  │
│  └─────────────────┘    └────────────────────────────┘  │
│                    ▼                                     │
│       Region → Semantic Label Mapping                    │
└─────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│  Stage 3: Name Generation                                │
│  ┌─────────────────────────────────────────────────────┐│
│  │ Label → Token Name                                  ││
│  │ "golden sword" → sword                              ││
│  │ "metal helmet" → helmet                             ││
│  │ "dark outline" → outline                            ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
    │
    ▼
pxlsrc with Semantic Token Names
```

---

## Implementation Options

### Option A: Local Inference (Recommended for Privacy/Speed)

Run models locally via ONNX Runtime or similar:

```
pxl import image.png --semantic
    │
    ├── Load SAM (onnx) → Generate masks
    ├── Load CLIP (onnx) → Embed regions
    ├── Match against vocabulary
    └── Output pxlsrc with semantic names
```

**Pros:**
- No API calls, works offline
- No usage costs
- Data stays local

**Cons:**
- Larger binary size (model weights)
- Requires ONNX runtime integration
- May need GPU for reasonable speed

### Option B: API-Based (Simpler Initial Implementation)

Call external APIs for classification:

```
pxl import image.png --semantic --api openai
    │
    ├── Extract regions (existing code)
    ├── For each region:
    │   └── POST to CLIP API or GPT-4V
    └── Output pxlsrc with semantic names
```

**Pros:**
- Simpler implementation
- Best model quality
- No local model management

**Cons:**
- Requires API key
- Per-call costs
- Network dependency

### Option C: Hybrid (Best of Both)

Use local models for common cases, API fallback for ambiguous regions:

```
pxl import image.png --semantic --fallback-api openai
    │
    ├── Local CLIP for initial classification
    ├── If confidence < threshold:
    │   └── API call for that region
    └── Output pxlsrc with semantic names
```

---

## Semantic Vocabulary

### Built-in Categories (Pixel Art Domain)

```rust
pub const SEMANTIC_VOCABULARY: &[&str] = &[
    // Character parts
    "head", "body", "arm", "leg", "hand", "foot",
    "hair", "face", "eye", "mouth", "nose",

    // Equipment
    "helmet", "armor", "shield", "sword", "axe", "bow",
    "staff", "wand", "cape", "boots", "gloves",

    // Environment
    "ground", "wall", "floor", "ceiling", "door", "window",
    "tree", "bush", "rock", "water", "grass",

    // Effects
    "outline", "shadow", "highlight", "glow", "aura",

    // UI
    "frame", "border", "button", "icon", "cursor",

    // Abstract
    "background", "foreground", "fill", "accent",
];
```

### Custom Vocabulary Support

```toml
# pxl.toml
[semantic]
vocabulary = ["potion", "gem", "coin", "key", "chest"]
vocabulary_file = "my_vocabulary.txt"
```

---

## Task Breakdown

### Task S.1: Embedding Infrastructure

**Wave:** 1 (Foundation)

Add embedding support to the codebase.

**Deliverables:**
- New crate feature: `semantic` (optional, off by default)
- New file `src/semantic/mod.rs`:
  ```rust
  pub mod embeddings;
  pub mod vocabulary;
  pub mod classifier;
  ```
- Embedding trait for pluggable backends:
  ```rust
  pub trait ImageEmbedder: Send + Sync {
      fn embed_region(&self, image: &[u8], mask: &[bool]) -> Result<Vec<f32>, Error>;
  }

  pub trait TextEmbedder: Send + Sync {
      fn embed_text(&self, text: &str) -> Result<Vec<f32>, Error>;
  }
  ```

**Verification:**
```bash
cargo build --features semantic
cargo test semantic::embeddings
```

**Dependencies:** None

---

### Task S.2: CLIP Backend (Local)

**Wave:** 2 (parallel with S.3)

Implement local CLIP inference via ONNX.

**Deliverables:**
- New file `src/semantic/backends/clip_onnx.rs`:
  ```rust
  pub struct ClipOnnx {
      session: ort::Session,
      tokenizer: Tokenizer,
  }

  impl ImageEmbedder for ClipOnnx { ... }
  impl TextEmbedder for ClipOnnx { ... }
  ```
- Model download/caching in `~/.pxl/models/`
- First-run model fetch from HuggingFace

**Verification:**
```bash
cargo test semantic::backends::clip_onnx
# Test: Image embedding produces 512-dim vector
# Test: Text embedding produces 512-dim vector
# Test: Cosine similarity works as expected
```

**Dependencies:** Task S.1

---

### Task S.3: API Backend (OpenAI/Anthropic)

**Wave:** 2 (parallel with S.2)

Implement API-based classification.

**Deliverables:**
- New file `src/semantic/backends/openai.rs`:
  ```rust
  pub struct OpenAiClassifier {
      api_key: String,
      model: String, // "gpt-4-vision-preview"
  }

  impl SemanticClassifier for OpenAiClassifier {
      fn classify_region(&self, image: &[u8], mask: &[bool]) -> Result<String, Error>;
  }
  ```
- Support for environment variable `OPENAI_API_KEY`
- Rate limiting and retry logic

**Verification:**
```bash
OPENAI_API_KEY=... cargo test semantic::backends::openai --ignored
# Test: Single region classification
# Test: Rate limiting respects API limits
```

**Dependencies:** Task S.1

---

### Task S.4: Vocabulary Matching

**Wave:** 3

Match embeddings to vocabulary labels.

**Deliverables:**
- New file `src/semantic/vocabulary.rs`:
  ```rust
  pub struct Vocabulary {
      labels: Vec<String>,
      embeddings: Vec<Vec<f32>>,  // Pre-computed
  }

  impl Vocabulary {
      pub fn from_builtin() -> Self;
      pub fn from_file(path: &Path) -> Result<Self, Error>;
      pub fn find_best_match(&self, embedding: &[f32], threshold: f32) -> Option<Match>;
  }

  pub struct Match {
      pub label: String,
      pub confidence: f32,
  }
  ```
- Built-in vocabulary embeddings (computed at build time or lazily)
- Confidence threshold filtering

**Verification:**
```bash
cargo test semantic::vocabulary
# Test: "sword" embedding matches "sword" label
# Test: Low-confidence results filtered
# Test: Custom vocabulary loads correctly
```

**Dependencies:** Task S.2 or S.3

---

### Task S.5: Import Integration

**Wave:** 4

Integrate semantic classification into import pipeline.

**Deliverables:**
- Update `src/import/mod.rs`:
  ```rust
  pub struct ImportOptions {
      // ... existing fields ...

      /// Enable semantic classification of regions
      pub semantic: bool,

      /// Semantic backend: "local", "openai", "anthropic"
      pub semantic_backend: Option<String>,

      /// Confidence threshold for semantic labels
      pub semantic_threshold: f32,
  }
  ```

- Update `ImportResult`:
  ```rust
  pub struct ImportResult {
      // ... existing fields ...

      /// Semantic labels for regions (token -> semantic label)
      pub semantic_labels: Option<HashMap<String, SemanticLabel>>,
  }

  pub struct SemanticLabel {
      pub label: String,
      pub confidence: f32,
      pub alternatives: Vec<(String, f32)>,
  }
  ```

- Token naming uses semantic labels when available:
  ```rust
  // Before: c1, c2, c3
  // After:  sword, armor, outline
  ```

**Verification:**
```bash
cargo build --features semantic
./target/release/pxl import knight.png --semantic -o knight.jsonl
# Output should have semantic token names
```

**Dependencies:** Task S.4

---

### Task S.6: CLI Flags

**Wave:** 5

Add CLI support for semantic import.

**Deliverables:**
- Update `src/cli/import.rs`:
  ```rust
  /// Import a PNG image and convert to Pixelsrc format
  Import {
      // ... existing flags ...

      /// Enable semantic classification of regions
      #[arg(long)]
      semantic: bool,

      /// Semantic backend: local, openai, anthropic
      #[arg(long, default_value = "local")]
      semantic_backend: String,

      /// Confidence threshold for semantic labels (0.0-1.0)
      #[arg(long, default_value = "0.7")]
      semantic_threshold: f32,

      /// Custom vocabulary file
      #[arg(long)]
      vocabulary: Option<PathBuf>,
  }
  ```

**Verification:**
```bash
pxl import --help
# Should show --semantic, --semantic-backend, etc.

pxl import test.png --semantic --semantic-backend local
pxl import test.png --semantic --semantic-backend openai
```

**Dependencies:** Task S.5

---

### Task S.7: SAM Integration (Optional Enhancement)

**Wave:** 6 (Optional)

Use SAM for better region segmentation.

**Deliverables:**
- New file `src/semantic/segmentation/sam.rs`:
  ```rust
  pub struct SamSegmenter {
      session: ort::Session,
  }

  impl SamSegmenter {
      pub fn segment(&self, image: &[u8]) -> Vec<Mask>;
  }
  ```
- Alternative to color-based region extraction
- Better handling of:
  - Anti-aliased edges
  - Gradients
  - Complex shapes

**Verification:**
```bash
pxl import test.png --segmentation sam --semantic
# Compare output quality vs color-based segmentation
```

**Dependencies:** Task S.5

---

### Task S.8: Embedding Cache

**Wave:** 6 (Optional)

Cache embeddings for faster repeated imports.

**Deliverables:**
- Cache vocabulary embeddings to disk
- Cache image region embeddings by hash
- Invalidation when vocabulary changes

**Verification:**
```bash
# First import: computes embeddings
pxl import large.png --semantic
# Second import: uses cache
pxl import large.png --semantic  # Should be faster
```

**Dependencies:** Task S.5

---

## Task Dependency Diagram

```
                        SEMANTIC IMPORT TASK FLOW
═══════════════════════════════════════════════════════════════════

WAVE 1 (Foundation)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────────────────────────────────┐ │
│  │   S.1 Embedding Infrastructure                             │ │
│  │   - Trait definitions                                      │ │
│  │   - Module structure                                       │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 2 (Parallel - Backends)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────┐  ┌────────────────────────────┐  │
│  │   S.2                    │  │   S.3                      │  │
│  │  CLIP Backend (Local)    │  │  API Backend               │  │
│  │  ONNX Runtime            │  │  OpenAI/Anthropic          │  │
│  └──────────────────────────┘  └────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
            │                              │
            └──────────────┬───────────────┘
                           │
                           ▼
WAVE 3 (Vocabulary)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────────────────────────────────┐ │
│  │   S.4 Vocabulary Matching                                  │ │
│  │   - Built-in labels                                        │ │
│  │   - Custom vocabulary support                              │ │
│  │   - Embedding similarity                                   │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 4 (Integration)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────────────────────────────────┐ │
│  │   S.5 Import Integration                                   │ │
│  │   - ImportOptions extension                                │ │
│  │   - Semantic token naming                                  │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 5 (CLI)
┌─────────────────────────────────────────────────────────────────┐
│  ┌────────────────────────────────────────────────────────────┐ │
│  │   S.6 CLI Flags                                            │ │
│  │   - --semantic, --semantic-backend                         │ │
│  │   - --vocabulary                                           │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
WAVE 6 (Optional Enhancements)
┌─────────────────────────────────────────────────────────────────┐
│  ┌──────────────────────────┐  ┌────────────────────────────┐  │
│  │   S.7                    │  │   S.8                      │  │
│  │  SAM Integration         │  │  Embedding Cache           │  │
│  │  (better segmentation)   │  │  (performance)             │  │
│  └──────────────────────────┘  └────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

PARALLELIZATION SUMMARY:
┌─────────────────────────────────────────────────────────────────┐
│  Wave 1: S.1                        (1 task, foundation)       │
│  Wave 2: S.2 + S.3                  (2 tasks in parallel)      │
│  Wave 3: S.4                        (1 task, needs S.2 or S.3) │
│  Wave 4: S.5                        (1 task, needs S.4)        │
│  Wave 5: S.6                        (1 task, needs S.5)        │
│  Wave 6: S.7 + S.8                  (2 tasks, optional)        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Configuration

### CLI Flags

```bash
pxl import <input.png> [options]

Options:
  --semantic                 Enable semantic classification (default: false)
  --semantic-backend <name>  Backend: local, openai, anthropic (default: local)
  --semantic-threshold <f32> Confidence threshold 0.0-1.0 (default: 0.7)
  --vocabulary <path>        Custom vocabulary file (one label per line)
  -o, --output <path>        Output file (default: <input>.pxl)
```

### `pxl.toml` Configuration

Add `[import]` and `[import.semantic]` sections to project configuration:

```toml
[project]
name = "my-game"

# Import defaults
[import]
max_colors = 256              # Maximum palette colors
analyze = true                # Enable role/relationship inference
extract_shapes = true         # Use geometric primitives (rect, polygon)

# Semantic import settings
[import.semantic]
enabled = true                # Enable by default for this project
backend = "local"             # "local", "openai", "anthropic"
threshold = 0.7               # Confidence threshold (0.0-1.0)
fallback_backend = "openai"   # Use API when local confidence < threshold

# Custom vocabulary (extends built-in)
vocabulary = [
  "potion", "gem", "coin", "key", "chest",
  "health_bar", "mana_bar", "stamina_bar"
]

# Or load from file
vocabulary_file = "assets/vocabulary.txt"

# Category-specific vocabularies
[import.semantic.categories.characters]
vocabulary = ["hero", "enemy", "npc", "boss"]

[import.semantic.categories.ui]
vocabulary = ["button", "panel", "slider", "checkbox"]
```

### Environment Variables

```bash
# API keys for remote backends
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Model cache location (default: ~/.pxl/models/)
PXL_MODEL_CACHE=/path/to/cache

# Force specific backend
PXL_SEMANTIC_BACKEND=local
```

---

## Demo Tests

Following the demo test pattern (Phase 23), semantic import gets dedicated tests:

### Test Structure

```
tests/
├── demos/
│   └── imports/
│       ├── mod.rs
│       └── semantic.rs       # DT-S* tests
└── fixtures/
    └── demos/
        └── imports/
            └── semantic/
                ├── knight.png
                ├── knight_expected.pxl
                ├── coin.png
                ├── coin_expected.pxl
                └── vocabulary.txt
```

### Demo Test Tasks

| ID | Test | Description | Fixture |
|----|------|-------------|---------|
| DT-S1 | `semantic_basic` | Basic semantic import with default vocabulary | `knight.png` |
| DT-S2 | `semantic_custom_vocab` | Import with custom vocabulary file | `coin.png` + `vocabulary.txt` |
| DT-S3 | `semantic_threshold` | Verify confidence threshold filtering | Various confidence levels |
| DT-S4 | `semantic_fallback` | Local → API fallback on low confidence | Ambiguous regions |
| DT-S5 | `semantic_toml_config` | Settings from `pxl.toml` | Project with config |

### Demo Test Implementation

```rust
// tests/demos/imports/semantic.rs

/// @demo DT-S1: Basic semantic import
/// @description Import PNG with automatic semantic token naming
/// @fixture tests/fixtures/demos/imports/semantic/knight.png
#[test]
fn semantic_basic() {
    let result = import_png_with_options(
        "tests/fixtures/demos/imports/semantic/knight.png",
        "knight",
        256,
        &ImportOptions { semantic: true, ..Default::default() }
    ).unwrap();

    // Verify semantic names were assigned
    assert!(result.palette.contains_key("helmet") ||
            result.palette.contains_key("armor") ||
            result.palette.contains_key("outline"));

    // Verify confidence metadata present
    assert!(result.analysis.unwrap().semantic_labels.is_some());
}

/// @demo DT-S2: Custom vocabulary
/// @description Import with project-specific vocabulary
#[test]
fn semantic_custom_vocab() {
    let vocab = Vocabulary::from_file("tests/fixtures/demos/imports/semantic/vocabulary.txt").unwrap();
    let result = import_with_vocabulary("coin.png", &vocab).unwrap();

    // Should use custom vocabulary labels
    assert!(result.palette.contains_key("gold_rim") ||
            result.palette.contains_key("coin_face"));
}
```

---

## mdbook Documentation

Add semantic import documentation to the book:

### New Pages

```
docs/book/src/
├── format/
│   └── semantic-import.md    # Semantic import reference
├── cli/
│   └── import.md             # Update with --semantic flags
└── ai-generation/
    └── semantic-naming.md    # Best practices for semantic vocabularies
```

### `format/semantic-import.md` Content

```markdown
# Semantic Import

Import PNG images with automatic semantic token naming using vision AI models.

## Quick Start

\`\`\`bash
pxl import knight.png --semantic -o knight.pxl
\`\`\`

<div class="pixelsrc-demo">
  <!-- WASM demo: upload PNG, see semantic import result -->
</div>

## How It Works

1. **Region extraction**: Colors are grouped into geometric regions
2. **Visual embedding**: Each region is embedded using CLIP
3. **Vocabulary matching**: Embeddings are matched against known labels
4. **Token naming**: Best-match label becomes the token name

## Configuration

### CLI Flags
[Table of flags]

### pxl.toml
[TOML configuration examples]

## Custom Vocabularies

Define project-specific labels:

\`\`\`toml
[import.semantic]
vocabulary = ["hero", "enemy", "powerup"]
\`\`\`

## Backends

| Backend | Pros | Cons |
|---------|------|------|
| `local` | Offline, free, fast | Lower accuracy |
| `openai` | High accuracy | Requires API key, costs |
| `anthropic` | High accuracy | Requires API key, costs |

## Examples

[Interactive examples with various sprite types]
```

### SUMMARY.md Addition

```markdown
- [Format](./format/README.md)
  - [Palette](./format/palette.md)
  - [Sprite](./format/sprite.md)
  - [Semantic Import](./format/semantic-import.md)  # NEW
  ...
```

---

## Test Fixtures

### New Fixtures for Semantic Import

```
tests/fixtures/
├── valid/
│   └── semantic/
│       ├── basic_semantic.pxl      # Sprite with semantic tokens
│       └── custom_vocab.pxl        # Uses non-standard vocabulary
├── demos/
│   └── imports/
│       └── semantic/
│           ├── knight.png          # 16x16 character
│           ├── knight_expected.pxl # Expected semantic output
│           ├── coin.png            # Simple object
│           ├── coin_expected.pxl
│           ├── tileset.png         # Environment tiles
│           ├── tileset_expected.pxl
│           └── vocabulary.txt      # Custom vocabulary for tests
└── lenient/
    └── semantic/
        └── low_confidence.pxl      # Regions below threshold → generic names
```

### Fixture: `knight_expected.pxl`

```json5
{
  type: "palette",
  name: "knight_palette",
  colors: {
    _: "transparent",
    outline: "#000000",
    helmet: "#808080",
    armor: "#C0C0C0",
    sword: "#FFD700",
    boots: "#8B4513"
  },
  roles: {
    outline: "boundary",
    helmet: "fill",
    armor: "fill",
    sword: "fill",
    boots: "fill"
  }
}
{
  type: "sprite",
  name: "knight",
  size: [16, 16],
  palette: "knight_palette",
  regions: {
    _: "background",
    outline: { "auto-outline": "armor" },
    helmet: { points: [[4,2], [5,2], [4,3], [5,3], [6,3]] },
    armor: { union: [{ rect: [5, 4, 6, 8] }] },
    sword: { points: [[12,6], [12,7], [12,8], [13,7]] },
    boots: { rect: [6, 12, 4, 4] }
  }
}
```

---

## Example Output

### Input: knight.png (16x16 pixel art character)

### Current Import Output:
```json5
{
  type: "palette",
  name: "knight_palette",
  colors: {
    _: "#00000000",
    c1: "#808080",
    c2: "#C0C0C0",
    c3: "#FFD700",
    c4: "#8B4513",
    c5: "#000000"
  }
}
{
  type: "sprite",
  name: "knight",
  size: [16, 16],
  palette: "knight_palette",
  regions: {
    _: "background",
    c1: { points: [[4,2], [5,2], [4,3], [5,3], [6,3]] },
    c2: { union: [{ rect: [5, 4, 6, 8] }, { points: [[4,5], [11,5]] }] },
    c3: { points: [[12,6], [12,7], [12,8], [13,7]] },
    c4: { rect: [6, 12, 4, 4] },
    c5: { "auto-outline": "c2", thickness: 1 }
  }
}
```

### Semantic Import Output:
```json5
{
  type: "palette",
  name: "knight_palette",
  colors: {
    _: "#00000000",
    helmet: "#808080",
    armor: "#C0C0C0",
    sword: "#FFD700",
    boots: "#8B4513",
    outline: "#000000"
  },
  roles: {
    outline: "boundary",
    helmet: "fill",
    armor: "fill",
    sword: "fill",
    boots: "fill"
  },
  semantic: {
    confidence: { helmet: 0.89, armor: 0.92, sword: 0.95, boots: 0.78, outline: 0.99 }
  }
}
{
  type: "sprite",
  name: "knight",
  size: [16, 16],
  palette: "knight_palette",
  regions: {
    _: "background",
    helmet: { points: [[4,2], [5,2], [4,3], [5,3], [6,3]] },
    armor: { union: [{ rect: [5, 4, 6, 8] }, { points: [[4,5], [11,5]] }] },
    sword: { points: [[12,6], [12,7], [12,8], [13,7]] },
    boots: { rect: [6, 12, 4, 4] },
    outline: { "auto-outline": "armor", thickness: 1 }
  }
}
```

---

## Open Questions

1. **Model size vs quality tradeoff:** Smaller CLIP models (ViT-B/32) vs larger (ViT-L/14)?
2. **Pixel art specificity:** Pre-trained models are trained on photos. Fine-tune on pixel art dataset?
3. **Hierarchical semantics:** Should we detect "character" containing "helmet" + "armor" + "sword"?
4. **Disambiguation:** When confidence is similar for multiple labels, what strategy?
5. **User feedback loop:** Allow users to correct labels and learn from corrections?

---

## Success Criteria

### Functional
1. `pxl import knight.png --semantic` produces meaningful token names
2. 80%+ of regions get semantically appropriate names on standard pixel art
3. Local inference works without API keys (acceptable quality)
4. API fallback achieves 95%+ accuracy
5. Custom vocabulary extends/overrides built-in labels

### Performance
6. < 5 seconds for 64x64 sprite on modern hardware (local backend)
7. Embedding cache reduces repeated imports to < 1 second

### Configuration
8. All options accessible via CLI flags
9. All options configurable via `pxl.toml`
10. Environment variables for API keys and cache paths

### Documentation
11. mdbook page at `format/semantic-import.md`
12. CLI help updated with all semantic flags
13. Interactive WASM demo in documentation

### Testing
14. Demo tests DT-S1 through DT-S5 pass
15. Test fixtures cover basic, custom vocab, threshold, fallback scenarios
16. CI runs semantic tests (with mocked API for external backends)

---

## Verification Summary

```bash
# 1. Build with semantic feature
cargo build --features semantic

# 2. Run demo tests
cargo test demos::imports::semantic

# 3. CLI integration
pxl import --help | grep semantic
pxl import tests/fixtures/demos/imports/semantic/knight.png --semantic

# 4. TOML configuration
cat > test_project/pxl.toml << 'EOF'
[project]
name = "test"

[import.semantic]
enabled = true
vocabulary = ["custom_label"]
EOF
cd test_project && pxl import ../knight.png

# 5. Verify mdbook builds
cd docs/book && mdbook build

# 6. All previous tests still pass
cargo test
```

---

## References

- [Segment Anything (SAM)](https://github.com/facebookresearch/segment-anything)
- [CLIP](https://github.com/openai/CLIP)
- [ONNX Runtime](https://onnxruntime.ai/)
- [Candle (Rust ML)](https://github.com/huggingface/candle)
