//! Alias CLI Demos
//!
//! Demo tests for the `pxl alias` command that extracts common tokens
//! into single-letter aliases for compact representation.

use pixelsrc::alias::{expand_aliases, extract_aliases};
use pixelsrc::models::TtpObject;
use pixelsrc::parser::parse_stream;
use std::collections::HashMap;
use std::io::Cursor;

/// @demo cli/alias#basic
/// @title Basic Alias Extraction
/// @description Extracts common tokens into single-letter aliases based on frequency.
#[test]
fn test_alias_basic() {
    let input = include_str!("../../../examples/demos/cli/format/alias_sprite.jsonl");

    // Parse to get sprite
    let reader = Cursor::new(input);
    let result = parse_stream(reader);

    let sprite = result
        .objects
        .iter()
        .find_map(|o| match o {
            TtpObject::Sprite(s) if s.name == "robot_head" => Some(s),
            _ => None,
        })
        .expect("Sprite 'robot_head' should exist");

    // Extract aliases
    let (aliases, transformed) = extract_aliases(&sprite.grid);

    // Should have aliases for each unique token
    assert!(!aliases.is_empty(), "Should extract aliases");

    // Transformed grid should use short aliases
    assert_eq!(
        transformed.len(),
        sprite.grid.len(),
        "Transformed grid should have same row count"
    );

    // Each transformed token should be 3 chars: {X}
    for row in &transformed {
        // Tokens in transformed grid should be short like {a}, {b}, {_}
        let tokens: Vec<&str> = row
            .split('{')
            .filter(|s| !s.is_empty())
            .collect();
        for token in tokens {
            // Token format is "X}" where X is single char
            assert!(
                token.len() <= 2,
                "Aliased tokens should be short (found: {{{})",
                token
            );
        }
    }
}

/// @demo cli/alias#underscore_preserved
/// @title Underscore Token Preserved
/// @description The special {_} token always maps to underscore, not a letter.
#[test]
fn test_alias_underscore_preserved() {
    let grid = vec![
        "{_}{x}{y}".to_string(),
        "{x}{_}{x}".to_string(),
        "{y}{x}{_}".to_string(),
    ];

    let (aliases, transformed) = extract_aliases(&grid);

    // Underscore should always be '_'
    assert_eq!(
        aliases.get(&'_'),
        Some(&"_".to_string()),
        "{{_}} should always map to '_'"
    );

    // Transformed grid should still have {_} tokens
    assert!(
        transformed.iter().any(|row| row.contains("{_}")),
        "Transformed grid should preserve {{_}} tokens"
    );
}

/// @demo cli/alias#frequency_order
/// @title Frequency-Based Assignment
/// @description Most frequent tokens get earlier letters (a, b, c...).
#[test]
fn test_alias_frequency_order() {
    // Token {x} appears most (9 times), then {y} (6 times), then {z} (3 times)
    let grid = vec![
        "{x}{x}{x}{y}{y}{z}".to_string(),
        "{x}{x}{x}{y}{y}{z}".to_string(),
        "{x}{x}{x}{y}{y}{z}".to_string(),
    ];

    let (aliases, _transformed) = extract_aliases(&grid);

    // Most frequent should get 'a'
    assert_eq!(
        aliases.get(&'a'),
        Some(&"x".to_string()),
        "Most frequent token should get 'a'"
    );

    // Second most frequent should get 'b'
    assert_eq!(
        aliases.get(&'b'),
        Some(&"y".to_string()),
        "Second most frequent should get 'b'"
    );

    // Third most frequent should get 'c'
    assert_eq!(
        aliases.get(&'c'),
        Some(&"z".to_string()),
        "Third most frequent should get 'c'"
    );
}

/// @demo cli/alias#roundtrip
/// @title Alias Roundtrip
/// @description Expanding aliases produces the original grid.
#[test]
fn test_alias_roundtrip() {
    let original = vec![
        "{transparent}{skin}{skin}{transparent}".to_string(),
        "{transparent}{hair}{hair}{transparent}".to_string(),
        "{skin}{eye}{eye}{skin}".to_string(),
        "{skin}{skin}{skin}{skin}".to_string(),
    ];

    // Extract aliases
    let (aliases, transformed) = extract_aliases(&original);

    // Expand back
    let expanded = expand_aliases(&transformed, &aliases);

    // Should match original
    assert_eq!(original, expanded, "Roundtrip should preserve original grid");
}

/// @demo cli/alias#expand
/// @title Alias Expansion
/// @description Short aliases can be expanded back to full token names.
#[test]
fn test_alias_expand() {
    let aliases: HashMap<char, String> = [
        ('a', "skin".to_string()),
        ('b', "hair".to_string()),
        ('c', "eye".to_string()),
        ('_', "_".to_string()),
    ]
    .into_iter()
    .collect();

    let grid = vec![
        "{_}{b}{b}{_}".to_string(),
        "{a}{c}{c}{a}".to_string(),
    ];

    let expanded = expand_aliases(&grid, &aliases);

    assert_eq!(expanded[0], "{_}{hair}{hair}{_}");
    assert_eq!(expanded[1], "{skin}{eye}{eye}{skin}");
}

/// @demo cli/alias#json_output
/// @title JSON Alias Map Output
/// @description Alias command outputs JSON mapping of aliases to full names.
#[test]
fn test_alias_json_output() {
    let grid = vec![
        "{_}{red}{green}".to_string(),
        "{blue}{red}{_}".to_string(),
    ];

    let (aliases, transformed) = extract_aliases(&grid);

    // Build JSON-like output (simulating CLI output)
    let mut alias_map: HashMap<String, String> = HashMap::new();
    for (alias_char, full_name) in &aliases {
        let key = format!("{{{}}}", alias_char);
        let value = format!("{{{}}}", full_name);
        alias_map.insert(key, value);
    }

    // Verify the mapping is complete
    assert!(alias_map.contains_key("{_}"), "Should have {{_}} in map");

    // Transformed grid should use the short aliases
    assert_eq!(transformed.len(), 2, "Should have 2 rows");
}

/// @demo cli/alias#verbose_tokens
/// @title Verbose Token Compression
/// @description Long descriptive token names are compressed to single letters.
#[test]
fn test_alias_verbose_tokens() {
    let input = include_str!("../../../examples/demos/cli/format/alias_sprite.jsonl");

    // Parse to get sprite
    let reader = Cursor::new(input);
    let result = parse_stream(reader);

    let sprite = result
        .objects
        .iter()
        .find_map(|o| match o {
            TtpObject::Sprite(s) if s.name == "robot_head" => Some(s),
            _ => None,
        })
        .expect("Sprite 'robot_head' should exist");

    // Original has verbose names like {metal_light}, {metal_dark}, etc.
    let original_len: usize = sprite.grid.iter().map(|r| r.len()).sum();

    // Extract aliases
    let (_aliases, transformed) = extract_aliases(&sprite.grid);
    let transformed_len: usize = transformed.iter().map(|r| r.len()).sum();

    // Aliased version should be significantly shorter
    assert!(
        transformed_len < original_len,
        "Aliased grid should be shorter ({} < {})",
        transformed_len,
        original_len
    );
}
