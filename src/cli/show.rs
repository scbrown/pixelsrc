//! Show command implementation (terminal display)

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::models::{Animation, Sprite, TtpObject};
use crate::onion::{parse_hex_color, render_onion_skin, OnionConfig};
use crate::parser::parse_stream;
use crate::registry::PaletteRegistry;
use crate::renderer::render_sprite;
use crate::suggest::{format_suggestion, suggest};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the show command - display sprite with colored terminal output
pub fn run_show(
    file: &PathBuf,
    sprite_filter: Option<&str>,
    animation_filter: Option<&str>,
    frame_index: usize,
    onion_count: Option<u32>,
    onion_opacity: f32,
    onion_prev_color: &str,
    onion_next_color: &str,
    onion_fade: bool,
    output: Option<&Path>,
) -> ExitCode {
    // Open input file
    let input_file = match File::open(file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Cannot open input file '{}': {}", file.display(), e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    // Parse JSONL stream
    let reader = BufReader::new(input_file);
    let parse_result = parse_stream(reader);

    // Collect sprites, animations, and palettes
    let mut sprites_by_name: HashMap<String, Sprite> = HashMap::new();
    let mut animations_by_name: HashMap<String, Animation> = HashMap::new();
    let mut registry = PaletteRegistry::new();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(palette) => {
                registry.register(palette);
            }
            TtpObject::Sprite(sprite) => {
                sprites_by_name.insert(sprite.name.clone(), sprite);
            }
            TtpObject::Animation(animation) => {
                animations_by_name.insert(animation.name.clone(), animation);
            }
            _ => {}
        }
    }

    // Handle onion skinning mode (animation + --onion flag)
    if let Some(onion) = onion_count {
        let animation = if let Some(name) = animation_filter {
            match animations_by_name.get(name) {
                Some(a) => a,
                None => {
                    eprintln!("Error: No animation named '{}' found in input", name);
                    let anim_names: Vec<&str> =
                        animations_by_name.keys().map(|s| s.as_str()).collect();
                    if let Some(suggestion) = format_suggestion(&suggest(name, &anim_names, 3)) {
                        eprintln!("{}", suggestion);
                    }
                    return ExitCode::from(EXIT_ERROR);
                }
            }
        } else {
            // Use the first animation found
            match animations_by_name.values().next() {
                Some(a) => a,
                None => {
                    eprintln!(
                        "Error: No animations found in input file (--onion requires an animation)"
                    );
                    return ExitCode::from(EXIT_ERROR);
                }
            }
        };

        if animation.frames.is_empty() {
            eprintln!("Error: Animation '{}' has no frames", animation.name);
            return ExitCode::from(EXIT_ERROR);
        }

        // Parse tint colors
        let prev_color = match parse_hex_color(onion_prev_color) {
            Some(c) => c,
            None => {
                eprintln!("Error: Invalid color for --onion-prev-color: {}", onion_prev_color);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };

        let next_color = match parse_hex_color(onion_next_color) {
            Some(c) => c,
            None => {
                eprintln!("Error: Invalid color for --onion-next-color: {}", onion_next_color);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };

        // Render all frames to images
        let mut frame_images = Vec::new();
        for frame_name in &animation.frames {
            let sprite = match sprites_by_name.get(frame_name) {
                Some(s) => s,
                None => {
                    eprintln!("Error: Animation frame '{}' not found in sprites", frame_name);
                    return ExitCode::from(EXIT_ERROR);
                }
            };

            let resolved_palette = match registry.resolve(sprite, false) {
                Ok(result) => result.palette.colors,
                Err(e) => {
                    eprintln!("Error: sprite '{}': {}", sprite.name, e);
                    return ExitCode::from(EXIT_ERROR);
                }
            };

            let (image, _warnings) = render_sprite(sprite, &resolved_palette);
            frame_images.push(image);
        }

        // Create onion skin config
        let config = OnionConfig {
            count: onion,
            opacity: onion_opacity.clamp(0.0, 1.0),
            prev_color,
            next_color,
            fade: onion_fade,
        };

        // Render with onion skinning
        let frame_idx = frame_index.min(frame_images.len().saturating_sub(1));
        let result = render_onion_skin(&frame_images, frame_idx, &config);

        // Output to file or terminal
        if let Some(output_path) = output {
            if let Err(e) = result.save(output_path) {
                eprintln!("Error: Failed to save image: {}", e);
                return ExitCode::from(EXIT_ERROR);
            }
            println!(
                "Onion skin preview saved: {} (frame {}/{}, {} ghost frames)",
                output_path.display(),
                frame_idx + 1,
                animation.frames.len(),
                onion
            );
        } else {
            // Render to terminal using ANSI
            println!(
                "Animation: {} (frame {}/{}, {} ghost frames)",
                animation.name,
                frame_idx + 1,
                animation.frames.len(),
                onion
            );
            println!();

            // Convert image to terminal output
            use crate::terminal::render_image_ansi;
            let ansi_output = render_image_ansi(&result);
            print!("{}", ansi_output);

            println!();
            println!("Legend: Previous frames = blue tint, Next frames = green tint");
        }

        return ExitCode::from(EXIT_SUCCESS);
    }

    // Standard sprite display mode (no onion skinning)
    if sprites_by_name.is_empty() {
        eprintln!("Error: No sprites found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    // Find the sprite to display
    let sprite = if let Some(name) = sprite_filter {
        match sprites_by_name.get(name) {
            Some(s) => s,
            None => {
                eprintln!("Error: No sprite named '{}' found in input", name);
                let sprite_names: Vec<&str> = sprites_by_name.keys().map(|s| s.as_str()).collect();
                if let Some(suggestion) = format_suggestion(&suggest(name, &sprite_names, 3)) {
                    eprintln!("{}", suggestion);
                }
                return ExitCode::from(EXIT_ERROR);
            }
        }
    } else {
        // Use the first sprite found
        match sprites_by_name.values().next() {
            Some(s) => s,
            None => {
                eprintln!("Error: No sprites found in input file");
                return ExitCode::from(EXIT_ERROR);
            }
        }
    };

    // Grid-based terminal rendering is deprecated
    // Use render command to generate PNG output instead
    eprintln!("Error: Terminal view requires grid format, which is deprecated.");
    eprintln!(
        "Use 'pxl render' to generate PNG output, or use sprites with structured regions format."
    );

    // Show basic sprite info
    let (width, height) =
        if let Some(size) = sprite.size { (size[0] as usize, size[1] as usize) } else { (0, 0) };
    println!("Sprite: {} ({}x{})", sprite.name, width, height);
    println!("Has regions: {}", sprite.regions.is_some());

    ExitCode::from(EXIT_ERROR)
}
