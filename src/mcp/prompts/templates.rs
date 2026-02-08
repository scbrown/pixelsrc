//! Prompt template definitions for the pixelsrc MCP server.
//!
//! Each prompt returns a sequence of messages that guide an AI model through
//! a specific pixelsrc workflow: creating sprites, animations, reviewing code,
//! or getting pixel art style guidance.

use rmcp::model::*;

use crate::prime::PRIMER_BRIEF;

/// Returns the list of all available MCP prompts.
pub fn list_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "create_sprite".into(),
            title: Some("Create Pixel Art Sprite".into()),
            description: Some(
                "Guided sprite creation workflow — generates .pxl source \
                 for a character, item, or object sprite."
                    .into(),
            ),
            arguments: Some(vec![
                PromptArgument {
                    name: "description".into(),
                    title: Some("Sprite description".into()),
                    description: Some("What the sprite should look like".into()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "size".into(),
                    title: Some("Dimensions".into()),
                    description: Some(
                        "Sprite dimensions, e.g. \"16x16\" or \"32x32\" (default: 16x16)".into(),
                    ),
                    required: Some(false),
                },
                PromptArgument {
                    name: "palette".into(),
                    title: Some("Palette".into()),
                    description: Some(
                        "Built-in palette name (gameboy, pico8, nes, etc.) or \"custom\"".into(),
                    ),
                    required: Some(false),
                },
                PromptArgument {
                    name: "style".into(),
                    title: Some("Art style".into()),
                    description: Some(
                        "Art style hint: retro, modern, minimal, cute, detailed".into(),
                    ),
                    required: Some(false),
                },
            ]),
            icons: None,
            meta: None,
        },
        Prompt {
            name: "create_animation".into(),
            title: Some("Create Pixel Art Animation".into()),
            description: Some(
                "Guided animation creation workflow — generates .pxl source \
                 with multiple frames and an animation object."
                    .into(),
            ),
            arguments: Some(vec![
                PromptArgument {
                    name: "description".into(),
                    title: Some("Animation description".into()),
                    description: Some("What the animation should show".into()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "frames".into(),
                    title: Some("Frame count".into()),
                    description: Some("Number of animation frames (default: 4)".into()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "fps".into(),
                    title: Some("Frame rate".into()),
                    description: Some("Frames per second (default: 8)".into()),
                    required: Some(false),
                },
            ]),
            icons: None,
            meta: None,
        },
        Prompt {
            name: "review_pxl".into(),
            title: Some("Review .pxl Source".into()),
            description: Some(
                "Review existing .pxl source for correctness, style, and improvements.".into(),
            ),
            arguments: Some(vec![PromptArgument {
                name: "source".into(),
                title: Some(".pxl source".into()),
                description: Some("The .pxl JSONL content to review".into()),
                required: Some(true),
            }]),
            icons: None,
            meta: None,
        },
        Prompt {
            name: "pixel_art_guide".into(),
            title: Some("Pixel Art Style Guide".into()),
            description: Some(
                "Palette recommendations and art style guidelines for a game genre.".into(),
            ),
            arguments: Some(vec![PromptArgument {
                name: "genre".into(),
                title: Some("Game genre".into()),
                description: Some(
                    "Game genre: RPG, platformer, puzzle, horror, sci-fi, etc.".into(),
                ),
                required: Some(true),
            }]),
            icons: None,
            meta: None,
        },
    ]
}

/// Retrieves a prompt by name, substituting the provided arguments.
///
/// Returns `None` if the prompt name is unknown.
pub fn get_prompt(
    name: &str,
    arguments: &serde_json::Map<String, serde_json::Value>,
) -> Option<GetPromptResult> {
    match name {
        "create_sprite" => Some(build_create_sprite(arguments)),
        "create_animation" => Some(build_create_animation(arguments)),
        "review_pxl" => Some(build_review_pxl(arguments)),
        "pixel_art_guide" => Some(build_pixel_art_guide(arguments)),
        _ => None,
    }
}

/// Helper: extract a string argument with an optional default.
fn arg_str<'a>(
    args: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
    default: &'a str,
) -> &'a str {
    args.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}

// ── create_sprite ──────────────────────────────────────────────────────

