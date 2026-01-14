//! Command-line interface implementation

use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::gif::render_gif;
use crate::include::{extract_include_path, is_include_ref, resolve_include_with_detection};
use crate::models::{Animation, PaletteRef, Sprite, TtpObject};
use crate::output::{generate_output_path, save_png, scale_image};
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, PaletteSource, ResolvedPalette};
use crate::renderer::render_sprite;
use crate::spritesheet::render_spritesheet;

/// Exit codes per Pixelsrc spec
const EXIT_SUCCESS: u8 = 0;
const EXIT_ERROR: u8 = 1;
const EXIT_INVALID_ARGS: u8 = 2;

/// Pixelsrc - Parse JSONL pixel art definitions and render to PNG
#[derive(Parser)]
#[command(name = "pxl")]
#[command(about = "Pixelsrc - Parse JSONL pixel art definitions and render to PNG")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Render sprites from a Pixelsrc JSONL file to PNG
    Render {
        /// Input JSONL file containing palette and sprite definitions
        input: PathBuf,

        /// Output file or directory.
        /// If omitted: {input}_{sprite}.png
        /// If file (single sprite): output.png
        /// If file (multiple): output_{sprite}.png
        /// If directory (ends with /): dir/{sprite}.png
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Only render the sprite with this name
        #[arg(short, long)]
        sprite: Option<String>,

        /// Strict mode: treat warnings as errors
        #[arg(long)]
        strict: bool,

        /// Scale output by integer factor (1-16, default: 1)
        #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=16))]
        scale: u8,

        /// Output as animated GIF (requires animation in input)
        #[arg(long)]
        gif: bool,

        /// Output as spritesheet (horizontal strip of all frames)
        #[arg(long)]
        spritesheet: bool,

        /// Select a specific animation by name
        #[arg(long)]
        animation: Option<String>,
    },
}

/// Run the CLI application
pub fn run() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render {
            input,
            output,
            sprite,
            strict,
            scale,
            gif,
            spritesheet,
            animation,
        } => run_render(
            &input,
            output.as_deref(),
            sprite.as_deref(),
            strict,
            scale,
            gif,
            spritesheet,
            animation.as_deref(),
        ),
    }
}

