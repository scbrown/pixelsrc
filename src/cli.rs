//! Command-line interface implementation

use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::include::{extract_include_path, is_include_ref, resolve_include_with_detection};
use crate::models::{PaletteRef, TtpObject};
use crate::output::{generate_output_path, save_png, scale_image};
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, PaletteSource, ResolvedPalette};
use crate::renderer::render_sprite;

/// Exit codes per TTP spec
const EXIT_SUCCESS: u8 = 0;
const EXIT_ERROR: u8 = 1;
const EXIT_INVALID_ARGS: u8 = 2;

/// TTP (Text To Pixel) - Parse JSONL pixel art definitions and render to PNG
#[derive(Parser)]
#[command(name = "pxl")]
#[command(about = "TTP (Text To Pixel) - Parse JSONL pixel art definitions and render to PNG")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Render sprites from a TTP JSONL file to PNG
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
        } => run_render(&input, output.as_deref(), sprite.as_deref(), strict, scale),
    }
}

/// Execute the render command
fn run_render(
    input: &PathBuf,
    output: Option<&std::path::Path>,
    sprite_filter: Option<&str>,
    strict: bool,
    scale: u8,
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

    // Build palette registry and collect sprites
    let mut registry = PaletteRegistry::new();
    let mut sprites_by_name: std::collections::HashMap<String, crate::models::Sprite> =
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
            TtpObject::Animation(_) => {
                // Animations are Phase 3, skip for now
            }
            TtpObject::Composition(_) => {
                // Composition rendering in CLI is Task 2.7, skip for now
            }
        }
    }

    // Convert to Vec, preserving insertion order (last wins for duplicates)
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

    // Get the input file's parent directory for resolving includes
    let input_dir = input
        .parent()
        .unwrap_or(std::path::Path::new("."));

    // Track visited files for circular include detection
    let mut include_visited: HashSet<PathBuf> = HashSet::new();

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
