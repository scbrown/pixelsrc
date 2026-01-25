//! Completion item generation for structured format elements.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

/// Helper to create completion items
pub fn make_completion(
    label: &str,
    detail: &str,
    kind: CompletionItemKind,
    insert: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        kind: Some(kind),
        insert_text: Some(insert.to_string()),
        ..Default::default()
    }
}

/// Get shape primitive completions for the structured format
pub fn get_shape_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "points",
            "Individual pixels: [[x, y], ...]",
            CompletionItemKind::PROPERTY,
            "points: [[0, 0]]",
        ),
        make_completion(
            "line",
            "Bresenham line: [[x1, y1], [x2, y2]]",
            CompletionItemKind::PROPERTY,
            "line: [[0, 0], [8, 8]]",
        ),
        make_completion(
            "rect",
            "Filled rectangle: [x, y, w, h]",
            CompletionItemKind::PROPERTY,
            "rect: [0, 0, 8, 8]",
        ),
        make_completion(
            "stroke",
            "Rectangle outline: [x, y, w, h]",
            CompletionItemKind::PROPERTY,
            "stroke: [0, 0, 8, 8]",
        ),
        make_completion(
            "ellipse",
            "Filled ellipse: [cx, cy, rx, ry]",
            CompletionItemKind::PROPERTY,
            "ellipse: [4, 4, 3, 2]",
        ),
        make_completion(
            "circle",
            "Filled circle: [cx, cy, r]",
            CompletionItemKind::PROPERTY,
            "circle: [4, 4, 3]",
        ),
        make_completion(
            "polygon",
            "Filled polygon: [[x, y], ...]",
            CompletionItemKind::PROPERTY,
            "polygon: [[0, 0], [8, 0], [4, 8]]",
        ),
        make_completion(
            "path",
            "SVG-lite path (M, L, H, V, Z)",
            CompletionItemKind::PROPERTY,
            "path: \"M0,0 L8,8\"",
        ),
        make_completion(
            "fill",
            "Flood fill: \"inside(token)\"",
            CompletionItemKind::PROPERTY,
            "fill: \"inside(outline)\"",
        ),
    ]
}

/// Get compound operation completions
pub fn get_compound_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "union",
            "Combine multiple shapes",
            CompletionItemKind::PROPERTY,
            "union: [{ rect: [0, 0, 4, 4] }]",
        ),
        make_completion(
            "base",
            "Base shape for subtraction",
            CompletionItemKind::PROPERTY,
            "base: { rect: [0, 0, 8, 8] }",
        ),
        make_completion(
            "subtract",
            "Remove shapes from base",
            CompletionItemKind::PROPERTY,
            "subtract: [{ circle: [4, 4, 2] }]",
        ),
        make_completion(
            "intersect",
            "Keep overlapping area",
            CompletionItemKind::PROPERTY,
            "intersect: [{ rect: [0, 0, 4, 4] }]",
        ),
        make_completion(
            "except",
            "Subtract token pixels",
            CompletionItemKind::PROPERTY,
            "except: [\"eye\", \"mouth\"]",
        ),
    ]
}

/// Get modifier completions for region definitions
pub fn get_modifier_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "symmetric",
            "Mirror across axis: \"x\", \"y\", \"xy\"",
            CompletionItemKind::PROPERTY,
            "symmetric: \"x\"",
        ),
        make_completion("z", "Render order (higher = on top)", CompletionItemKind::PROPERTY, "z: 10"),
        make_completion(
            "round",
            "Corner radius for rect/stroke",
            CompletionItemKind::PROPERTY,
            "round: 2",
        ),
        make_completion("thickness", "Line thickness", CompletionItemKind::PROPERTY, "thickness: 1"),
        make_completion(
            "within",
            "Validate containment",
            CompletionItemKind::PROPERTY,
            "within: \"eye\"",
        ),
        make_completion(
            "adjacent-to",
            "Validate adjacency",
            CompletionItemKind::PROPERTY,
            "\"adjacent-to\": \"skin\"",
        ),
        make_completion("x", "Column range: [min, max]", CompletionItemKind::PROPERTY, "x: [0, 8]"),
        make_completion("y", "Row range: [min, max]", CompletionItemKind::PROPERTY, "y: [0, 8]"),
        make_completion(
            "auto-outline",
            "Generate outline",
            CompletionItemKind::PROPERTY,
            "\"auto-outline\": \"body\"",
        ),
        make_completion(
            "auto-shadow",
            "Generate shadow",
            CompletionItemKind::PROPERTY,
            "\"auto-shadow\": \"body\"",
        ),
        make_completion(
            "offset",
            "Shadow offset: [x, y]",
            CompletionItemKind::PROPERTY,
            "offset: [1, 1]",
        ),
        make_completion(
            "repeat",
            "Tile shape: [count_x, count_y]",
            CompletionItemKind::PROPERTY,
            "repeat: [4, 4]",
        ),
        make_completion(
            "spacing",
            "Tile spacing: [x, y]",
            CompletionItemKind::PROPERTY,
            "spacing: [1, 1]",
        ),
        make_completion(
            "offset-alternate",
            "Offset alternating rows",
            CompletionItemKind::PROPERTY,
            "\"offset-alternate\": true",
        ),
        make_completion(
            "transform",
            "Geometric transform",
            CompletionItemKind::PROPERTY,
            "transform: \"rotate(45deg)\"",
        ),
        make_completion(
            "jitter",
            "Controlled randomness",
            CompletionItemKind::PROPERTY,
            "jitter: { y: [-1, 1] }",
        ),
        make_completion("seed", "Random seed for jitter", CompletionItemKind::PROPERTY, "seed: 42"),
        make_completion("role", "Semantic role", CompletionItemKind::PROPERTY, "role: \"fill\""),
    ]
}