fn build_create_sprite(args: &serde_json::Map<String, serde_json::Value>) -> GetPromptResult {
    let description = arg_str(args, "description", "a pixel art sprite");
    let size = arg_str(args, "size", "16x16");
    let palette = arg_str(args, "palette", "custom");
    let style = arg_str(args, "style", "retro");

    let palette_note = if palette == "custom" {
        "Define a custom palette with semantic token names (skin, hair, outline, etc.). \
         Include \"_\" mapped to #00000000 for transparency."
            .to_string()
    } else {
        format!(
            "Use the built-in @{palette} palette. Reference it by name in the sprite's \
             \"palette\" field."
        )
    };

    let system_text = format!(
        "{PRIMER_BRIEF}\n\n\
         ## Sprite Creation Guidelines\n\n\
         - Use semantic token names: skin, hair, eye, shirt, outline, highlight, shadow\n\
         - Include \"_\" mapped to #00000000 for transparent background\n\
         - Use an \"outline\" color for definition (darker than base colors)\n\
         - Add highlights and shadows for depth\n\
         - Limit palette to 8-12 colors for a clean look\n\
         - Each region needs a \"z\" value for layering (higher z draws on top)\n\
         - Available shape primitives: rect, points, union, circle, ellipse, line\n\n\
         ## Palette\n\n\
         {palette_note}\n\n\
         ## Style: {style}\n\n\
         Match the \"{style}\" aesthetic in your color choices and level of detail."
    );

    let user_text = format!(
        "Create a {size} pixel art sprite of {description} using the {palette} palette \
         in {style} style. Output valid .pxl JSONL format."
    );

    GetPromptResult {
        description: Some("Guided sprite creation workflow".into()),
        messages: vec![
            PromptMessage::new_text(PromptMessageRole::Assistant, system_text),
            PromptMessage::new_text(PromptMessageRole::User, user_text),
        ],
    }
}

// ── create_animation ───────────────────────────────────────────────────

fn build_create_animation(args: &serde_json::Map<String, serde_json::Value>) -> GetPromptResult {
    let description = arg_str(args, "description", "a pixel art animation");
    let frames = arg_str(args, "frames", "4");
    let fps = arg_str(args, "fps", "8");

    let duration_ms: u32 =
        fps.parse::<u32>().ok().filter(|&f| f > 0).map(|f| 1000 / f).unwrap_or(125);

    let system_text = format!(
        "{PRIMER_BRIEF}\n\n\
         ## Animation Creation Guidelines\n\n\
         - All frames MUST share the same palette (define it once)\n\
         - Keep the subject centered and consistent across frames\n\
         - Only animate the parts that move\n\
         - Use meaningful frame names: name_1, name_2, etc.\n\
         - End with an animation object linking the frames\n\
         - Each frame is a separate sprite object with \"regions\"\n\n\
         ## Frame Timing\n\n\
         - Target: {fps} fps → {duration_ms}ms per frame\n\
         - Walk cycle: 4 frames minimum (left, center, right, center)\n\
         - Idle: 2-4 frames with subtle movement\n\
         - Attack: 3-5 frames (wind-up, strike, recovery)\n\
         - Effects: 4-6 frames, may not loop\n\n\
         ## Animation Object Format\n\n\
         ```json\n\
         {{\"type\": \"animation\", \"name\": \"...\", \"frames\": [\"frame_1\", \"frame_2\"], \
         \"duration\": {duration_ms}, \"loop\": true}}\n\
         ```"
    );

    let user_text = format!(
        "Create a {frames}-frame animation at {fps} fps showing {description}. \
         Output valid .pxl JSONL format with a shared palette, {frames} sprite frames, \
         and an animation object."
    );

    GetPromptResult {
        description: Some("Guided animation creation workflow".into()),
        messages: vec![
            PromptMessage::new_text(PromptMessageRole::Assistant, system_text),
            PromptMessage::new_text(PromptMessageRole::User, user_text),
        ],
    }
}

// ── review_pxl ─────────────────────────────────────────────────────────

fn build_review_pxl(args: &serde_json::Map<String, serde_json::Value>) -> GetPromptResult {
    let source = arg_str(args, "source", "");

    let system_text = format!(
        "{PRIMER_BRIEF}\n\n\
         ## .pxl Review Checklist\n\n\
         Check the source against these rules:\n\n\
         ### Structure\n\
         - Each line is valid JSON (JSONL format)\n\
         - Every object has a \"type\" field (palette, sprite, animation, composition, variant)\n\
         - Sprites reference a valid palette (by name or inline)\n\
         - Animation frames reference existing sprite names\n\n\
         ### Palette\n\
         - Includes \"_\" token for transparency (#00000000)\n\
         - Uses semantic token names (not c1, c2, c3)\n\
         - Colors are valid hex: #RRGGBB or #RRGGBBAA\n\
         - No duplicate token names\n\n\
         ### Sprites\n\
         - Has \"size\": [width, height]\n\
         - Regions use valid shapes: rect, points, union, circle, ellipse, line\n\
         - Regions have \"z\" values for layering\n\
         - No pixels fall outside the declared size\n\
         - Rect format: [x, y, width, height] (not [x1, y1, x2, y2])\n\n\
         ### Common Mistakes\n\
         - Missing \"size\" field on sprites\n\
         - Rect uses [x1, y1, x2, y2] instead of [x, y, w, h]\n\
         - Points outside sprite bounds\n\
         - Referencing undefined palette tokens in regions\n\
         - Animation referencing non-existent frame names\n\
         - Forgetting transparent background token \"_\""
    );

    let user_text =
        format!("Review this .pxl source and suggest improvements:\n\n```\n{source}\n```");

    GetPromptResult {
        description: Some("Review .pxl source for correctness and style".into()),
        messages: vec![
            PromptMessage::new_text(PromptMessageRole::Assistant, system_text),
            PromptMessage::new_text(PromptMessageRole::User, user_text),
        ],
    }
}

