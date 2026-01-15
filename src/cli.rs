//! Command-line interface implementation

use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::analyze::{collect_files, format_report_text, AnalysisReport};
use crate::composition::render_composition;
#[allow(unused_imports)]
use crate::emoji::render_emoji_art;
use crate::fmt::format_pixelsrc;
use crate::prime::{get_primer, list_sections, PrimerSection};
use glob::glob;

/// Check if a path has a valid Pixelsrc file extension (.pxl or .jsonl).
pub fn is_pixelsrc_file(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("pxl") | Some("jsonl")
    )
}

/// Find all Pixelsrc files in a directory (recursively).
///
/// Searches for both `.pxl` and `.jsonl` files.
pub fn find_pixelsrc_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let dir_str = dir.display().to_string();

    // Search for .pxl files
    if let Ok(paths) = glob(&format!("{}/**/*.pxl", dir_str)) {
        files.extend(paths.filter_map(Result::ok));
    }

    // Search for .jsonl files
    if let Ok(paths) = glob(&format!("{}/**/*.jsonl", dir_str)) {
        files.extend(paths.filter_map(Result::ok));
    }

    files
}
use crate::gif::render_gif;
use crate::import::import_png;
use crate::include::{extract_include_path, is_include_ref, resolve_include_with_detection};
use crate::models::{Animation, Composition, PaletteRef, Sprite, TtpObject};
use crate::output::{generate_output_path, save_png, scale_image};
use crate::palettes;
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, PaletteSource, ResolvedPalette};
use crate::renderer::render_sprite;
use crate::spritesheet::render_spritesheet;

/// Exit codes per Pixelsrc spec
const EXIT_SUCCESS: u8 = 0;
const EXIT_ERROR: u8 = 1;
const EXIT_INVALID_ARGS: u8 = 2;

