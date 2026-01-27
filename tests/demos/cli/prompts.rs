//! Prompts CLI Demos
//!
//! Demo tests for the `pxl prompts` command that shows LLM prompt templates.

/// @demo cli/info#prompts
/// @title Prompts Command
/// @description The pxl prompts command shows LLM prompt templates.
#[test]
fn test_prompts_list() {
    // These are the available prompt templates
    let expected_templates = vec!["character", "item", "tileset", "animation"];

    // Get the project root directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Each template should be available
    for template in &expected_templates {
        // Templates are loaded from docs/prompts/templates/{name}.txt
        let path = format!("{}/docs/prompts/templates/{}.txt", manifest_dir, template);
        assert!(
            std::path::Path::new(&path).exists(),
            "Template '{}' should exist at {}",
            template,
            path
        );
    }
}

/// @demo cli/prompts#character
/// @title Character Template
/// @description Prompt template for generating character sprites.
#[test]
fn test_prompts_character() {
    let content = include_str!("../../../docs/prompts/templates/character.txt");

    // Character template should have key sections
    assert!(!content.is_empty(), "Character template should not be empty");
    assert!(content.len() > 100, "Character template should have substantial content");
}

/// @demo cli/prompts#item
/// @title Item Template
/// @description Prompt template for generating item sprites.
#[test]
fn test_prompts_item() {
    let content = include_str!("../../../docs/prompts/templates/item.txt");

    // Item template should have key sections
    assert!(!content.is_empty(), "Item template should not be empty");
    assert!(content.len() > 100, "Item template should have substantial content");
}

/// @demo cli/prompts#tileset
/// @title Tileset Template
/// @description Prompt template for generating tileset sprites.
#[test]
fn test_prompts_tileset() {
    let content = include_str!("../../../docs/prompts/templates/tileset.txt");

    // Tileset template should have key sections
    assert!(!content.is_empty(), "Tileset template should not be empty");
    assert!(content.len() > 100, "Tileset template should have substantial content");
}

/// @demo cli/prompts#animation
/// @title Animation Template
/// @description Prompt template for generating animation sequences.
#[test]
fn test_prompts_animation() {
    let content = include_str!("../../../docs/prompts/templates/animation.txt");

    // Animation template should have key sections
    assert!(!content.is_empty(), "Animation template should not be empty");
    assert!(content.len() > 100, "Animation template should have substantial content");
}
