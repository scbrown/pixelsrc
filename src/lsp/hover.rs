//! Hover information for structured format elements.

/// Get hover information for a role
pub fn get_role_hover(role: &str) -> Option<String> {
    match role {
        "boundary" => Some(
            "**Role: Boundary**\n\nEdge-defining regions (outlines, borders).\n\n**Transform Behavior:** High priority, preserve connectivity.".to_string(),
        ),
        "anchor" => Some(
            "**Role: Anchor**\n\nCritical details like eyes that identify the sprite.\n\n**Transform Behavior:** Must survive transforms (minimum 1px).".to_string(),
        ),
        "fill" => Some(
            "**Role: Fill**\n\nInterior mass regions (skin, clothes).\n\n**Transform Behavior:** Can shrink, low priority.".to_string(),
        ),
        "shadow" => Some(
            "**Role: Shadow**\n\nDepth indicators, typically darker variants.\n\n**Transform Behavior:** Derives position from parent.".to_string(),
        ),
        "highlight" => Some(
            "**Role: Highlight**\n\nLight indicators, typically lighter variants.\n\n**Transform Behavior:** Derives position from parent.".to_string(),
        ),
        _ => None,
    }
}

/// Get hover information for a shape type
pub fn get_shape_hover(shape: &str) -> Option<String> {
    match shape {
        "points" => Some(
            "**Shape: Points**\n\nIndividual pixels at specific coordinates.\n\n```json5\npoints: [[2, 3], [5, 3]]\n```".to_string(),
        ),
        "line" => Some(
            "**Shape: Line**\n\nBresenham line between points.\n\n```json5\nline: [[0, 0], [8, 8]]\n```\n\nOptional: `thickness` (default: 1)".to_string(),
        ),
        "rect" => Some(
            "**Shape: Rectangle**\n\nFilled rectangle.\n\n```json5\nrect: [x, y, width, height]\n```\n\nOptional: `round` (corner radius)".to_string(),
        ),
        "stroke" => Some(
            "**Shape: Stroke**\n\nRectangle outline (unfilled).\n\n```json5\nstroke: [x, y, width, height]\n```\n\nOptional: `thickness`, `round`".to_string(),
        ),
        "ellipse" => Some(
            "**Shape: Ellipse**\n\nFilled ellipse.\n\n```json5\nellipse: [cx, cy, rx, ry]\n```".to_string(),
        ),
        "circle" => Some(
            "**Shape: Circle**\n\nShorthand for equal-radius ellipse.\n\n```json5\ncircle: [cx, cy, r]\n```".to_string(),
        ),
        "polygon" => Some(
            "**Shape: Polygon**\n\nFilled polygon from vertices.\n\n```json5\npolygon: [[0, 0], [8, 0], [4, 8]]\n```".to_string(),
        ),
        "path" => Some(
            "**Shape: Path**\n\nSVG-lite path syntax.\n\n**Supported commands:** M (move), L (line), H (horizontal), V (vertical), Z (close)\n\n```json5\npath: \"M2,0 L6,0 L8,2 Z\"\n```".to_string(),
        ),
        "fill" => Some(
            "**Shape: Fill**\n\nFlood fill inside a boundary.\n\n```json5\nfill: \"inside(outline)\"\n```\n\nOptional: `seed: [x, y]` (starting point)".to_string(),
        ),
        _ => None,
    }
}

/// Get hover information for a modifier
pub fn get_modifier_hover(modifier: &str) -> Option<String> {
    match modifier {
        "symmetric" => Some(
            "**Modifier: Symmetric**\n\nAuto-mirror across axis.\n\n**Values:**\n- `\"x\"` - Horizontal mirror\n- `\"y\"` - Vertical mirror\n- `\"xy\"` - Both axes (4-way)\n- `8` - Mirror around specific x-coordinate".to_string(),
        ),
        "z" => Some(
            "**Modifier: Z-Order**\n\nExplicit render order. Higher values render on top.\n\nDefault: definition order.".to_string(),
        ),
        "within" => Some(
            "**Constraint: Within**\n\nValidates that this region stays inside another.\n\nChecked after all regions are resolved.".to_string(),
        ),
        "adjacent-to" => Some(
            "**Constraint: Adjacent-To**\n\nValidates that this region touches another.\n\nChecked after all regions are resolved.".to_string(),
        ),
        "repeat" => Some(
            "**Transform: Repeat**\n\nTile a shape.\n\n```json5\nrepeat: [count_x, count_y],\nspacing: [gap_x, gap_y],\n\"offset-alternate\": true\n```".to_string(),
        ),
        "jitter" => Some(
            "**Transform: Jitter**\n\nControlled randomness.\n\n```json5\njitter: { x: [-1, 1], y: [-2, 0] },\nseed: 42\n```".to_string(),
        ),
        _ => None,
    }
}

/// Get hover information for structured format elements at cursor position
///
/// Checks for roles, shapes, and modifiers in the line and provides hover info.
pub fn get_structured_format_hover(line: &str, char_pos: u32) -> Option<String> {
    let pos = char_pos as usize;

    // List of keywords to check for hover
    let shape_keywords =
        ["points", "line", "rect", "stroke", "ellipse", "circle", "polygon", "path", "fill"];
    let modifier_keywords =
        ["symmetric", "z", "within", "adjacent-to", "repeat", "jitter", "round", "thickness"];
    let role_keywords = ["boundary", "anchor", "fill", "shadow", "highlight"];

    // Find the word at cursor position
    let line_chars: Vec<char> = line.chars().collect();

    // Find word boundaries
    let mut start = pos;
    while start > 0
        && (line_chars
            .get(start - 1)
            .map(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .unwrap_or(false))
    {
        start -= 1;
    }

    let mut end = pos;
    while end < line_chars.len()
        && (line_chars
            .get(end)
            .map(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .unwrap_or(false))
    {
        end += 1;
    }

    if start >= end {
        return None;
    }

    let word: String = line_chars[start..end].iter().collect();
    let word_lower = word.to_lowercase();

    // Check if it's a role value (either as key in roles object or as value)
    for role in &role_keywords {
        if word_lower == *role {
            return get_role_hover(role);
        }
    }

    // Check if it's a shape keyword
    for shape in &shape_keywords {
        if word_lower == *shape {
            return get_shape_hover(shape);
        }
    }

    // Check if it's a modifier keyword
    for modifier in &modifier_keywords {
        if word_lower == *modifier {
            return get_modifier_hover(modifier);
        }
    }

    // Check for relationship types in the line
    if line.contains("derives-from")
        && (word == "derives" || word == "from" || word_lower == "derives-from")
    {
        return Some(
            "**Relationship: Derives-From**\n\nColor is derived from another token.\n\nUsed for shadow/highlight variants.".to_string(),
        );
    }
    if line.contains("contained-within")
        && (word == "contained" || word == "within" || word_lower == "contained-within")
    {
        return Some(
            "**Relationship: Contained-Within**\n\nSpatially inside another region.\n\nUsed for constraint validation.".to_string(),
        );
    }
    if line.contains("paired-with")
        && (word == "paired" || word == "with" || word_lower == "paired-with")
    {
        return Some(
            "**Relationship: Paired-With**\n\nSymmetric relationship between two regions.\n\nUsed for left/right eye, wing pairs, etc.".to_string(),
        );
    }

    None
}
