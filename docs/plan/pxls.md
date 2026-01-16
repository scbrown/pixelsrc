# Proposal: Pixelsrc Stream (PXS)

**Status:** Proposal
**Related:** Phase 16 (.pxl Format)

## The Problem with JSON

While the `.pxl` multi-line JSON format (Phase 16) improves readability, it still suffers from the "JSON Tax":
1.  **Fragility**: A single missing closing brace invalidates the entire object.
2.  **Verbosity**: Repeated keys (`"type": "sprite"`, `"grid": [...]`) consume context window.
3.  **Late Emission**: Standard JSON parsers often wait for a complete array before emitting data, hurting streaming responsiveness.

## The Solution: Pixelsrc Stream (PXS)

PXS is a custom DSL designed for **streaming robustness** and **token efficiency**. It treats sprite definition as a stream of plotting instructions rather than data serialization.

### Grammar

- **Structure**: `keyword [attributes] { body }`
- **Row Separator**: `/` (allows visual alignment without relying on newlines)
- **Whitespace**: Ignored (spaces, tabs, newlines have no semantic meaning)
- **Tokens**: `{name}` (same as current format)

### Example

```pxs
palette[coin] {
  {_}: #0000;
  {gold}: #FFD700;
  {shine}: #FFFACD;
  {shadow}: #B8860B
}

sprite[coin, size:8x8, palette:coin] {
  {_}{_}{gold}{gold}{gold}{gold}{_}{_} /
  {_}{gold}{shine}{shine}{gold}{gold}{gold}{_} /
  {gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold} /
  {gold}{shine}{gold}{gold}{gold}{gold}{shadow}{gold} /
  {gold}{gold}{gold}{gold}{gold}{gold}{shadow}{gold} /
  {gold}{gold}{gold}{gold}{gold}{shadow}{shadow}{gold} /
  {_}{gold}{gold}{gold}{gold}{gold}{gold}{_} /
  {_}{_}{shadow}{shadow}{shadow}{shadow}{_}{_}
}
```

## Benefits

### 1. Streaming Recoverability
If the stream cuts off at line 5, a parser can still render the first 4 rows perfectly. There is no waiting for a closing `]` to validate the grid structure.

### 2. Token Efficiency
**JSON:** `{"type":"sprite","name":"dot","grid":["{x}"]}` (51 chars)
**PXS:** `sprite[dot]{{x}}` (16 chars)

PXS is ~70% more concise for headers and structure, allowing larger sprites in the same context window.

### 3. Spatial Correspondence
The `/` separator forces a row break logic that aligns with the visual grid, but because whitespace is ignored, the AI can indent arbitrarily for its own "mental map" without breaking the parser.

### 4. Advanced Features (Potential)
**Run-Length Encoding (RLE):**
Native support for repetition multipliers:
`{grass}*10` vs `"{grass}{grass}{grass}..."`

## Implementation Strategy

This would be a parser alternative in `src/parser.rs`. The renderer remains unchanged; only the input deserialization logic changes.

1.  **Lexer**: Tokenize input into `Keyword`, `Attribute`, `BlockStart`, `Token`, `RowBreak`, `BlockEnd`.
2.  **Parser**: Convert stream of tokens into existing `Sprite` and `Palette` structs.
3.  **CLI**: Auto-detect format based on first non-whitespace character (e.g., `{` vs `sprite`).