/// Pixelsrc - Parse pixel art definitions and render to PNG
#[derive(Parser)]
#[command(name = "pxl")]
#[command(about = "Pixelsrc - Parse pixel art definitions (.pxl, .jsonl) and render to PNG")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Render sprites from a Pixelsrc file to PNG
    Render {
        /// Input file containing palette and sprite definitions (.pxl or .jsonl)
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

        /// Only render the composition with this name
        #[arg(short = 'c', long)]
        composition: Option<String>,

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

        /// Output as emoji art to terminal (for quick preview)
        #[arg(long)]
        emoji: bool,

        /// Select a specific animation by name
        #[arg(long)]
        animation: Option<String>,
    },
    /// Import a PNG image and convert to Pixelsrc format
    Import {
        /// Input PNG file to convert
        input: PathBuf,

        /// Output file (default: {input}.jsonl, use .pxl extension for new format)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum number of colors in the palette (2-256, default: 16)
        #[arg(long, default_value = "16")]
        max_colors: usize,

        /// Name for the generated sprite (default: derived from filename)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Show GenAI prompt templates for sprite generation
    Prompts {
        /// Template name to show (character, item, tileset, animation)
        /// If omitted, lists all available templates
        #[arg()]
        template: Option<String>,
    },
    /// List and inspect built-in palettes
    Palettes {
        #[command(subcommand)]
        action: PaletteAction,
    },

    /// Analyze pixelsrc files and extract corpus metrics
    Analyze {
        /// Files to analyze
        #[arg(required_unless_present = "dir")]
        files: Vec<PathBuf>,

        /// Directory to scan for .jsonl/.pxl files
        #[arg(long)]
        dir: Option<PathBuf>,

        /// Include subdirectories when scanning a directory
        #[arg(long, short)]
        recursive: bool,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Format pixelsrc files for readability
    Fmt {
        /// Input file(s) to format
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Check formatting without writing (exit 1 if changes needed)
        #[arg(long)]
        check: bool,

        /// Write to stdout instead of in-place
        #[arg(long)]
        stdout: bool,
    },

    /// Print pixelsrc format guide for AI context injection
    Prime {
        /// Print condensed version (~2000 tokens)
        #[arg(long)]
        brief: bool,

        /// Print specific section: format, examples, tips, full
        #[arg(long)]
        section: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PaletteAction {
    /// List all available built-in palettes
    List,
    /// Show details of a specific palette
    Show {
        /// Name of the palette to show
        name: String,
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
            composition,
            strict,
            scale,
            gif,
            spritesheet,
            emoji,
            animation,
        } => run_render(
            &input,
            output.as_deref(),
            sprite.as_deref(),
            composition.as_deref(),
            strict,
            scale,
            gif,
            spritesheet,
            emoji,
            animation.as_deref(),
        ),
        Commands::Import {
            input,
            output,
            max_colors,
            name,
        } => run_import(&input, output.as_deref(), max_colors, name.as_deref()),
        Commands::Prompts { template } => run_prompts(template.as_deref()),
        Commands::Palettes { action } => run_palettes(action),
        Commands::Analyze {
            files,
            dir,
            recursive,
            format,
            output,
        } => run_analyze(&files, dir.as_deref(), recursive, &format, output.as_deref()),
        Commands::Fmt {
            files,
            check,
            stdout,
        } => run_fmt(&files, check, stdout),
        Commands::Prime { brief, section } => run_prime(brief, section.as_deref()),
    }
}

// Embedded prompt templates
const TEMPLATE_CHARACTER: &str = include_str!("../docs/prompts/templates/character.txt");
const TEMPLATE_ITEM: &str = include_str!("../docs/prompts/templates/item.txt");
const TEMPLATE_TILESET: &str = include_str!("../docs/prompts/templates/tileset.txt");
const TEMPLATE_ANIMATION: &str = include_str!("../docs/prompts/templates/animation.txt");

/// Available template names
const TEMPLATES: &[(&str, &str)] = &[
    ("character", TEMPLATE_CHARACTER),
    ("item", TEMPLATE_ITEM),
    ("tileset", TEMPLATE_TILESET),
    ("animation", TEMPLATE_ANIMATION),
];

/// Execute the prime command
fn run_prime(brief: bool, section: Option<&str>) -> ExitCode {
    // Parse section if provided
    let primer_section = match section {
        None => PrimerSection::Full,
        Some(s) => match s.parse::<PrimerSection>() {
            Ok(sec) => sec,
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!();
                eprintln!("Available sections:");
                for sec in list_sections() {
                    eprintln!("  {}", sec);
                }
                return ExitCode::from(EXIT_ERROR);
            }
        },
    };

    // Get and print the primer content
    let content = get_primer(primer_section, brief);
    println!("{}", content);
    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the prompts command
fn run_prompts(template: Option<&str>) -> ExitCode {
    match template {
        None => {
            // List available templates
            println!("Available prompt templates:");
            println!();
            for (name, _) in TEMPLATES {
                println!("  {}", name);
            }
            println!();
            println!("Usage: pxl prompts <template>");
            println!();
            println!("Templates are designed for use with Claude, GPT, or other LLMs.");
            println!("See docs/prompts/ for full documentation and examples.");
            ExitCode::from(EXIT_SUCCESS)
        }
        Some(name) => {
            // Show specific template
            for (tpl_name, content) in TEMPLATES {
                if *tpl_name == name {
                    println!("{}", content);
                    return ExitCode::from(EXIT_SUCCESS);
                }
            }
            // Template not found
            eprintln!("Error: Unknown template '{}'", name);
            eprintln!();
            eprintln!("Available templates:");
            for (tpl_name, _) in TEMPLATES {
                eprintln!("  {}", tpl_name);
            }
            ExitCode::from(EXIT_ERROR)
        }
    }
}

/// Execute the palettes command
fn run_palettes(action: PaletteAction) -> ExitCode {
    match action {
        PaletteAction::List => {
            println!("Built-in palettes:");
            for name in palettes::list_builtins() {
                println!("  @{}", name);
            }
            ExitCode::from(EXIT_SUCCESS)
        }
        PaletteAction::Show { name } => {
            let palette_name = name.strip_prefix('@').unwrap_or(&name);
            match palettes::get_builtin(palette_name) {
                Some(palette) => {
                    println!("Palette: @{}", palette_name);
                    println!();
                    for (key, color) in &palette.colors {
                        println!("  {} => {}", key, color);
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                None => {
                    eprintln!("Error: Unknown palette '{}'", name);
                    eprintln!();
                    eprintln!("Available palettes:");
                    for builtin_name in palettes::list_builtins() {
                        eprintln!("  @{}", builtin_name);
                    }
                    ExitCode::from(EXIT_ERROR)
                }
            }
        }
    }
}

/// Execute the render command
#[allow(clippy::too_many_arguments)]
fn run_render(
    input: &PathBuf,
    output: Option<&std::path::Path>,
    sprite_filter: Option<&str>,
    composition_filter: Option<&str>,
    strict: bool,
    scale: u8,
    gif_output: bool,
    spritesheet_output: bool,
    _emoji_output: bool,
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

    // Build palette registry and collect sprites, animations, and compositions
    let mut registry = PaletteRegistry::new();
    let mut sprites_by_name: std::collections::HashMap<String, Sprite> =
        std::collections::HashMap::new();
    let mut animations_by_name: std::collections::HashMap<String, Animation> =
        std::collections::HashMap::new();
    let mut compositions_by_name: std::collections::HashMap<String, Composition> =
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
            TtpObject::Composition(comp) => {
                if compositions_by_name.contains_key(&comp.name) {
                    let warning_msg =
                        format!("Duplicate composition name '{}', using latest", comp.name);
                    all_warnings.push(warning_msg);
                    if strict {
                        for warning in &all_warnings {
                            eprintln!("Error: {}", warning);
                        }
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
                compositions_by_name.insert(comp.name.clone(), comp);
            }
            TtpObject::Variant(_) => {
                // Variant rendering in CLI is handled by resolving to sprites
                // The variant will be processed when rendering sprites
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

    // Handle composition rendering if --composition is provided
    if let Some(comp_name) = composition_filter {
        return run_composition_render(
            input,
            output,
            comp_name,
            &compositions_by_name,
            &sprites_by_name,
            &registry,
            input_dir,
            &mut include_visited,
            &mut all_warnings,
            strict,
            scale,
        );
    }

    // Determine what to render: sprites and/or compositions
    let render_sprites = sprite_filter.is_some() || !sprites_by_name.is_empty();
    let render_compositions = !compositions_by_name.is_empty() && sprite_filter.is_none();

    // Convert to Vec for sprite rendering
    let mut sprites: Vec<_> = sprites_by_name.values().cloned().collect();

    // Filter sprites if --sprite is provided
    if let Some(name) = sprite_filter {
        sprites.retain(|s| s.name == name);
        if sprites.is_empty() {
            eprintln!("Error: No sprite named '{}' found in input", name);
            return ExitCode::from(EXIT_ERROR);
        }
    }

    // Check if we have anything to render
    if sprites.is_empty() && compositions_by_name.is_empty() {
        eprintln!("Error: No sprites or compositions found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    let is_single_output = sprites.len() == 1 && compositions_by_name.is_empty();

    // Render each sprite
    if render_sprites {
        for sprite in &sprites {
            // Resolve palette - handle @include: syntax specially
            let resolved = match &sprite.palette {
                PaletteRef::Named(name) if is_include_ref(name) => {
                    // Handle @include:path syntax
                    let include_path = extract_include_path(name).unwrap();
                    match resolve_include_with_detection(
                        include_path,
                        input_dir,
                        &mut include_visited,
                    ) {
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
                                all_warnings
                                    .push(format!("sprite '{}': {}", sprite.name, warning.message));
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
            let output_path = generate_output_path(input, &sprite.name, output, is_single_output);

            // Save PNG
            if let Err(e) = save_png(&image, &output_path) {
                eprintln!("Error: Failed to save '{}': {}", output_path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }

            println!("Saved: {}", output_path.display());
        }
    }

    // Render compositions (when no --sprite filter is active)
    if render_compositions {
        for (comp_name, comp) in &compositions_by_name {
            // Render the composition
            let result = render_composition_to_image(
                comp,
                &sprites_by_name,
                &registry,
                input_dir,
                &mut include_visited,
                &mut all_warnings,
                strict,
            );

            let image = match result {
                Ok(img) => img,
                Err(code) => return code,
            };

            // Apply scaling if requested
            let image = scale_image(image, scale);

            // In strict mode, check for accumulated warnings
            if strict && !all_warnings.is_empty() {
                for warning in &all_warnings {
                    eprintln!("Error: {}", warning);
                }
                return ExitCode::from(EXIT_ERROR);
            }

            // Generate output path
            let is_single = compositions_by_name.len() == 1 && sprites.is_empty();
            let output_path = generate_output_path(input, comp_name, output, is_single);

            // Save PNG
            if let Err(e) = save_png(&image, &output_path) {
                eprintln!("Error: Failed to save '{}': {}", output_path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }

            println!("Saved: {}", output_path.display());
        }
    }

    // Print warnings to stderr (in lenient mode)
    for warning in &all_warnings {
        eprintln!("Warning: {}", warning);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Render a specific composition
#[allow(clippy::too_many_arguments)]
fn run_composition_render(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    comp_name: &str,
    compositions: &std::collections::HashMap<String, Composition>,
    sprites: &std::collections::HashMap<String, Sprite>,
    registry: &PaletteRegistry,
    input_dir: &std::path::Path,
    include_visited: &mut HashSet<PathBuf>,
    all_warnings: &mut Vec<String>,
    strict: bool,
    scale: u8,
) -> ExitCode {
    // Find the composition
    let comp = match compositions.get(comp_name) {
        Some(c) => c,
        None => {
            eprintln!("Error: No composition named '{}' found in input", comp_name);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Render the composition
    let result = render_composition_to_image(
        comp,
        sprites,
        registry,
        input_dir,
        include_visited,
        all_warnings,
        strict,
    );

    let image = match result {
        Ok(img) => img,
        Err(code) => return code,
    };

    // Apply scaling if requested
    let image = scale_image(image, scale);

    // In strict mode, check for accumulated warnings
    if strict && !all_warnings.is_empty() {
        for warning in all_warnings.iter() {
            eprintln!("Error: {}", warning);
        }
        return ExitCode::from(EXIT_ERROR);
    }

    // Generate output path
    let output_path = generate_output_path(input, comp_name, output, true);

    // Save PNG
    if let Err(e) = save_png(&image, &output_path) {
        eprintln!("Error: Failed to save '{}': {}", output_path.display(), e);
        return ExitCode::from(EXIT_ERROR);
    }

    println!("Saved: {}", output_path.display());

    // Print warnings to stderr (in lenient mode)
    for warning in all_warnings.iter() {
        eprintln!("Warning: {}", warning);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Render a composition to an image buffer
#[allow(clippy::too_many_arguments)]
fn render_composition_to_image(
    comp: &Composition,
    sprites: &std::collections::HashMap<String, Sprite>,
    registry: &PaletteRegistry,
    input_dir: &std::path::Path,
    include_visited: &mut HashSet<PathBuf>,
    all_warnings: &mut Vec<String>,
    strict: bool,
) -> Result<image::RgbaImage, ExitCode> {
    use image::RgbaImage;

    // Collect all sprite names referenced by the composition
    let mut required_sprites: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Add base sprite if specified
    if let Some(ref base_name) = comp.base {
        required_sprites.insert(base_name.clone());
    }

    // Add sprites from the sprites map
    for sprite_name in comp.sprites.values().flatten() {
        required_sprites.insert(sprite_name.clone());
    }

    // Render all required sprites
    let mut rendered_sprites: std::collections::HashMap<String, RgbaImage> =
        std::collections::HashMap::new();

    for sprite_name in &required_sprites {
        let sprite = match sprites.get(sprite_name) {
            Some(s) => s,
            None => {
                let warning_msg = format!(
                    "composition '{}': sprite '{}' not found",
                    comp.name, sprite_name
                );
                if strict {
                    eprintln!("Error: {}", warning_msg);
                    return Err(ExitCode::from(EXIT_ERROR));
                }
                all_warnings.push(warning_msg);
                continue;
            }
        };

        // Resolve palette for the sprite
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
                            return Err(ExitCode::from(EXIT_ERROR));
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
                            for w in all_warnings.iter() {
                                eprintln!("Error: {}", w);
                            }
                            return Err(ExitCode::from(EXIT_ERROR));
                        }
                    }
                    result.palette
                }
                Err(e) => {
                    eprintln!("Error: sprite '{}': {}", sprite.name, e);
                    return Err(ExitCode::from(EXIT_ERROR));
                }
            },
        };

        // Render the sprite
        let (image, render_warnings) = render_sprite(sprite, &resolved.colors);

        // Collect render warnings
        for warning in render_warnings {
            all_warnings.push(format!("sprite '{}': {}", sprite.name, warning.message));
        }

        if strict && !all_warnings.is_empty() {
            for w in all_warnings.iter() {
                eprintln!("Error: {}", w);
            }
            return Err(ExitCode::from(EXIT_ERROR));
        }

        rendered_sprites.insert(sprite_name.clone(), image);
    }

    // Render the composition
    let result = render_composition(comp, &rendered_sprites, strict);

    match result {
        Ok((image, comp_warnings)) => {
            // Collect composition warnings
            for warning in comp_warnings {
                all_warnings.push(format!("composition '{}': {}", comp.name, warning.message));
            }
            Ok(image)
        }
        Err(e) => {
            eprintln!("Error: composition '{}': {}", comp.name, e);
            Err(ExitCode::from(EXIT_ERROR))
        }
    }
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
        eprintln!(
            "Error: No valid frames to render in animation '{}'",
            animation.name
        );
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
            eprintln!(
                "Error: Failed to save GIF '{}': {}",
                output_path.display(),
                e
            );
            return ExitCode::from(EXIT_ERROR);
        }
    } else {
        // Spritesheet output
        let sheet = render_spritesheet(&frame_images, None);
        if let Err(e) = save_png(&sheet, &output_path) {
            eprintln!(
                "Error: Failed to save spritesheet '{}': {}",
                output_path.display(),
                e
            );
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

/// Execute the import command
fn run_import(
    input: &PathBuf,
    output: Option<&std::path::Path>,
    max_colors: usize,
    sprite_name: Option<&str>,
) -> ExitCode {
    // Validate max_colors
    if !(2..=256).contains(&max_colors) {
        eprintln!("Error: --max-colors must be between 2 and 256");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Derive sprite name from filename if not provided
    let name = sprite_name.map(String::from).unwrap_or_else(|| {
        input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    // Import the PNG
    let result = match import_png(input, &name, max_colors) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Generate output path
    let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        input
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}.jsonl", stem))
    });

    // Write JSONL output
    let jsonl = result.to_jsonl();
    if let Err(e) = std::fs::write(&output_path, jsonl) {
        eprintln!("Error: Failed to write '{}': {}", output_path.display(), e);
        return ExitCode::from(EXIT_ERROR);
    }

    println!(
        "Imported: {} ({}x{}, {} colors)",
        output_path.display(),
        result.width,
        result.height,
        result.palette.len()
    );
    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the analyze command
fn run_analyze(
    files: &[PathBuf],
    dir: Option<&std::path::Path>,
    recursive: bool,
    format: &str,
    output: Option<&std::path::Path>,
) -> ExitCode {
    // Validate format
    if format != "text" && format != "json" {
        eprintln!("Error: --format must be 'text' or 'json'");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Collect files to analyze
    let file_list = match collect_files(files, dir, recursive) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    if file_list.is_empty() {
        eprintln!("Error: No files to analyze");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Run analysis with progress indication
    let mut report = AnalysisReport::new();
    let total_files = file_list.len();
    let show_progress = total_files > 1 && output.is_some();

    for (i, path) in file_list.iter().enumerate() {
        if show_progress {
            eprint!("\rAnalyzing file {}/{}: {}", i + 1, total_files, path.display());
        }
        if let Err(e) = report.analyze_file(path) {
            report.files_failed += 1;
            report.failed_files.push((path.clone(), e));
        }
    }
    if show_progress {
        eprintln!(); // Clear progress line
    }

    // Format output
    let output_text = if format == "json" {
        // JSON output - basic structure for now
        serde_json::json!({
            "files_analyzed": report.files_analyzed,
            "files_failed": report.files_failed,
            "total_sprites": report.total_sprites,
            "total_palettes": report.total_palettes,
            "total_compositions": report.total_compositions,
            "total_animations": report.total_animations,
            "total_variants": report.total_variants,
            "unique_tokens": report.token_counter.unique_count(),
            "total_token_occurrences": report.token_counter.total(),
            "top_tokens": report.token_counter.top_n(10).iter().map(|(t, c)| {
                serde_json::json!({
                    "token": t,
                    "count": c,
                    "percentage": report.token_counter.percentage(t)
                })
            }).collect::<Vec<_>>(),
            "avg_palette_size": report.avg_palette_size(),
        })
        .to_string()
    } else {
        format_report_text(&report)
    };

    // Write output
    if let Some(output_path) = output {
        if let Err(e) = std::fs::write(output_path, &output_text) {
            eprintln!("Error: Failed to write '{}': {}", output_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }
        println!("Report written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the fmt command
fn run_fmt(files: &[PathBuf], check: bool, stdout_mode: bool) -> ExitCode {
    let mut needs_formatting = false;

    for file in files {
        // Read file content
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Cannot read '{}': {}", file.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        };

        // Format the content
        let formatted = match format_pixelsrc(&content) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: Cannot format '{}': {}", file.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        };

        if check {
            // Check mode: compare and report
            if content != formatted {
                eprintln!("{}: needs formatting", file.display());
                needs_formatting = true;
            }
        } else if stdout_mode {
            // Stdout mode: print formatted content
            print!("{}", formatted);
        } else {
            // In-place mode: write back to file
            if content != formatted {
                if let Err(e) = std::fs::write(file, &formatted) {
                    eprintln!("Error: Cannot write '{}': {}", file.display(), e);
                    return ExitCode::from(EXIT_ERROR);
                }
                eprintln!("{}: formatted", file.display());
            } else {
                eprintln!("{}: already formatted", file.display());
            }
        }
    }

    if check && needs_formatting {
        ExitCode::from(EXIT_ERROR)
    } else {
        ExitCode::from(EXIT_SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_pixelsrc_file_pxl() {
        assert!(is_pixelsrc_file(Path::new("test.pxl")));
        assert!(is_pixelsrc_file(Path::new("path/to/sprite.pxl")));
        assert!(is_pixelsrc_file(Path::new("/absolute/path.pxl")));
    }

    #[test]
    fn test_is_pixelsrc_file_jsonl() {
        assert!(is_pixelsrc_file(Path::new("test.jsonl")));
        assert!(is_pixelsrc_file(Path::new("path/to/sprite.jsonl")));
        assert!(is_pixelsrc_file(Path::new("/absolute/path.jsonl")));
    }

    #[test]
    fn test_is_pixelsrc_file_invalid() {
        assert!(!is_pixelsrc_file(Path::new("test.png")));
        assert!(!is_pixelsrc_file(Path::new("test.json")));
        assert!(!is_pixelsrc_file(Path::new("test.txt")));
        assert!(!is_pixelsrc_file(Path::new("test")));
        assert!(!is_pixelsrc_file(Path::new("pxl")));
        assert!(!is_pixelsrc_file(Path::new(".pxl")));
    }

    #[test]
    fn test_find_pixelsrc_files() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create test files
        fs::write(dir_path.join("test1.pxl"), "{}").unwrap();
        fs::write(dir_path.join("test2.jsonl"), "{}").unwrap();
        fs::write(dir_path.join("test3.png"), "ignored").unwrap();

        // Create a subdirectory with more files
        let sub_dir = dir_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("nested.pxl"), "{}").unwrap();

        let files = find_pixelsrc_files(dir_path);

        // Should find 3 pixelsrc files (.pxl and .jsonl)
        assert_eq!(files.len(), 3);

        // Check that all found files have correct extensions
        for file in &files {
            assert!(is_pixelsrc_file(file));
        }
    }
}