/// Execute the render command
#[allow(clippy::too_many_arguments)]
fn run_render(
    input: &PathBuf,
    output: Option<&std::path::Path>,
    sprite_filter: Option<&str>,
    strict: bool,
    scale: u8,
    gif_output: bool,
    spritesheet_output: bool,
    animation_filter: Option<&str>,
) -> ExitCode {
    // Open input file
    let file = match File::open(input) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Cannot open input file '{}': {}", input.display(), e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    // Parse JSONL stream
    let reader = BufReader::new(file);
    let parse_result = parse_stream(reader);

    // Collect all warnings
    let mut all_warnings: Vec<String> = Vec::new();

    // Add parse warnings
    for warning in &parse_result.warnings {
        all_warnings.push(format!("line {}: {}", warning.line, warning.message));
    }

    // In strict mode, parse warnings are fatal
    if strict && !parse_result.warnings.is_empty() {
        for warning in &all_warnings {
            eprintln!("Error: {}", warning);
        }
        return ExitCode::from(EXIT_ERROR);
    }

    // Build palette registry and collect sprites and animations
    let mut registry = PaletteRegistry::new();
    let mut sprites_by_name: std::collections::HashMap<String, Sprite> =
        std::collections::HashMap::new();
    let mut animations_by_name: std::collections::HashMap<String, Animation> =
        std::collections::HashMap::new();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(palette) => {
                registry.register(palette);
            }
            TtpObject::Sprite(sprite) => {
                if sprites_by_name.contains_key(&sprite.name) {
                    let warning_msg =
                        format!("Duplicate sprite name '{}', using latest", sprite.name);
                    all_warnings.push(warning_msg);
                    if strict {
                        for warning in &all_warnings {
                            eprintln!("Error: {}", warning);
                        }
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
                sprites_by_name.insert(sprite.name.clone(), sprite);
            }
            TtpObject::Animation(anim) => {
                if animations_by_name.contains_key(&anim.name) {
                    let warning_msg =
                        format!("Duplicate animation name '{}', using latest", anim.name);
                    all_warnings.push(warning_msg);
                    if strict {
                        for warning in &all_warnings {
                            eprintln!("Error: {}", warning);
                        }
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
                animations_by_name.insert(anim.name.clone(), anim);
            }
            TtpObject::Composition(_) => {
                // Composition rendering in CLI is Task 2.7, skip for now
            }
        }
    }

    // Get the input file's parent directory for resolving includes
    let input_dir = input.parent().unwrap_or(std::path::Path::new("."));

    // Track visited files for circular include detection
    let mut include_visited: HashSet<PathBuf> = HashSet::new();

    // Handle animation rendering (--gif or --spritesheet)
    if gif_output || spritesheet_output {
        return run_animation_render(
            input,
            output,
            &animations_by_name,
            &sprites_by_name,
            &registry,
            input_dir,
            &mut include_visited,
            &mut all_warnings,
            strict,
            scale,
            gif_output,
            animation_filter,
        );
    }

    // Convert to Vec for sprite rendering
    let mut sprites: Vec<_> = sprites_by_name.into_values().collect();

    // Filter sprites if --sprite is provided
    if let Some(name) = sprite_filter {
        sprites.retain(|s| s.name == name);
        if sprites.is_empty() {
            eprintln!("Error: No sprite named '{}' found in input", name);
            return ExitCode::from(EXIT_ERROR);
        }
    }

    // Check if we have any sprites to render
    if sprites.is_empty() {
        eprintln!("Error: No sprites found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    let is_single_sprite = sprites.len() == 1;

    // Render each sprite
    for sprite in &sprites {
        // Resolve palette - handle @include: syntax specially
        let resolved = match &sprite.palette {
            PaletteRef::Named(name) if is_include_ref(name) => {
                // Handle @include:path syntax
                let include_path = extract_include_path(name).unwrap();
                match resolve_include_with_detection(include_path, input_dir, &mut include_visited) {
                    Ok(palette) => ResolvedPalette {
                        colors: palette.colors,
                        source: PaletteSource::Named(format!("@include:{}", include_path)),
                    },
                    Err(e) => {
                        if strict {
                            eprintln!("Error: sprite '{}': {}", sprite.name, e);
                            return ExitCode::from(EXIT_ERROR);
                        }
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, e));
                        // Fallback to empty palette in lenient mode
                        ResolvedPalette {
                            colors: std::collections::HashMap::new(),
                            source: PaletteSource::Fallback,
                        }
                    }
                }
            }
            _ => {
                // Normal palette resolution via registry
                match registry.resolve(sprite, strict) {
                    Ok(result) => {
                        if let Some(warning) = result.warning {
                            all_warnings.push(format!(
                                "sprite '{}': {}",
                                sprite.name, warning.message
                            ));
                            if strict {
                                for warning in &all_warnings {
                                    eprintln!("Error: {}", warning);
                                }
                                return ExitCode::from(EXIT_ERROR);
                            }
                        }
                        result.palette
                    }
                    Err(e) => {
                        eprintln!("Error: sprite '{}': {}", sprite.name, e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
            }
        };

        // Render sprite
        let (image, render_warnings) = render_sprite(sprite, &resolved.colors);

        // Apply scaling if requested
        let image = scale_image(image, scale);

        // Collect render warnings
        for warning in render_warnings {
            all_warnings.push(format!("sprite '{}': {}", sprite.name, warning.message));
        }

        // In strict mode, render warnings are fatal
        if strict && !all_warnings.is_empty() {
            for warning in &all_warnings {
                eprintln!("Error: {}", warning);
            }
            return ExitCode::from(EXIT_ERROR);
        }

        // Generate output path
        let output_path = generate_output_path(input, &sprite.name, output, is_single_sprite);

        // Save PNG
        if let Err(e) = save_png(&image, &output_path) {
            eprintln!("Error: Failed to save '{}': {}", output_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }

        println!("Saved: {}", output_path.display());
    }

    // Print warnings to stderr (in lenient mode)
    for warning in &all_warnings {
        eprintln!("Warning: {}", warning);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Render an animation as GIF or spritesheet
#[allow(clippy::too_many_arguments)]
fn run_animation_render(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    animations: &std::collections::HashMap<String, Animation>,
    sprites: &std::collections::HashMap<String, Sprite>,
    registry: &PaletteRegistry,
    input_dir: &std::path::Path,
    include_visited: &mut HashSet<PathBuf>,
    all_warnings: &mut Vec<String>,
    strict: bool,
    scale: u8,
    gif_output: bool,
    animation_filter: Option<&str>,
) -> ExitCode {
    // Find the animation to render
    let animation = if let Some(name) = animation_filter {
        match animations.get(name) {
            Some(anim) => anim,
            None => {
                eprintln!("Error: No animation named '{}' found in input", name);
                return ExitCode::from(EXIT_ERROR);
            }
        }
    } else {
        // Use the first animation found
        match animations.values().next() {
            Some(anim) => anim,
            None => {
                eprintln!("Error: No animations found in input file");
                return ExitCode::from(EXIT_ERROR);
            }
        }
    };

    // Validate animation: check that all frame references exist
    let mut missing_frames = Vec::new();
    for frame_name in &animation.frames {
        if !sprites.contains_key(frame_name) {
            missing_frames.push(frame_name.clone());
        }
    }

    if !missing_frames.is_empty() {
        let warning_msg = format!(
            "Animation '{}' references missing sprites: {}",
            animation.name,
            missing_frames.join(", ")
        );
        if strict {
            eprintln!("Error: {}", warning_msg);
            return ExitCode::from(EXIT_ERROR);
        }
        all_warnings.push(warning_msg);
    }

    if animation.frames.is_empty() {
        let warning_msg = format!("Animation '{}' has no frames", animation.name);
        if strict {
            eprintln!("Error: {}", warning_msg);
            return ExitCode::from(EXIT_ERROR);
        }
        all_warnings.push(warning_msg);
    }

    // Render each frame
    let mut frame_images = Vec::new();
    for frame_name in &animation.frames {
        let sprite = match sprites.get(frame_name) {
            Some(s) => s,
            None => continue, // Skip missing sprites (warned above)
        };

        // Resolve palette
        let resolved = match &sprite.palette {
            PaletteRef::Named(name) if is_include_ref(name) => {
                let include_path = extract_include_path(name).unwrap();
                match resolve_include_with_detection(include_path, input_dir, include_visited) {
                    Ok(palette) => ResolvedPalette {
                        colors: palette.colors,
                        source: PaletteSource::Named(format!("@include:{}", include_path)),
                    },
                    Err(e) => {
                        if strict {
                            eprintln!("Error: sprite '{}': {}", sprite.name, e);
                            return ExitCode::from(EXIT_ERROR);
                        }
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, e));
                        ResolvedPalette {
                            colors: std::collections::HashMap::new(),
                            source: PaletteSource::Fallback,
                        }
                    }
                }
            }
            _ => match registry.resolve(sprite, strict) {
                Ok(result) => {
                    if let Some(warning) = result.warning {
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, warning.message));
                        if strict {
                            for warning in all_warnings.iter() {
                                eprintln!("Error: {}", warning);
                            }
                            return ExitCode::from(EXIT_ERROR);
                        }
                    }
                    result.palette
                }
                Err(e) => {
                    eprintln!("Error: sprite '{}': {}", sprite.name, e);
                    return ExitCode::from(EXIT_ERROR);
                }
            },
        };

        // Render sprite
        let (image, render_warnings) = render_sprite(sprite, &resolved.colors);

        // Apply scaling if requested
        let image = scale_image(image, scale);

        // Collect render warnings
        for warning in render_warnings {
            all_warnings.push(format!("sprite '{}': {}", sprite.name, warning.message));
        }

        if strict && !all_warnings.is_empty() {
            for warning in all_warnings.iter() {
                eprintln!("Error: {}", warning);
            }
            return ExitCode::from(EXIT_ERROR);
        }

        frame_images.push(image);
    }

    if frame_images.is_empty() {
        eprintln!("Error: No valid frames to render in animation '{}'", animation.name);
        return ExitCode::from(EXIT_ERROR);
    }

    // Generate output path
    let output_path = if let Some(path) = output {
        path.to_path_buf()
    } else {
        // Default: input_animation.gif or input_animation.png
        let extension = if gif_output { "gif" } else { "png" };
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        input
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_{}.{}", stem, animation.name, extension))
    };

    // Output as GIF or spritesheet
    if gif_output {
        if let Err(e) = render_gif(
            &frame_images,
            animation.duration_ms(),
            animation.loops(),
            &output_path,
        ) {
            eprintln!("Error: Failed to save GIF '{}': {}", output_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }
    } else {
        // Spritesheet output
        let sheet = render_spritesheet(&frame_images, None);
        if let Err(e) = save_png(&sheet, &output_path) {
            eprintln!("Error: Failed to save spritesheet '{}': {}", output_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }
    }

    println!("Saved: {}", output_path.display());

    // Print warnings to stderr (in lenient mode)
    for warning in all_warnings.iter() {
        eprintln!("Warning: {}", warning);
    }

    ExitCode::from(EXIT_SUCCESS)
}