/// Get role value completions
pub fn get_role_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "boundary",
            "Edge-defining (outlines) - high priority",
            CompletionItemKind::ENUM_MEMBER,
            "\"boundary\"",
        ),
        make_completion(
            "anchor",
            "Critical details (eyes) - must survive transforms",
            CompletionItemKind::ENUM_MEMBER,
            "\"anchor\"",
        ),
        make_completion(
            "fill",
            "Interior mass (skin, clothes) - can shrink",
            CompletionItemKind::ENUM_MEMBER,
            "\"fill\"",
        ),
        make_completion(
            "shadow",
            "Depth indicators - derives from parent",
            CompletionItemKind::ENUM_MEMBER,
            "\"shadow\"",
        ),
        make_completion(
            "highlight",
            "Light indicators - derives from parent",
            CompletionItemKind::ENUM_MEMBER,
            "\"highlight\"",
        ),
    ]
}

/// Get relationship type completions
pub fn get_relationship_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "derives-from",
            "Color derived from another token",
            CompletionItemKind::ENUM_MEMBER,
            "{ \"type\": \"derives-from\", \"target\": \"skin\" }",
        ),
        make_completion(
            "contained-within",
            "Spatially inside another region",
            CompletionItemKind::ENUM_MEMBER,
            "{ \"type\": \"contained-within\", \"target\": \"eye\" }",
        ),
        make_completion(
            "adjacent-to",
            "Must touch specified region",
            CompletionItemKind::ENUM_MEMBER,
            "{ \"type\": \"adjacent-to\", \"target\": \"outline\" }",
        ),
        make_completion(
            "paired-with",
            "Symmetric relationship",
            CompletionItemKind::ENUM_MEMBER,
            "{ \"type\": \"paired-with\", \"target\": \"right-eye\" }",
        ),
    ]
}

/// Get state rule apply property completions
pub fn get_state_apply_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "color",
            "Override region color",
            CompletionItemKind::PROPERTY,
            "\"color\": \"#FF0000\"",
        ),
        make_completion(
            "visible",
            "Override visibility",
            CompletionItemKind::PROPERTY,
            "\"visible\": false",
        ),
        make_completion("z", "Override z-index", CompletionItemKind::PROPERTY, "\"z\": 100"),
        make_completion(
            "transform",
            "Apply transform",
            CompletionItemKind::PROPERTY,
            "\"transform\": \"scale(1.1)\"",
        ),
    ]
}

/// Get state selector completions
pub fn get_state_selector_completions() -> Vec<CompletionItem> {
    vec![
        make_completion(
            "[token=name]",
            "Select by token name",
            CompletionItemKind::SNIPPET,
            "[token=outline]",
        ),
        make_completion("[role=type]", "Select by role", CompletionItemKind::SNIPPET, "[role=boundary]"),
        make_completion(".state", "Select by sprite state", CompletionItemKind::SNIPPET, ".hover"),
        make_completion(
            ".hover [role=fill]",
            "Example: hover state fill regions",
            CompletionItemKind::SNIPPET,
            ".hover [role=fill]",
        ),
        make_completion(
            ".pressed [token=background]",
            "Example: pressed state background",
            CompletionItemKind::SNIPPET,
            ".pressed [token=background]",
        ),
    ]
}