// ── pixel_art_guide ────────────────────────────────────────────────────

fn build_pixel_art_guide(args: &serde_json::Map<String, serde_json::Value>) -> GetPromptResult {
    let genre = arg_str(args, "genre", "RPG");

    let palette_catalog = crate::palettes::list_builtins()
        .iter()
        .filter_map(|name| {
            crate::palettes::get_builtin(name).map(|p| {
                let count = p.colors.keys().filter(|k| k.as_str() != "{_}").count();
                format!("- @{name}: {count} colors")
            })
        })
        .collect::<Vec<_>>()
        .join("\n");

    let system_text = format!(
        "## Available Built-in Palettes\n\n\
         {palette_catalog}\n\n\
         Use `pixelsrc_palettes` tool with action=\"show\" to see full color definitions.\n\n\
         ## Pixel Art Style Tips by Genre\n\n\
         ### General Principles\n\
         - Limit palette size: 4-16 colors per sprite for a cohesive look\n\
         - Use 2-3 shades per hue (base, highlight, shadow)\n\
         - Outline with a dark color (not pure black unless 1-bit style)\n\
         - Consistent light source direction across all sprites\n\
         - Leave transparent padding for breathing room\n\n\
         ### Genre-Specific Recommendations\n\n\
         **RPG**: Rich palettes (16-32 colors), detailed characters, layered equipment.\n\
         Recommended palettes: @nes, @pico8\n\n\
         **Platformer**: Bright, high-contrast colors for readability at speed.\n\
         Recommended palettes: @pico8, @nes\n\n\
         **Puzzle**: Clean, distinct colors for game pieces. Minimal detail.\n\
         Recommended palettes: @gameboy, @1bit\n\n\
         **Horror**: Desaturated, dark palettes with selective color accents.\n\
         Recommended palettes: @gameboy (dark), @grayscale\n\n\
         **Sci-fi**: Cool blues and teals, neon accents, metallic grays.\n\
         Recommended palettes: @pico8, custom\n\n\
         **Fantasy**: Warm earth tones, magical purples and golds.\n\
         Recommended palettes: @nes, custom"
    );

    let user_text = format!(
        "Recommend palettes and art style guidelines for a {genre} game using pixelsrc. \
         Include specific palette suggestions, recommended sprite sizes, and style tips."
    );

    GetPromptResult {
        description: Some(format!("Pixel art style guide for {genre} games")),
        messages: vec![
            PromptMessage::new_text(PromptMessageRole::Assistant, system_text),
            PromptMessage::new_text(PromptMessageRole::User, user_text),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_returns_four_prompts() {
        let prompts = list_prompts();
        assert_eq!(prompts.len(), 4);
    }

    #[test]
    fn test_list_prompt_names() {
        let prompts = list_prompts();
        let names: Vec<&str> = prompts.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"create_sprite"));
        assert!(names.contains(&"create_animation"));
        assert!(names.contains(&"review_pxl"));
        assert!(names.contains(&"pixel_art_guide"));
    }

    #[test]
    fn test_all_prompts_have_descriptions_and_arguments() {
        for p in list_prompts() {
            assert!(p.description.is_some(), "{} should have a description", p.name);
            assert!(p.arguments.is_some(), "{} should have arguments", p.name);

            let args = p.arguments.unwrap();
            assert!(!args.is_empty(), "{} should have at least one argument", p.name);

            // First argument should be required
            assert_eq!(
                args[0].required,
                Some(true),
                "{}: first argument should be required",
                p.name
            );
        }
    }

    #[test]
    fn test_get_unknown_prompt_returns_none() {
        let args = serde_json::Map::new();
        assert!(get_prompt("nonexistent", &args).is_none());
    }

    #[test]
    fn test_create_sprite_default_args() {
        let mut args = serde_json::Map::new();
        args.insert("description".into(), serde_json::Value::String("a hero character".into()));

        let result = get_prompt("create_sprite", &args).expect("should return prompt");
        assert_eq!(result.messages.len(), 2);

        // First message is assistant (system context), second is user request
        assert_eq!(result.messages[0].role, PromptMessageRole::Assistant);
        assert_eq!(result.messages[1].role, PromptMessageRole::User);

        // User message should contain the description
        if let PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains("a hero character"));
            assert!(text.contains("16x16")); // default size
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn test_create_sprite_custom_args() {
        let mut args = serde_json::Map::new();
        args.insert("description".into(), serde_json::Value::String("a treasure chest".into()));
        args.insert("size".into(), serde_json::Value::String("32x32".into()));
        args.insert("palette".into(), serde_json::Value::String("pico8".into()));
        args.insert("style".into(), serde_json::Value::String("cute".into()));

        let result = get_prompt("create_sprite", &args).unwrap();

        if let PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains("32x32"));
            assert!(text.contains("a treasure chest"));
            assert!(text.contains("pico8"));
            assert!(text.contains("cute"));
        } else {
            panic!("expected text content");
        }

        // System message should mention the built-in palette
        if let PromptMessageContent::Text { text } = &result.messages[0].content {
            assert!(text.contains("@pico8"));
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn test_create_animation_default_args() {
        let mut args = serde_json::Map::new();
        args.insert("description".into(), serde_json::Value::String("a walking cycle".into()));

        let result = get_prompt("create_animation", &args).unwrap();
        assert_eq!(result.messages.len(), 2);

        if let PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains("4-frame")); // default frames
            assert!(text.contains("8 fps")); // default fps
            assert!(text.contains("a walking cycle"));
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn test_create_animation_custom_fps() {
        let mut args = serde_json::Map::new();
        args.insert("description".into(), serde_json::Value::String("an explosion".into()));
        args.insert("frames".into(), serde_json::Value::String("6".into()));
        args.insert("fps".into(), serde_json::Value::String("12".into()));

        let result = get_prompt("create_animation", &args).unwrap();

        // System should mention calculated duration
        if let PromptMessageContent::Text { text } = &result.messages[0].content {
            assert!(text.contains("83ms"), "should calculate 1000/12 ≈ 83ms per frame");
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn test_review_pxl() {
        let mut args = serde_json::Map::new();
        let source =
            r##"{"type":"palette","name":"test","colors":{"_":"#00000000","a":"#FF0000"}}"##;
        args.insert("source".into(), serde_json::Value::String(source.into()));

        let result = get_prompt("review_pxl", &args).unwrap();
        assert_eq!(result.messages.len(), 2);

        // System message should contain review checklist
        if let PromptMessageContent::Text { text } = &result.messages[0].content {
            assert!(text.contains("Review Checklist"));
            assert!(text.contains("Common Mistakes"));
        } else {
            panic!("expected text content");
        }

        // User message should contain the source
        if let PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains(source));
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn test_pixel_art_guide() {
        let mut args = serde_json::Map::new();
        args.insert("genre".into(), serde_json::Value::String("platformer".into()));

        let result = get_prompt("pixel_art_guide", &args).unwrap();
        assert_eq!(result.messages.len(), 2);

        // System should contain palette catalog
        if let PromptMessageContent::Text { text } = &result.messages[0].content {
            assert!(text.contains("@gameboy"));
            assert!(text.contains("@pico8"));
        } else {
            panic!("expected text content");
        }

        // User message should mention the genre
        if let PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains("platformer"));
        } else {
            panic!("expected text content");
        }

        // Description should mention genre
        assert!(result.description.unwrap().contains("platformer"));
    }

    #[test]
    fn test_all_prompts_produce_valid_results() {
        let test_cases = vec![
            ("create_sprite", vec![("description", "a knight")]),
            ("create_animation", vec![("description", "a coin spin")]),
            ("review_pxl", vec![("source", "{}")]),
            ("pixel_art_guide", vec![("genre", "RPG")]),
        ];

        for (name, arg_pairs) in test_cases {
            let mut args = serde_json::Map::new();
            for (k, v) in arg_pairs {
                args.insert(k.into(), serde_json::Value::String(v.into()));
            }

            let result = get_prompt(name, &args).unwrap_or_else(|| panic!("{name} should exist"));
            assert!(!result.messages.is_empty(), "{name} should return messages");
            assert!(result.description.is_some(), "{name} should have a description");

            // All messages should have non-empty text content
            for (i, msg) in result.messages.iter().enumerate() {
                if let PromptMessageContent::Text { text } = &msg.content {
                    assert!(!text.is_empty(), "{name} message {i} should have content");
                }
            }
        }
    }
}
