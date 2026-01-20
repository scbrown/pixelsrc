//! Command-line interface implementation

use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::alias::{parse_simple_grid, simple_grid_to_sprite};
use crate::analyze::{collect_files, format_report_text, AnalysisReport};
use crate::atlas::{add_animation_to_atlas, pack_atlas, AtlasBox, AtlasConfig, SpriteInput};
use crate::composition::render_composition;
use crate::diff::{diff_files, format_diff};
#[allow(unused_imports)]
use crate::emoji::render_emoji_art;
use crate::explain::{explain_object, format_explanation, resolve_palette_colors, Explanation};
use crate::fmt::format_pixelsrc;
use crate::prime::{get_primer, list_sections, PrimerSection};
use crate::suggest::{format_suggestion, suggest, Suggester, SuggestionFix, SuggestionType};
use crate::terminal::{render_ansi_grid, render_coordinate_grid};
use crate::validate::{Severity, Validator};
use glob::glob;

/// Check if a path has a valid Pixelsrc file extension (.pxl or .jsonl).
pub fn is_pixelsrc_file(path: &std::path::Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("pxl") | Some("jsonl"))
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
use crate::lsp_agent_client::LspAgentClient;
use crate::models::{Animation, Composition, PaletteRef, Sprite, TtpObject};
use crate::output::{generate_output_path, save_png, scale_image};
use crate::palette_cycle::{generate_cycle_frames, get_cycle_duration};
use crate::palettes;
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, PaletteSource, ResolvedPalette, SpriteRegistry};
use crate::renderer::{render_resolved, render_sprite};
use crate::spritesheet::render_spritesheet;
use crate::transforms::{
    apply_crop, apply_mirror_horizontal, apply_mirror_vertical, apply_outline, apply_pad,
    apply_rotate, apply_shadow, apply_shift, apply_tile,
};

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

        /// Output format: atlas, atlas-aseprite, atlas-godot, atlas-unity, atlas-libgdx
        #[arg(long)]
        format: Option<String>,

        /// Maximum atlas size (e.g., "512x512", "1024x1024")
        #[arg(long)]
        max_size: Option<String>,

        /// Padding between sprites in atlas (pixels)
        #[arg(long, default_value = "0")]
        padding: u32,

        /// Force power-of-two dimensions for atlas
        #[arg(long)]
        power_of_two: bool,
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

    /// Validate pixelsrc files for common mistakes
    Validate {
        /// Files to validate (omit if using --stdin)
        #[arg(required_unless_present = "stdin")]
        files: Vec<PathBuf>,

        /// Read input from stdin
        #[arg(long)]
        stdin: bool,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Verify pixelsrc content for AI agents (returns JSON)
    ///
    /// Provides a structured verification API for AI agents with additional
    /// features like CSS color resolution and timing function analysis.
    #[command(name = "agent-verify")]
    AgentVerify {
        /// Read input from stdin (default behavior)
        #[arg(long)]
        stdin: bool,

        /// Content to verify (alternative to stdin)
        #[arg(long)]
        content: Option<String>,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,

        /// Include grid coordinate info for each sprite
        #[arg(long)]
        grid_info: bool,

        /// Include token suggestions for completion
        #[arg(long)]
        suggest_tokens: bool,

        /// Resolve CSS variables and color-mix() to computed hex values
        #[arg(long)]
        resolve_colors: bool,

        /// Analyze timing functions in animations
        #[arg(long)]
        analyze_timing: bool,
    },

    /// Explain sprites and other objects in human-readable format
    Explain {
        /// Input file containing pixelsrc objects
        input: PathBuf,

        /// Name of specific object to explain (sprite, palette, etc.)
        #[arg(short, long)]
        name: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Compare sprites semantically between two files
    Diff {
        /// First file to compare
        file_a: PathBuf,

        /// Second file to compare
        file_b: PathBuf,

        /// Compare only a specific sprite by name
        #[arg(long)]
        sprite: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Suggest fixes for pixelsrc files (missing tokens, row completion)
    Suggest {
        /// Files to analyze (omit if using --stdin)
        #[arg(required_unless_present = "stdin")]
        files: Vec<PathBuf>,

        /// Read input from stdin
        #[arg(long)]
        stdin: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Only show a specific type of suggestion (token, row)
        #[arg(long)]
        only: Option<String>,
    },

    /// Expand grid with column-aligned spacing for readability
    Inline {
        /// Input file containing sprite definitions
        input: PathBuf,

        /// Sprite name (if file contains multiple)
        #[arg(long)]
        sprite: Option<String>,
    },

    /// Extract repeated patterns into single-letter aliases (outputs JSON)
    Alias {
        /// Input file containing sprite definitions
        input: PathBuf,

        /// Sprite name (if file contains multiple)
        #[arg(long)]
        sprite: Option<String>,
    },

    /// Display grid with row/column coordinates for easy reference
    Grid {
        /// Input file containing palette and sprite definitions
        input: PathBuf,

        /// Sprite name (if file contains multiple sprites)
        #[arg(long)]
        sprite: Option<String>,

        /// Show full token names instead of abbreviations
        #[arg(long)]
        full: bool,
    },

    /// Display sprite with colored terminal output (ANSI true-color)
    Show {
        /// Input file containing sprite definitions
        file: PathBuf,

        /// Sprite name (if file contains multiple sprites)
        #[arg(long)]
        sprite: Option<String>,

        /// Animation name to show with onion skinning
        #[arg(long)]
        animation: Option<String>,

        /// Frame index to display (for animations)
        #[arg(long, default_value = "0")]
        frame: usize,

        /// Number of frames before/after to show as onion skin
        #[arg(long)]
        onion: Option<u32>,

        /// Ghost frame opacity (0.0-1.0, default: 0.3)
        #[arg(long, default_value = "0.3")]
        onion_opacity: f32,

        /// Tint color for previous frames (default: #0000FF blue)
        #[arg(long, default_value = "#0000FF")]
        onion_prev_color: String,

        /// Tint color for next frames (default: #00FF00 green)
        #[arg(long, default_value = "#00FF00")]
        onion_next_color: String,

        /// Decrease opacity for frames farther from current
        #[arg(long)]
        onion_fade: bool,

        /// Output file (PNG) for onion skin preview
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build all assets according to pxl.toml
    Build {
        /// Override output directory
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Override source directory
        #[arg(long)]
        src: Option<PathBuf>,

        /// Watch for changes and rebuild automatically
        #[arg(short, long)]
        watch: bool,

        /// Dry run (show what would be built without building)
        #[arg(long)]
        dry_run: bool,

        /// Force rebuild all targets (ignore cache)
        #[arg(short, long)]
        force: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Create a new asset from template
    New {
        /// Asset type: sprite, animation, palette
        asset_type: String,

        /// Asset name
        name: String,

        /// Palette to use (for sprites and animations)
        #[arg(long)]
        palette: Option<String>,
    },

    /// Initialize a new pixelsrc project
    Init {
        /// Project directory (default: current directory)
        path: Option<PathBuf>,

        /// Project name (default: directory name)
        #[arg(long)]
        name: Option<String>,

        /// Preset template: minimal, artist, animator, game
        #[arg(long, default_value = "minimal")]
        preset: String,
    },

    /// Create sprite from simple text grid (space-separated characters)
    ///
    /// Input format: each line is a row, characters separated by spaces.
    /// Use _ for transparent pixels. Example:
    ///   _ _ b b
    ///   _ b c b
    ///   b c c b
    Sketch {
        /// Input file (omit to read from stdin)
        #[arg()]
        file: Option<PathBuf>,

        /// Sprite name (default: "sketch")
        #[arg(short, long, default_value = "sketch")]
        name: String,

        /// Reference a named palette instead of inline placeholder colors
        #[arg(short, long)]
        palette: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Transform sprites (mirror, rotate, tile, etc.)
    ///
    /// Applies transforms to sprite grids and outputs new source files.
    /// Transforms are applied in the order specified.
    Transform {
        /// Input file containing sprite definitions
        input: PathBuf,

        /// Mirror axis (horizontal, vertical, both)
        #[arg(long)]
        mirror: Option<String>,

        /// Rotate degrees (90, 180, 270)
        #[arg(long)]
        rotate: Option<u16>,

        /// Tile pattern (e.g., "2x2", "3x1")
        #[arg(long)]
        tile: Option<String>,

        /// Padding pixels
        #[arg(long)]
        pad: Option<u32>,

        /// Add outline (optionally specify token, e.g., "{border}" or "#000")
        #[arg(long)]
        outline: Option<Option<String>>,

        /// Outline width (default: 1)
        #[arg(long, default_value = "1")]
        outline_width: u32,

        /// Crop region (X,Y,W,H)
        #[arg(long)]
        crop: Option<String>,

        /// Shift pixels (X,Y) - circular shift with wrap around
        #[arg(long)]
        shift: Option<String>,

        /// Shadow offset (X,Y) - add drop shadow
        #[arg(long)]
        shadow: Option<String>,

        /// Shadow token (default: {shadow})
        #[arg(long)]
        shadow_token: Option<String>,

        /// Target sprite name (if file contains multiple)
        #[arg(long)]
        sprite: Option<String>,

        /// Output file (required)
        #[arg(short, long)]
        output: PathBuf,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Allow large output (disable expansion warnings)
        #[arg(long)]
        allow_large: bool,
    },

    /// Start the Language Server Protocol server (for editor integration)
    #[command(hide = true)]
    Lsp,
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
            format,
            max_size,
            padding,
            power_of_two,
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
            format.as_deref(),
            max_size.as_deref(),
            padding,
            power_of_two,
        ),
        Commands::Import { input, output, max_colors, name } => {
            run_import(&input, output.as_deref(), max_colors, name.as_deref())
        }
        Commands::Prompts { template } => run_prompts(template.as_deref()),
        Commands::Palettes { action } => run_palettes(action),
        Commands::Analyze { files, dir, recursive, format, output } => {
            run_analyze(&files, dir.as_deref(), recursive, &format, output.as_deref())
        }
        Commands::Fmt { files, check, stdout } => run_fmt(&files, check, stdout),
        Commands::Prime { brief, section } => run_prime(brief, section.as_deref()),
        Commands::Validate { files, stdin, strict, json } => {
            run_validate(&files, stdin, strict, json)
        }
        Commands::AgentVerify {
            stdin,
            content,
            strict,
            grid_info,
            suggest_tokens,
            resolve_colors,
            analyze_timing,
        } => run_agent_verify(
            stdin,
            content.as_deref(),
            strict,
            grid_info,
            suggest_tokens,
            resolve_colors,
            analyze_timing,
        ),
        Commands::Explain { input, name, json } => run_explain(&input, name.as_deref(), json),
        Commands::Diff { file_a, file_b, sprite, json } => {
            run_diff(&file_a, &file_b, sprite.as_deref(), json)
        }
        Commands::Suggest { files, stdin, json, only } => {
            run_suggest(&files, stdin, json, only.as_deref())
        }
        Commands::Inline { input, sprite } => run_inline(&input, sprite.as_deref()),
        Commands::Alias { input, sprite } => run_alias(&input, sprite.as_deref()),
        Commands::Grid { input, sprite, full } => run_grid(&input, sprite.as_deref(), full),
        Commands::Show {
            file,
            sprite,
            animation,
            frame,
            onion,
            onion_opacity,
            onion_prev_color,
            onion_next_color,
            onion_fade,
            output,
        } => run_show(
            &file,
            sprite.as_deref(),
            animation.as_deref(),
            frame,
            onion,
            onion_opacity,
            &onion_prev_color,
            &onion_next_color,
            onion_fade,
            output.as_deref(),
        ),
        Commands::Build { out, src, watch, dry_run, force, verbose } => {
            run_build(out.as_deref(), src.as_deref(), watch, dry_run, force, verbose)
        }
        Commands::New { asset_type, name, palette } => {
            run_new(&asset_type, &name, palette.as_deref())
        }
        Commands::Init { path, name, preset } => {
            run_init(path.as_deref(), name.as_deref(), &preset)
        }
        Commands::Sketch { file, name, palette, output } => {
            run_sketch(file.as_deref(), &name, palette.as_deref(), output.as_deref())
        }
        Commands::Transform {
            input,
            mirror,
            rotate,
            tile,
            pad,
            outline,
            outline_width,
            crop,
            shift,
            shadow,
            shadow_token,
            sprite,
            output,
            stdin,
            allow_large,
        } => run_transform(
            &input,
            mirror.as_deref(),
            rotate,
            tile.as_deref(),
            pad,
            outline,
            outline_width,
            crop.as_deref(),
            shift.as_deref(),
            shadow.as_deref(),
            shadow_token.as_deref(),
            sprite.as_deref(),
            &output,
            stdin,
            allow_large,
        ),
        #[cfg(not(target_arch = "wasm32"))]
        Commands::Lsp => run_lsp(),
        #[cfg(target_arch = "wasm32")]
        Commands::Lsp => {
            eprintln!("Error: LSP is not available in WASM builds");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

/// Execute the LSP server command
#[cfg(not(target_arch = "wasm32"))]
fn run_lsp() -> ExitCode {
    use tokio::runtime::Runtime;

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Error: Failed to create async runtime: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    rt.block_on(crate::lsp::run_server());
    ExitCode::from(EXIT_SUCCESS)
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
            let template_names: Vec<&str> = TEMPLATES.iter().map(|(n, _)| *n).collect();
            if let Some(suggestion) = format_suggestion(&suggest(name, &template_names, 3)) {
                eprintln!("{}", suggestion);
            }
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
                    let builtin_names = palettes::list_builtins();
                    if let Some(suggestion) =
                        format_suggestion(&suggest(palette_name, &builtin_names, 3))
                    {
                        eprintln!("{}", suggestion);
                    }
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
    format: Option<&str>,
    max_size_arg: Option<&str>,
    padding: u32,
    power_of_two: bool,
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

    // Build palette registry and sprite registry, and collect sprites, animations, and compositions
    let mut registry = PaletteRegistry::new();
    let mut sprite_registry = SpriteRegistry::new();
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
                sprite_registry.register_sprite(sprite.clone());
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
            TtpObject::Variant(variant) => {
                // Register variant with sprite registry for transform resolution
                sprite_registry.register_variant(variant);
            }
            TtpObject::Particle(_) => {
                // Particle systems are runtime constructs, not rendered statically
            }
            TtpObject::Transform(_) => {
                // User-defined transforms are stored in transform registry
                // (future: register in transform_registry)
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
            &sprite_registry,
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

    // Handle atlas format rendering (--format atlas)
    if let Some(fmt) = format {
        if fmt.starts_with("atlas") {
            return run_atlas_render(
                input,
                output,
                &sprites_by_name,
                &animations_by_name,
                &sprite_registry,
                &registry,
                input_dir,
                &mut include_visited,
                &mut all_warnings,
                strict,
                scale,
                fmt,
                max_size_arg,
                padding,
                power_of_two,
            );
        } else {
            eprintln!("Error: Unknown format '{}'. Supported: atlas, atlas-aseprite, atlas-godot, atlas-unity, atlas-libgdx", fmt);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    }

    // Handle composition rendering if --composition is provided
    if let Some(comp_name) = composition_filter {
        return run_composition_render(
            input,
            output,
            comp_name,
            &compositions_by_name,
            &sprites_by_name,
            &sprite_registry,
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
            let sprite_names: Vec<&str> = sprites_by_name.keys().map(|s| s.as_str()).collect();
            if let Some(suggestion) = format_suggestion(&suggest(name, &sprite_names, 3)) {
                eprintln!("{}", suggestion);
            }
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
            // TRF-9: Use sprite registry to resolve transforms
            // Check if sprite uses @include: palette (needs special handling)
            let uses_include_palette =
                matches!(&sprite.palette, PaletteRef::Named(name) if is_include_ref(name));

            // For @include: palettes, resolve palette first, then apply transforms
            // For normal palettes, use sprite_registry.resolve() which handles both
            let (final_grid, final_palette) = if uses_include_palette {
                // Handle @include: palette specially
                let include_path = if let PaletteRef::Named(name) = &sprite.palette {
                    extract_include_path(name).unwrap()
                } else {
                    unreachable!()
                };

                let palette_colors = match resolve_include_with_detection(
                    include_path,
                    input_dir,
                    &mut include_visited,
                ) {
                    Ok(palette) => palette.colors,
                    Err(e) => {
                        if strict {
                            eprintln!("Error: sprite '{}': {}", sprite.name, e);
                            return ExitCode::from(EXIT_ERROR);
                        }
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, e));
                        std::collections::HashMap::new()
                    }
                };

                // Still use sprite registry to get transformed grid
                let resolved = match sprite_registry.resolve(&sprite.name, &registry, strict) {
                    Ok(r) => r,
                    Err(e) => {
                        if strict {
                            eprintln!("Error: sprite '{}': {}", sprite.name, e);
                            return ExitCode::from(EXIT_ERROR);
                        }
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, e));
                        continue;
                    }
                };
                // Use transformed grid but with include-resolved palette
                (resolved.grid, palette_colors)
            } else {
                // Normal path: sprite_registry handles both transforms and palette
                let resolved = match sprite_registry.resolve(&sprite.name, &registry, strict) {
                    Ok(r) => {
                        for warning in &r.warnings {
                            all_warnings
                                .push(format!("sprite '{}': {}", sprite.name, warning.message));
                        }
                        r
                    }
                    Err(e) => {
                        if strict {
                            eprintln!("Error: sprite '{}': {}", sprite.name, e);
                            return ExitCode::from(EXIT_ERROR);
                        }
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, e));
                        continue;
                    }
                };
                (resolved.grid, resolved.palette)
            };

            // Create resolved sprite for rendering
            let render_sprite_data = crate::registry::ResolvedSprite {
                name: sprite.name.clone(),
                size: sprite.size,
                grid: final_grid,
                palette: final_palette,
                warnings: vec![],
            };

            // Render the resolved sprite (transforms already applied)
            let (image, render_warnings) = render_resolved(&render_sprite_data);

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
            // Render the composition with sprite registry for transform support (TRF-9)
            let result = render_composition_to_image(
                comp,
                &sprites_by_name,
                &sprite_registry,
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
/// TRF-9: Now uses SpriteRegistry for transform support
#[allow(clippy::too_many_arguments)]
fn run_composition_render(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    comp_name: &str,
    compositions: &std::collections::HashMap<String, Composition>,
    sprites: &std::collections::HashMap<String, Sprite>,
    sprite_registry: &SpriteRegistry,
    palette_registry: &PaletteRegistry,
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
            let comp_names: Vec<&str> = compositions.keys().map(|s| s.as_str()).collect();
            if let Some(suggestion) = format_suggestion(&suggest(comp_name, &comp_names, 3)) {
                eprintln!("{}", suggestion);
            }
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Render the composition with sprite registry for transform support
    let result = render_composition_to_image(
        comp,
        sprites,
        sprite_registry,
        palette_registry,
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
/// TRF-9: Now uses SpriteRegistry to resolve sprites with transforms applied
#[allow(clippy::too_many_arguments)]
fn render_composition_to_image(
    comp: &Composition,
    sprites: &std::collections::HashMap<String, Sprite>,
    sprite_registry: &SpriteRegistry,
    palette_registry: &PaletteRegistry,
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

    // Render all required sprites using sprite registry for transform resolution (TRF-9)
    let mut rendered_sprites: std::collections::HashMap<String, RgbaImage> =
        std::collections::HashMap::new();

    for sprite_name in &required_sprites {
        // First check if sprite exists in the raw sprites map (for @include: handling)
        let original_sprite = sprites.get(sprite_name);

        // Use sprite registry to resolve the sprite with transforms applied
        let resolved_sprite = match sprite_registry.resolve(sprite_name, palette_registry, strict) {
            Ok(resolved) => {
                // Collect any sprite warnings
                for warning in &resolved.warnings {
                    all_warnings.push(format!("sprite '{}': {}", sprite_name, warning.message));
                }
                resolved
            }
            Err(e) => {
                let warning_msg = format!(
                    "composition '{}': sprite '{}' resolution failed: {}",
                    comp.name, sprite_name, e
                );
                if strict {
                    eprintln!("Error: {}", warning_msg);
                    return Err(ExitCode::from(EXIT_ERROR));
                }
                all_warnings.push(warning_msg);
                continue;
            }
        };

        // Check if we need to handle @include: syntax for palette
        let final_palette = if resolved_sprite.palette.is_empty() {
            if let Some(sprite) = original_sprite {
                if let PaletteRef::Named(name) = &sprite.palette {
                    if is_include_ref(name) {
                        let include_path = extract_include_path(name).unwrap();
                        match resolve_include_with_detection(
                            include_path,
                            input_dir,
                            include_visited,
                        ) {
                            Ok(palette) => palette.colors,
                            Err(e) => {
                                if strict {
                                    eprintln!("Error: sprite '{}': {}", sprite_name, e);
                                    return Err(ExitCode::from(EXIT_ERROR));
                                }
                                all_warnings.push(format!("sprite '{}': {}", sprite_name, e));
                                std::collections::HashMap::new()
                            }
                        }
                    } else {
                        resolved_sprite.palette.clone()
                    }
                } else {
                    resolved_sprite.palette.clone()
                }
            } else {
                resolved_sprite.palette.clone()
            }
        } else {
            resolved_sprite.palette.clone()
        };

        // Create resolved sprite with final palette for rendering
        let render_sprite_data = crate::registry::ResolvedSprite {
            name: resolved_sprite.name.clone(),
            size: resolved_sprite.size,
            grid: resolved_sprite.grid.clone(),
            palette: final_palette,
            warnings: vec![],
        };

        // Render the resolved sprite (transforms already applied)
        let (image, render_warnings) = render_resolved(&render_sprite_data);

        // Collect render warnings
        for warning in render_warnings {
            all_warnings.push(format!("sprite '{}': {}", sprite_name, warning.message));
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
    // TODO(CSS-9): Pass variable registry when available from palette parsing
    let result = render_composition(comp, &rendered_sprites, strict, None);

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
/// TRF-9: Now uses SpriteRegistry for transform support
#[allow(clippy::too_many_arguments)]
fn run_animation_render(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    animations: &std::collections::HashMap<String, Animation>,
    sprites: &std::collections::HashMap<String, Sprite>,
    _sprite_registry: &SpriteRegistry,
    palette_registry: &PaletteRegistry,
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
                let anim_names: Vec<&str> = animations.keys().map(|s| s.as_str()).collect();
                if let Some(suggestion) = format_suggestion(&suggest(name, &anim_names, 3)) {
                    eprintln!("{}", suggestion);
                }
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

    // Check if this is a palette-cycle animation
    // Palette cycling is used when animation has palette_cycle defined
    let (frame_images, frame_duration) = if animation.has_palette_cycle()
        && animation.frames.len() == 1
    {
        // Palette cycle mode: generate frames by rotating colors
        let frame_name = &animation.frames[0];
        let sprite = match sprites.get(frame_name) {
            Some(s) => s,
            None => {
                eprintln!(
                    "Error: Animation '{}' references missing sprite '{}'",
                    animation.name, frame_name
                );
                return ExitCode::from(EXIT_ERROR);
            }
        };

        // Resolve base palette
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
            _ => match palette_registry.resolve(sprite, strict) {
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

        // Generate palette-cycled frames
        let (frames, cycle_warnings) = generate_cycle_frames(sprite, &resolved.colors, animation);

        // Collect warnings
        for warning in cycle_warnings {
            all_warnings.push(format!("sprite '{}': {}", sprite.name, warning));
        }

        if strict && !all_warnings.is_empty() {
            for warning in all_warnings.iter() {
                eprintln!("Error: {}", warning);
            }
            return ExitCode::from(EXIT_ERROR);
        }

        // Apply scaling to all frames
        let scaled_frames: Vec<_> = frames.into_iter().map(|f| scale_image(f, scale)).collect();

        // Use cycle duration for GIF timing
        let duration = get_cycle_duration(animation);

        (scaled_frames, duration)
    } else {
        // Traditional frame-based animation
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
                _ => match palette_registry.resolve(sprite, strict) {
                    Ok(result) => {
                        if let Some(warning) = result.warning {
                            all_warnings
                                .push(format!("sprite '{}': {}", sprite.name, warning.message));
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

        (frame_images, animation.duration_ms())
    };

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
        if let Err(e) = render_gif(&frame_images, frame_duration, animation.loops(), &output_path) {
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

/// Parse max-size argument (e.g., "512x512") into (width, height)
fn parse_max_size(arg: Option<&str>) -> Result<(u32, u32), String> {
    match arg {
        None => Ok((4096, 4096)), // Default
        Some(s) => {
            let parts: Vec<&str> = s.split('x').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid max-size format '{}'. Use WxH (e.g., 512x512)", s));
            }
            let w = parts[0].parse::<u32>().map_err(|_| format!("Invalid width in '{}'", s))?;
            let h = parts[1].parse::<u32>().map_err(|_| format!("Invalid height in '{}'", s))?;
            if w == 0 || h == 0 {
                return Err("Width and height must be greater than 0".to_string());
            }
            Ok((w, h))
        }
    }
}

/// Execute atlas rendering
#[allow(clippy::too_many_arguments)]
fn run_atlas_render(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    sprites: &std::collections::HashMap<String, Sprite>,
    animations: &std::collections::HashMap<String, Animation>,
    _sprite_registry: &SpriteRegistry,
    palette_registry: &PaletteRegistry,
    input_dir: &std::path::Path,
    include_visited: &mut HashSet<PathBuf>,
    all_warnings: &mut Vec<String>,
    strict: bool,
    scale: u8,
    format: &str,
    max_size_arg: Option<&str>,
    padding: u32,
    power_of_two: bool,
) -> ExitCode {
    // Parse max-size
    let max_size = match parse_max_size(max_size_arg) {
        Ok(size) => size,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    // Configure atlas packing
    let config = AtlasConfig { max_size, padding, power_of_two };

    // Render all sprites to images
    let mut sprite_inputs: Vec<SpriteInput> = Vec::new();

    for sprite in sprites.values() {
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
                        continue;
                    }
                }
            }
            _ => match palette_registry.resolve(sprite, strict) {
                Ok(result) => {
                    if let Some(warning) = result.warning {
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, warning.message));
                        if strict {
                            for w in all_warnings.iter() {
                                eprintln!("Error: {}", w);
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
            for w in all_warnings.iter() {
                eprintln!("Error: {}", w);
            }
            return ExitCode::from(EXIT_ERROR);
        }

        // Extract metadata for atlas export
        let (origin, boxes) = if let Some(ref meta) = sprite.metadata {
            let origin = meta.origin;
            let boxes = meta.boxes.as_ref().map(|b| {
                b.iter()
                    .map(|(name, cb)| {
                        (name.clone(), AtlasBox { x: cb.x, y: cb.y, w: cb.w, h: cb.h })
                    })
                    .collect()
            });
            (origin, boxes)
        } else {
            (None, None)
        };

        sprite_inputs.push(SpriteInput { name: sprite.name.clone(), image, origin, boxes });
    }

    if sprite_inputs.is_empty() {
        eprintln!("Error: No sprites to pack into atlas");
        return ExitCode::from(EXIT_ERROR);
    }

    // Determine output base name
    let base_name = if let Some(out_path) = output {
        out_path.file_stem().and_then(|s| s.to_str()).unwrap_or("atlas").to_string()
    } else {
        input
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| format!("{}_atlas", s))
            .unwrap_or_else(|| "atlas".to_string())
    };

    let output_dir = output
        .and_then(|p| p.parent())
        .unwrap_or_else(|| input.parent().unwrap_or(std::path::Path::new(".")));

    // Pack sprites into atlas(es)
    let result = pack_atlas(&sprite_inputs, &config, &base_name);

    if result.atlases.is_empty() {
        eprintln!("Error: Failed to pack sprites into atlas");
        return ExitCode::from(EXIT_ERROR);
    }

    // Save each atlas
    for (image, mut metadata) in result.atlases {
        // Add animation metadata
        for anim in animations.values() {
            let fps = 1000 / anim.duration_ms().max(1);
            add_animation_to_atlas(&mut metadata, &anim.name, &anim.frames, fps);
        }

        // Determine file paths
        let image_path = output_dir.join(&metadata.image);
        let json_name = metadata.image.replace(".png", ".json");
        let json_path = output_dir.join(&json_name);

        // Save PNG
        if let Err(e) = save_png(&image, &image_path) {
            eprintln!("Error: Failed to save atlas '{}': {}", image_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }

        // Generate JSON based on format variant
        let json_content = match format {
            "atlas" => serde_json::to_string_pretty(&metadata).unwrap(),
            "atlas-aseprite" => generate_aseprite_json(&metadata),
            "atlas-godot" => generate_godot_json(&metadata),
            "atlas-unity" => generate_unity_json(&metadata),
            "atlas-libgdx" => generate_libgdx_atlas(&metadata),
            _ => serde_json::to_string_pretty(&metadata).unwrap(),
        };

        // Determine JSON file extension for libGDX
        let final_json_path = if format == "atlas-libgdx" {
            output_dir.join(metadata.image.replace(".png", ".atlas"))
        } else {
            json_path
        };

        // Save JSON/metadata
        if let Err(e) = std::fs::write(&final_json_path, &json_content) {
            eprintln!("Error: Failed to save metadata '{}': {}", final_json_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }

        println!("Saved: {} + {}", image_path.display(), final_json_path.display());
    }

    // Print warnings
    for warning in all_warnings.iter() {
        eprintln!("Warning: {}", warning);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Generate Aseprite-compatible JSON format
fn generate_aseprite_json(metadata: &crate::atlas::AtlasMetadata) -> String {
    let frames: serde_json::Map<String, serde_json::Value> = metadata
        .frames
        .iter()
        .map(|(name, frame)| {
            (
                format!("{}.png", name),
                serde_json::json!({
                    "frame": {"x": frame.x, "y": frame.y, "w": frame.w, "h": frame.h},
                    "rotated": false,
                    "trimmed": false,
                    "spriteSourceSize": {"x": 0, "y": 0, "w": frame.w, "h": frame.h},
                    "sourceSize": {"w": frame.w, "h": frame.h}
                }),
            )
        })
        .collect();

    let meta = serde_json::json!({
        "app": "pixelsrc",
        "version": "1.0",
        "image": metadata.image,
        "format": "RGBA8888",
        "size": {"w": metadata.size[0], "h": metadata.size[1]},
        "scale": "1"
    });

    serde_json::to_string_pretty(&serde_json::json!({
        "frames": frames,
        "meta": meta
    }))
    .unwrap()
}

/// Generate Godot-compatible JSON format
fn generate_godot_json(metadata: &crate::atlas::AtlasMetadata) -> String {
    let textures: Vec<serde_json::Value> = metadata
        .frames
        .iter()
        .map(|(name, frame)| {
            serde_json::json!({
                "name": name,
                "region": {"x": frame.x, "y": frame.y, "w": frame.w, "h": frame.h}
            })
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "textures": [{
            "image": metadata.image,
            "size": {"w": metadata.size[0], "h": metadata.size[1]},
            "sprites": textures
        }]
    }))
    .unwrap()
}

/// Generate Unity-compatible JSON format
fn generate_unity_json(metadata: &crate::atlas::AtlasMetadata) -> String {
    let sprites: Vec<serde_json::Value> = metadata
        .frames
        .iter()
        .map(|(name, frame)| {
            serde_json::json!({
                "name": name,
                "rect": {
                    "x": frame.x,
                    "y": metadata.size[1] - frame.y - frame.h, // Unity uses bottom-left origin
                    "width": frame.w,
                    "height": frame.h
                },
                "pivot": {"x": 0.5, "y": 0.5}
            })
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "texture": metadata.image,
        "textureSize": {"width": metadata.size[0], "height": metadata.size[1]},
        "sprites": sprites
    }))
    .unwrap()
}

/// Generate libGDX-compatible atlas format
fn generate_libgdx_atlas(metadata: &crate::atlas::AtlasMetadata) -> String {
    let mut lines = vec![
        metadata.image.clone(),
        format!("size: {},{}", metadata.size[0], metadata.size[1]),
        "format: RGBA8888".to_string(),
        "filter: Nearest,Nearest".to_string(),
        "repeat: none".to_string(),
    ];

    for (name, frame) in &metadata.frames {
        lines.push(name.clone());
        lines.push("  rotate: false".to_string());
        lines.push(format!("  xy: {}, {}", frame.x, frame.y));
        lines.push(format!("  size: {}, {}", frame.w, frame.h));
        lines.push(format!("  orig: {}, {}", frame.w, frame.h));
        lines.push("  offset: 0, 0".to_string());
        lines.push("  index: -1".to_string());
    }

    lines.join("\n")
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
    let name = sprite_name
        .map(String::from)
        .unwrap_or_else(|| input.file_stem().unwrap_or_default().to_string_lossy().to_string());

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
        input.parent().unwrap_or(std::path::Path::new(".")).join(format!("{}.jsonl", stem))
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
        // JSON output
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
            "co_occurrence": report.co_occurrence.top_n(10).iter().map(|((t1, t2), count)| {
                serde_json::json!({
                    "token1": t1,
                    "token2": t2,
                    "sprites": count
                })
            }).collect::<Vec<_>>(),
            "token_families": report.token_families().iter().take(10).map(|family| {
                serde_json::json!({
                    "prefix": family.prefix,
                    "tokens": family.tokens,
                    "total_count": family.total_count
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

/// Execute the validate command
fn run_validate(files: &[PathBuf], stdin: bool, strict: bool, json: bool) -> ExitCode {
    use std::io::{self, BufRead};

    let mut validator = Validator::new();

    if stdin {
        // Read from stdin
        let stdin_handle = io::stdin();
        for (line_idx, line_result) in stdin_handle.lock().lines().enumerate() {
            let line_number = line_idx + 1;
            match line_result {
                Ok(line) => validator.validate_line(line_number, &line),
                Err(e) => {
                    eprintln!("Error reading stdin at line {}: {}", line_number, e);
                    return ExitCode::from(EXIT_ERROR);
                }
            }
        }
    } else {
        // Validate files
        if files.is_empty() {
            eprintln!("Error: No files to validate");
            return ExitCode::from(EXIT_INVALID_ARGS);
        }

        for path in files {
            if !json {
                println!("Validating {}...", path.display());
            }
            if let Err(e) = validator.validate_file(path) {
                eprintln!("Error: Cannot read '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        }
    }

    let issues = validator.into_issues();
    let error_count = issues.iter().filter(|i| matches!(i.severity, Severity::Error)).count();
    let warning_count = issues.iter().filter(|i| matches!(i.severity, Severity::Warning)).count();

    // Determine validity based on strict mode
    let has_failures = error_count > 0 || (strict && warning_count > 0);

    if json {
        // JSON output
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .map(|i| {
                let mut obj = serde_json::json!({
                    "line": i.line,
                    "type": i.issue_type.to_string(),
                    "message": i.message,
                });
                if let Some(ref ctx) = i.context {
                    obj["context"] = serde_json::json!(ctx);
                }
                if let Some(ref sug) = i.suggestion {
                    obj["suggestion"] = serde_json::json!(sug);
                }
                obj
            })
            .collect();

        let warnings: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Warning))
            .map(|i| {
                let mut obj = serde_json::json!({
                    "line": i.line,
                    "type": i.issue_type.to_string(),
                    "message": i.message,
                });
                if let Some(ref ctx) = i.context {
                    obj["context"] = serde_json::json!(ctx);
                }
                if let Some(ref sug) = i.suggestion {
                    obj["suggestion"] = serde_json::json!(sug);
                }
                obj
            })
            .collect();

        let output = serde_json::json!({
            "valid": !has_failures,
            "errors": errors,
            "warnings": warnings,
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        // Text output
        if issues.is_empty() {
            println!();
            println!("No issues found.");
        } else {
            println!();
            for issue in &issues {
                let severity_str = match issue.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARNING",
                };

                let mut msg = format!("Line {}: {} - {}", issue.line, severity_str, issue.message);

                if let Some(ref ctx) = issue.context {
                    msg.push_str(&format!(" ({})", ctx));
                }
                if let Some(ref sug) = issue.suggestion {
                    msg.push_str(&format!(" ({})", sug));
                }

                eprintln!("{}", msg);
            }

            println!();
            match (error_count, warning_count) {
                (0, w) => println!("Found {} warning{}.", w, if w == 1 { "" } else { "s" }),
                (e, 0) => println!("Found {} error{}.", e, if e == 1 { "" } else { "s" }),
                (e, w) => println!(
                    "Found {} error{}, {} warning{}.",
                    e,
                    if e == 1 { "" } else { "s" },
                    w,
                    if w == 1 { "" } else { "s" }
                ),
            }

            if !strict && warning_count > 0 && error_count == 0 {
                println!("Hint: Run with --strict to treat warnings as errors.");
            }
        }
    }

    if has_failures {
        ExitCode::from(EXIT_ERROR)
    } else {
        ExitCode::from(EXIT_SUCCESS)
    }
}

/// Execute the agent-verify command
fn run_agent_verify(
    stdin: bool,
    content: Option<&str>,
    strict: bool,
    grid_info: bool,
    suggest_tokens: bool,
    resolve_colors_flag: bool,
    analyze_timing_flag: bool,
) -> ExitCode {
    use std::io::{self, Read};

    // Get content from --content arg or stdin
    let content_string: String = if let Some(c) = content {
        c.to_string()
    } else if stdin || content.is_none() {
        // Read from stdin by default
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            let error_json = serde_json::json!({
                "error": format!("Failed to read from stdin: {}", e)
            });
            println!("{}", serde_json::to_string_pretty(&error_json).unwrap());
            return ExitCode::from(EXIT_ERROR);
        }
        buffer
    } else {
        let error_json = serde_json::json!({
            "error": "No content provided. Use --content or provide input via stdin."
        });
        println!("{}", serde_json::to_string_pretty(&error_json).unwrap());
        return ExitCode::from(EXIT_INVALID_ARGS);
    };

    // Create client with appropriate strictness
    let client = if strict { LspAgentClient::strict() } else { LspAgentClient::new() };

    // Build the result object
    let mut result = serde_json::Map::new();

    // Always include verification result
    let verification = client.verify_content(&content_string);
    result.insert("valid".to_string(), serde_json::json!(verification.valid));
    result.insert("error_count".to_string(), serde_json::json!(verification.error_count));
    result.insert("warning_count".to_string(), serde_json::json!(verification.warning_count));

    // Convert errors to JSON
    let errors: Vec<serde_json::Value> = verification
        .errors
        .iter()
        .map(|d| {
            let mut obj = serde_json::json!({
                "line": d.line,
                "type": d.issue_type,
                "message": d.message,
            });
            if let Some(ref ctx) = d.context {
                obj["context"] = serde_json::json!(ctx);
            }
            if let Some(ref sug) = d.suggestion {
                obj["suggestion"] = serde_json::json!(sug);
            }
            obj
        })
        .collect();
    result.insert("errors".to_string(), serde_json::json!(errors));

    // Convert warnings to JSON
    let warnings: Vec<serde_json::Value> = verification
        .warnings
        .iter()
        .map(|d| {
            let mut obj = serde_json::json!({
                "line": d.line,
                "type": d.issue_type,
                "message": d.message,
            });
            if let Some(ref ctx) = d.context {
                obj["context"] = serde_json::json!(ctx);
            }
            if let Some(ref sug) = d.suggestion {
                obj["suggestion"] = serde_json::json!(sug);
            }
            obj
        })
        .collect();
    result.insert("warnings".to_string(), serde_json::json!(warnings));

    // Optional: grid info
    if grid_info {
        // Extract grid info from sprites
        let grid_info_vec: Vec<serde_json::Value> = content_string
            .lines()
            .filter_map(|line| {
                let obj: serde_json::Value = serde_json::from_str(line).ok()?;
                let obj = obj.as_object()?;
                if obj.get("type")?.as_str()? != "sprite" {
                    return None;
                }
                let name = obj.get("name")?.as_str()?;
                let grid = obj.get("grid")?.as_array()?;

                // Calculate row widths
                let row_widths: Vec<usize> = grid
                    .iter()
                    .filter_map(|row| {
                        let row_str = row.as_str()?;
                        let (tokens, _) = crate::tokenizer::tokenize(row_str);
                        Some(tokens.len())
                    })
                    .collect();

                // Get size from size field or first row
                let expected_width = if let Some(size) = obj.get("size").and_then(|s| s.as_array())
                {
                    size.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize
                } else {
                    row_widths.first().copied().unwrap_or(0)
                };

                let expected_height = if let Some(size) = obj.get("size").and_then(|s| s.as_array())
                {
                    size.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize
                } else {
                    row_widths.len()
                };

                let aligned = row_widths.iter().all(|&w| w == expected_width)
                    && row_widths.len() == expected_height;

                Some(serde_json::json!({
                    "name": name,
                    "size": [expected_width, expected_height],
                    "actual_rows": row_widths.len(),
                    "row_widths": row_widths,
                    "aligned": aligned,
                }))
            })
            .collect();

        result.insert("grid_info".to_string(), serde_json::json!(grid_info_vec));
    }

    // Optional: token suggestions
    if suggest_tokens {
        let completions = client.get_completions(&content_string, 1, 0);
        let tokens: Vec<serde_json::Value> = completions
            .items
            .iter()
            .map(|c| {
                let mut obj = serde_json::json!({
                    "token": c.label,
                });
                if let Some(ref detail) = c.detail {
                    obj["color"] = serde_json::json!(detail);
                }
                obj
            })
            .collect();
        result.insert("available_tokens".to_string(), serde_json::json!(tokens));
    }

    // Optional: resolve colors
    if resolve_colors_flag {
        let color_result = client.resolve_colors(&content_string);
        let resolved: Vec<serde_json::Value> = color_result
            .colors
            .iter()
            .map(|c| {
                serde_json::json!({
                    "token": c.token,
                    "original": c.original,
                    "resolved": c.resolved,
                    "palette": c.palette,
                })
            })
            .collect();
        result.insert("resolved_colors".to_string(), serde_json::json!(resolved));

        if !color_result.errors.is_empty() {
            result.insert(
                "color_resolution_errors".to_string(),
                serde_json::json!(color_result.errors),
            );
        }
    }

    // Optional: analyze timing
    if analyze_timing_flag {
        let timing_result = client.analyze_timing(&content_string);
        let analysis: Vec<serde_json::Value> = timing_result
            .animations
            .iter()
            .map(|t| {
                let mut obj = serde_json::json!({
                    "animation": t.animation,
                    "timing_function": t.timing_function,
                    "description": t.description,
                    "curve_type": t.curve_type,
                });
                if let Some(ref curve) = t.ascii_curve {
                    obj["ascii_curve"] = serde_json::json!(curve);
                }
                obj
            })
            .collect();
        result.insert("timing_analysis".to_string(), serde_json::json!(analysis));
    }

    // Output JSON result
    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(result)).unwrap());

    if verification.valid {
        ExitCode::from(EXIT_SUCCESS)
    } else {
        ExitCode::from(EXIT_ERROR)
    }
}

/// Execute the explain command
fn run_explain(input: &PathBuf, name_filter: Option<&str>, json: bool) -> ExitCode {
    use std::collections::HashMap;

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

    if parse_result.objects.is_empty() {
        eprintln!("Error: No objects found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    // Collect palettes for color resolution
    let mut known_palettes: HashMap<String, HashMap<String, String>> = HashMap::new();
    for obj in &parse_result.objects {
        if let TtpObject::Palette(palette) = obj {
            known_palettes.insert(palette.name.clone(), palette.colors.clone());
        }
    }

    // Filter objects if name is specified
    let objects_to_explain: Vec<&TtpObject> = if let Some(name) = name_filter {
        let filtered: Vec<_> = parse_result
            .objects
            .iter()
            .filter(|obj| match obj {
                TtpObject::Sprite(s) => s.name == name,
                TtpObject::Palette(p) => p.name == name,
                TtpObject::Animation(a) => a.name == name,
                TtpObject::Composition(c) => c.name == name,
                TtpObject::Variant(v) => v.name == name,
                TtpObject::Particle(p) => p.name == name,
                TtpObject::Transform(t) => t.name == name,
            })
            .collect();

        if filtered.is_empty() {
            eprintln!("Error: No object named '{}' found", name);
            // Suggest similar names
            let all_names: Vec<&str> = parse_result
                .objects
                .iter()
                .map(|obj| match obj {
                    TtpObject::Sprite(s) => s.name.as_str(),
                    TtpObject::Palette(p) => p.name.as_str(),
                    TtpObject::Animation(a) => a.name.as_str(),
                    TtpObject::Composition(c) => c.name.as_str(),
                    TtpObject::Variant(v) => v.name.as_str(),
                    TtpObject::Particle(p) => p.name.as_str(),
                    TtpObject::Transform(t) => t.name.as_str(),
                })
                .collect();
            if let Some(suggestion) = format_suggestion(&suggest(name, &all_names, 3)) {
                eprintln!("{}", suggestion);
            }
            return ExitCode::from(EXIT_ERROR);
        }
        filtered
    } else {
        parse_result.objects.iter().collect()
    };

    // Generate explanations
    let mut explanations = Vec::new();
    for obj in objects_to_explain {
        // Resolve palette colors for sprites
        let palette_colors = match obj {
            TtpObject::Sprite(sprite) => resolve_palette_colors(&sprite.palette, &known_palettes),
            _ => None,
        };

        let explanation = explain_object(obj, palette_colors.as_ref());
        explanations.push(explanation);
    }

    // Output
    if json {
        // JSON output
        let json_explanations: Vec<serde_json::Value> = explanations
            .iter()
            .map(|exp| match exp {
                Explanation::Sprite(s) => serde_json::json!({
                    "type": "sprite",
                    "name": s.name,
                    "width": s.width,
                    "height": s.height,
                    "total_cells": s.total_cells,
                    "palette": s.palette_ref,
                    "tokens": s.tokens.iter().map(|t| serde_json::json!({
                        "token": t.token,
                        "count": t.count,
                        "percentage": t.percentage,
                        "color": t.color,
                        "color_name": t.color_name,
                    })).collect::<Vec<_>>(),
                    "transparency_ratio": s.transparency_ratio,
                    "consistent_rows": s.consistent_rows,
                    "issues": s.issues,
                }),
                Explanation::Palette(p) => serde_json::json!({
                    "type": "palette",
                    "name": p.name,
                    "color_count": p.color_count,
                    "colors": p.colors.iter().map(|(token, hex, name)| serde_json::json!({
                        "token": token,
                        "color": hex,
                        "color_name": name,
                    })).collect::<Vec<_>>(),
                    "is_builtin": p.is_builtin,
                }),
                Explanation::Animation(a) => serde_json::json!({
                    "type": "animation",
                    "name": a.name,
                    "frames": a.frames,
                    "frame_count": a.frame_count,
                    "duration_ms": a.duration_ms,
                    "loops": a.loops,
                }),
                Explanation::Composition(c) => serde_json::json!({
                    "type": "composition",
                    "name": c.name,
                    "base": c.base,
                    "size": c.size,
                    "cell_size": c.cell_size,
                    "sprite_count": c.sprite_count,
                    "layer_count": c.layer_count,
                }),
                Explanation::Variant(v) => serde_json::json!({
                    "type": "variant",
                    "name": v.name,
                    "base": v.base,
                    "override_count": v.override_count,
                    "overrides": v.overrides.iter().map(|(token, color)| serde_json::json!({
                        "token": token,
                        "color": color,
                    })).collect::<Vec<_>>(),
                }),
                Explanation::Particle(p) => serde_json::json!({
                    "type": "particle",
                    "name": p.name,
                    "sprite": p.sprite,
                    "rate": p.rate,
                    "lifetime": p.lifetime,
                    "has_gravity": p.has_gravity,
                    "has_fade": p.has_fade,
                }),
                Explanation::Transform(t) => serde_json::json!({
                    "type": "transform",
                    "name": t.name,
                    "is_parameterized": t.is_parameterized,
                    "params": t.params,
                    "generates_animation": t.generates_animation,
                    "frame_count": t.frame_count,
                    "transform_type": t.transform_type,
                }),
            })
            .collect();

        let output = if json_explanations.len() == 1 {
            serde_json::to_string_pretty(&json_explanations[0]).unwrap()
        } else {
            serde_json::to_string_pretty(&json_explanations).unwrap()
        };
        println!("{}", output);
    } else {
        // Text output
        for (i, exp) in explanations.iter().enumerate() {
            if i > 0 {
                println!("\n{}", "=".repeat(40));
                println!();
            }
            print!("{}", format_explanation(exp));
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the diff command
fn run_diff(file_a: &PathBuf, file_b: &PathBuf, sprite: Option<&str>, json: bool) -> ExitCode {
    // Get display names for the files
    let file_a_display = file_a.display().to_string();
    let file_b_display = file_b.display().to_string();

    // Compare the files
    let diffs = match diff_files(file_a, file_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Filter by sprite name if specified
    let filtered_diffs: Vec<_> = if let Some(name) = sprite {
        diffs.into_iter().filter(|(n, _)| n == name).collect()
    } else {
        diffs
    };

    if filtered_diffs.is_empty() {
        if sprite.is_some() {
            eprintln!("Error: Sprite '{}' not found in either file", sprite.unwrap());
            return ExitCode::from(EXIT_ERROR);
        }
        println!("No sprites found to compare.");
        return ExitCode::from(EXIT_SUCCESS);
    }

    if json {
        // JSON output
        let output: Vec<_> = filtered_diffs
            .iter()
            .map(|(name, diff)| {
                let mut obj = serde_json::json!({
                    "sprite": name,
                    "summary": diff.summary,
                });

                if let Some(ref dim) = diff.dimension_change {
                    obj["dimension_change"] = serde_json::json!({
                        "old": [dim.old.0, dim.old.1],
                        "new": [dim.new.0, dim.new.1],
                    });
                }

                if !diff.palette_changes.is_empty() {
                    let palette_changes: Vec<_> = diff
                        .palette_changes
                        .iter()
                        .map(|c| match c {
                            crate::diff::PaletteChange::Added { token, color } => {
                                serde_json::json!({
                                    "type": "added",
                                    "token": token,
                                    "color": color,
                                })
                            }
                            crate::diff::PaletteChange::Removed { token } => {
                                serde_json::json!({
                                    "type": "removed",
                                    "token": token,
                                })
                            }
                            crate::diff::PaletteChange::Changed { token, old_color, new_color } => {
                                serde_json::json!({
                                    "type": "changed",
                                    "token": token,
                                    "old_color": old_color,
                                    "new_color": new_color,
                                })
                            }
                        })
                        .collect();
                    obj["palette_changes"] = serde_json::json!(palette_changes);
                }

                if !diff.grid_changes.is_empty() {
                    let grid_changes: Vec<_> = diff
                        .grid_changes
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "row": c.row,
                                "description": c.description,
                            })
                        })
                        .collect();
                    obj["grid_changes"] = serde_json::json!(grid_changes);
                }

                obj
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        // Text output
        for (i, (name, diff)) in filtered_diffs.iter().enumerate() {
            if i > 0 {
                println!();
                println!("---");
                println!();
            }
            println!("{}", format_diff(name, diff, &file_a_display, &file_b_display));
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the suggest command
fn run_suggest(files: &[PathBuf], stdin: bool, json: bool, only: Option<&str>) -> ExitCode {
    use std::io::{self, BufReader};

    // Parse the --only filter
    let type_filter: Option<SuggestionType> = match only {
        Some("token") => Some(SuggestionType::MissingToken),
        Some("row") => Some(SuggestionType::RowCompletion),
        Some(other) => {
            eprintln!("Error: Unknown suggestion type '{}'. Use 'token' or 'row'.", other);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        None => None,
    };

    let mut suggester = Suggester::new();

    if stdin {
        // Read from stdin
        let stdin_handle = io::stdin();
        if let Err(e) = suggester.analyze_reader(stdin_handle.lock()) {
            eprintln!("Error reading stdin: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    } else {
        // Analyze files
        if files.is_empty() {
            eprintln!("Error: No files to analyze");
            return ExitCode::from(EXIT_INVALID_ARGS);
        }

        for path in files {
            if !json {
                println!("Analyzing {}...", path.display());
            }
            let file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: Cannot open '{}': {}", path.display(), e);
                    return ExitCode::from(EXIT_ERROR);
                }
            };
            if let Err(e) = suggester.analyze_reader(BufReader::new(file)) {
                eprintln!("Error reading '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        }
    }

    let report = suggester.into_report();

    // Apply type filter if specified
    let suggestions: Vec<_> = if let Some(filter_type) = type_filter {
        report.filter_by_type(filter_type).into_iter().cloned().collect()
    } else {
        report.suggestions.clone()
    };

    if json {
        // JSON output
        let output = serde_json::json!({
            "sprites_analyzed": report.sprites_analyzed,
            "suggestion_count": suggestions.len(),
            "suggestions": suggestions,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        // Text output
        if suggestions.is_empty() {
            println!();
            println!("No suggestions found.");
            println!("Analyzed {} sprite(s).", report.sprites_analyzed);
        } else {
            println!();
            println!(
                "Found {} suggestion(s) in {} sprite(s):",
                suggestions.len(),
                report.sprites_analyzed
            );
            println!();

            for suggestion in &suggestions {
                println!(
                    "Line {}: [{}] {}",
                    suggestion.line, suggestion.suggestion_type, suggestion.sprite
                );
                println!("  {}", suggestion.message);

                // Show fix details
                match &suggestion.fix {
                    SuggestionFix::ReplaceToken { from, to } => {
                        println!("  Fix: Replace {} with {}", from, to);
                    }
                    SuggestionFix::AddToPalette { token, suggested_color } => {
                        println!("  Fix: Add \"{}\": \"{}\" to palette", token, suggested_color);
                    }
                    SuggestionFix::ExtendRow {
                        row_index,
                        suggested,
                        tokens_to_add,
                        pad_token,
                        ..
                    } => {
                        println!(
                            "  Fix: Extend row {} by adding {} {} token(s)",
                            row_index + 1,
                            tokens_to_add,
                            pad_token
                        );
                        println!("  Suggested: \"{}\"", suggested);
                    }
                }
                println!();
            }

            // Summary by type
            let token_count = suggestions
                .iter()
                .filter(|s| s.suggestion_type == SuggestionType::MissingToken)
                .count();
            let row_count = suggestions
                .iter()
                .filter(|s| s.suggestion_type == SuggestionType::RowCompletion)
                .count();

            if token_count > 0 || row_count > 0 {
                print!("Summary: ");
                let mut parts = Vec::new();
                if token_count > 0 {
                    parts.push(format!("{} missing token(s)", token_count));
                }
                if row_count > 0 {
                    parts.push(format!("{} row completion(s)", row_count));
                }
                println!("{}", parts.join(", "));
            }
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the inline command
fn run_inline(input: &PathBuf, sprite_filter: Option<&str>) -> ExitCode {
    use crate::alias::{format_columns, parse_grid_row};
    use crate::models::TtpObject;

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

    // Collect sprites
    let mut sprites: Vec<_> = parse_result
        .objects
        .into_iter()
        .filter_map(|obj| match obj {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
        .collect();

    if sprites.is_empty() {
        eprintln!("Error: No sprites found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    // Filter by sprite name if specified
    if let Some(name) = sprite_filter {
        // Collect names for suggestion before filtering
        let sprite_names: Vec<String> = sprites.iter().map(|s| s.name.clone()).collect();
        sprites.retain(|s| s.name == name);
        if sprites.is_empty() {
            eprintln!("Error: No sprite named '{}' found in input", name);
            let name_refs: Vec<&str> = sprite_names.iter().map(|s| s.as_str()).collect();
            if let Some(suggestion) = format_suggestion(&suggest(name, &name_refs, 3)) {
                eprintln!("{}", suggestion);
            }
            return ExitCode::from(EXIT_ERROR);
        }
    }

    // Process each sprite
    for (i, sprite) in sprites.iter().enumerate() {
        if i > 0 {
            println!(); // Blank line between sprites
        }

        if sprites.len() > 1 {
            println!("# {}", sprite.name);
        }

        // Convert grid rows to tokenized vectors
        let rows: Vec<Vec<String>> = sprite.grid.iter().map(|row| parse_grid_row(row)).collect();

        // Format with column alignment
        let formatted = format_columns(rows);

        // Output each row
        for row in formatted {
            println!("{}", row);
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the alias command - extract repeated patterns into single-letter aliases
fn run_alias(input: &PathBuf, sprite_filter: Option<&str>) -> ExitCode {
    use crate::alias::extract_aliases;
    use crate::models::TtpObject;
    use serde_json::json;

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

    // Collect sprites
    let mut sprites: Vec<_> = parse_result
        .objects
        .into_iter()
        .filter_map(|obj| match obj {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
        .collect();

    if sprites.is_empty() {
        eprintln!("Error: No sprites found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    // Filter by sprite name if specified
    if let Some(name) = sprite_filter {
        // Collect names for suggestion before filtering
        let sprite_names: Vec<String> = sprites.iter().map(|s| s.name.clone()).collect();
        sprites.retain(|s| s.name == name);
        if sprites.is_empty() {
            eprintln!("Error: No sprite named '{}' found in input", name);
            let name_refs: Vec<&str> = sprite_names.iter().map(|s| s.as_str()).collect();
            if let Some(suggestion) = format_suggestion(&suggest(name, &name_refs, 3)) {
                eprintln!("{}", suggestion);
            }
            return ExitCode::from(EXIT_ERROR);
        }
    }

    // Process each sprite
    for (i, sprite) in sprites.iter().enumerate() {
        if i > 0 {
            println!(); // Blank line between sprites
        }

        // Extract aliases from the grid
        let (aliases, transformed_grid) = extract_aliases(&sprite.grid);

        // Convert aliases HashMap to a sorted JSON object (char -> String)
        // Sort by alias character for consistent output
        let mut alias_pairs: Vec<_> = aliases.iter().collect();
        alias_pairs.sort_by_key(|(c, _)| *c);
        let aliases_map: serde_json::Map<String, serde_json::Value> =
            alias_pairs.into_iter().map(|(c, name)| (c.to_string(), json!(name))).collect();

        // Build output JSON
        let output = json!({
            "aliases": aliases_map,
            "grid": transformed_grid
        });

        // Pretty-print the JSON
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the grid command
fn run_grid(input: &PathBuf, sprite_filter: Option<&str>, full_names: bool) -> ExitCode {
    use crate::models::TtpObject;

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

    // Collect sprites
    let mut sprites_by_name: std::collections::HashMap<String, crate::models::Sprite> =
        std::collections::HashMap::new();

    for obj in parse_result.objects {
        if let TtpObject::Sprite(sprite) = obj {
            sprites_by_name.insert(sprite.name.clone(), sprite);
        }
    }

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
        sprites_by_name.values().next().unwrap()
    };

    // Render the coordinate grid
    let output = render_coordinate_grid(&sprite.grid, full_names);

    // Print sprite name if there are multiple sprites
    if sprites_by_name.len() > 1 || sprite_filter.is_some() {
        println!("Sprite: {}", sprite.name);
        println!();
    }

    print!("{}", output);

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the show command - display sprite with colored terminal output
fn run_show(
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
    use crate::models::{Animation, Sprite, TtpObject};
    use crate::onion::{parse_hex_color, render_onion_skin, OnionConfig};
    use crate::registry::PaletteRegistry;
    use crate::renderer::render_sprite;
    use std::collections::HashMap;

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

    // Resolve the palette colors
    let resolved_palette = match registry.resolve(sprite, false) {
        Ok(result) => result.palette.colors,
        Err(e) => {
            eprintln!("Error: sprite '{}': {}", sprite.name, e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Convert palette colors to hex strings for render_ansi_grid
    let palette_hex: HashMap<String, String> =
        resolved_palette.iter().map(|(token, hex)| (token.clone(), hex.clone())).collect();

    // Build aliases map (empty for now - we'll use auto-aliasing)
    let aliases: HashMap<char, String> = HashMap::new();

    // Render the colored grid
    let (colored_output, legend) = render_ansi_grid(&sprite.grid, &palette_hex, &aliases);

    // Calculate dimensions from grid if size not provided
    let height = sprite.grid.len();
    let width = if let Some(size) = &sprite.size {
        size[0] as usize
    } else {
        // Infer from first row by counting tokens
        use crate::tokenizer::tokenize;
        sprite.grid.first().map(|row| tokenize(row).0.len()).unwrap_or(0)
    };

    // Print sprite name and dimensions
    println!("Sprite: {} ({}x{})", sprite.name, width, height);
    println!();

    // Print the colored grid
    print!("{}", colored_output);

    // Print the legend
    println!("{}", legend);

    ExitCode::from(EXIT_SUCCESS)
}

/// Run the build command
fn run_build(
    out: Option<&Path>,
    src: Option<&Path>,
    watch: bool,
    dry_run: bool,
    force: bool,
    verbose: bool,
) -> ExitCode {
    use crate::build::{BuildContext, BuildPipeline, IncrementalBuild, IncrementalStats};
    use crate::config::loader::{find_config, load_config, merge_cli_overrides, CliOverrides};

    // Find config file path and determine project root
    let (config, project_root) = match find_config() {
        Some(config_path) => {
            if verbose {
                println!("Using config: {}", config_path.display());
            }
            let cfg = match load_config(Some(&config_path)) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Error loading config: {}", e);
                    return ExitCode::from(EXIT_ERROR);
                }
            };
            let root = config_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            (cfg, root)
        }
        None => {
            if verbose {
                println!("No pxl.toml found, using defaults");
            }
            let root = std::env::current_dir().unwrap_or_default();
            (crate::config::loader::default_config(), root)
        }
    };

    // Apply CLI overrides to config
    let mut config = config;
    let overrides = CliOverrides {
        out: out.map(|p| p.to_path_buf()),
        src: src.map(|p| p.to_path_buf()),
        ..Default::default()
    };
    merge_cli_overrides(&mut config, &overrides);

    // Resolve source directory
    let src_dir = if config.project.src.is_absolute() {
        config.project.src.clone()
    } else {
        project_root.join(&config.project.src)
    };

    // Check source directory exists
    if !src_dir.exists() {
        eprintln!("Error: Source directory not found: {}", src_dir.display());
        eprintln!("Create the directory or specify a different path with --src");
        return ExitCode::from(EXIT_ERROR);
    }

    // Dry run mode
    if dry_run {
        let out_dir = if config.project.out.is_absolute() {
            config.project.out.clone()
        } else {
            project_root.join(&config.project.out)
        };

        println!("Dry run - would build:");
        println!("  Source: {}", src_dir.display());
        println!("  Output: {}", out_dir.display());

        // Use BuildPipeline in dry-run mode to discover targets
        let context = BuildContext::new(config, project_root).with_verbose(verbose);
        let pipeline = BuildPipeline::new(context).with_dry_run(true);

        match pipeline.build() {
            Ok(result) => {
                println!("  Targets: {}", result.targets.len());
                for target in &result.targets {
                    println!("    - {}", target.target_id);
                }
            }
            Err(e) => {
                eprintln!("  Error discovering targets: {}", e);
            }
        }
        return ExitCode::from(EXIT_SUCCESS);
    }

    // Watch mode using incremental build pipeline
    if watch {
        let watch_config = config.watch.clone();
        let context = BuildContext::new(config, project_root).with_verbose(verbose);

        println!("Starting watch mode...");
        if force {
            println!("Force mode: caching disabled");
        }
        println!("Press Ctrl+C to stop");
        println!();

        match crate::watch::watch_with_incremental(context, watch_config, force) {
            Ok(()) => ExitCode::from(EXIT_SUCCESS),
            Err(e) => {
                eprintln!("Watch error: {}", e);
                ExitCode::from(EXIT_ERROR)
            }
        }
    } else {
        // Single build using IncrementalBuild
        if force {
            println!("Building (force rebuild, ignoring cache)...");
        } else {
            println!("Building (incremental)...");
        }

        let context = BuildContext::new(config, project_root).with_verbose(verbose);
        let mut incremental = IncrementalBuild::new(context).with_force(force);

        match incremental.run() {
            Ok(result) => {
                let stats = IncrementalStats::from_result(&result);
                if result.is_success() {
                    // Show incremental stats in summary
                    if stats.had_skips() && !force {
                        println!("{} ({} skipped - unchanged)", result.summary(), stats.skipped);
                    } else {
                        println!("{}", result.summary());
                    }
                    ExitCode::from(EXIT_SUCCESS)
                } else {
                    eprintln!("{}", result.summary());
                    ExitCode::from(EXIT_ERROR)
                }
            }
            Err(e) => {
                eprintln!("Build error: {}", e);
                ExitCode::from(EXIT_ERROR)
            }
        }
    }
}

/// Run the new command
fn run_new(asset_type: &str, name: &str, palette: Option<&str>) -> ExitCode {
    use crate::scaffold::{new_animation, new_palette, new_sprite, ScaffoldError};

    let result = match asset_type.to_lowercase().as_str() {
        "sprite" => new_sprite(name, palette),
        "animation" | "anim" => new_animation(name, palette),
        "palette" => new_palette(name),
        _ => {
            eprintln!(
                "Unknown asset type '{}'. Available types: sprite, animation, palette",
                asset_type
            );
            return ExitCode::from(EXIT_ERROR);
        }
    };

    match result {
        Ok(path) => {
            println!("Created {} at {}", asset_type, path.display());
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(ScaffoldError::FileExists(path)) => {
            eprintln!("Error: File already exists: {}", path.display());
            ExitCode::from(EXIT_ERROR)
        }
        Err(ScaffoldError::NotInProject) => {
            eprintln!("Error: Not in a pixelsrc project (no pxl.toml found)");
            eprintln!("Run 'pxl init' to create a new project first");
            ExitCode::from(EXIT_ERROR)
        }
        Err(ScaffoldError::InvalidName(name)) => {
            eprintln!(
                "Error: Invalid asset name '{}'. Use lowercase letters, numbers, and underscores.",
                name
            );
            ExitCode::from(EXIT_ERROR)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(EXIT_ERROR)
        }
    }
}

/// Run the init command
fn run_init(path: Option<&Path>, name: Option<&str>, preset: &str) -> ExitCode {
    use crate::init::{init_project, InitError};

    // Determine project path
    let project_path = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    // Determine project name
    let project_name = name
        .map(|n| n.to_string())
        .or_else(|| project_path.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "my-project".to_string());

    // Run initialization
    match init_project(&project_path, &project_name, preset) {
        Ok(()) => {
            println!("Created pixelsrc project '{}' at {}", project_name, project_path.display());
            println!();
            println!("Project structure:");
            println!("  {}/", project_path.display());
            println!("  ├── pxl.toml");
            println!("  ├── .gitignore");
            println!("  ├── src/pxl/");
            println!("  │   ├── palettes/main.pxl");
            println!("  │   └── sprites/example.pxl");
            println!("  └── build/");
            println!();
            println!("Next steps:");
            println!("  cd {}", project_path.display());
            println!("  pxl render src/pxl/sprites/example.pxl");
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(InitError::DirectoryExists(dir)) => {
            eprintln!("Error: Directory '{}' already exists and is not empty", dir);
            eprintln!("Use an empty directory or specify a different path");
            ExitCode::from(EXIT_ERROR)
        }
        Err(InitError::UnknownPreset(preset)) => {
            eprintln!("Error: Unknown preset '{}'", preset);
            eprintln!("Available presets: minimal, artist, animator, game");
            ExitCode::from(EXIT_ERROR)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(EXIT_ERROR)
        }
    }
}

/// Create sprite from simple text grid input
fn run_sketch(
    file: Option<&Path>,
    name: &str,
    palette: Option<&str>,
    output: Option<&Path>,
) -> ExitCode {
    use std::io::{self, Read, Write};

    // Read input
    let input = match file {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error: Cannot read '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        },
        None => {
            // Read from stdin
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("Error: Cannot read from stdin: {}", e);
                return ExitCode::from(EXIT_ERROR);
            }
            buffer
        }
    };

    // Parse the simple grid
    let grid = parse_simple_grid(&input);

    if grid.is_empty() {
        eprintln!("Error: Empty input - no grid data found");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Convert to sprite JSON
    let sprite = simple_grid_to_sprite(grid, name, palette);

    // Format output as pretty JSON
    let json_output = match serde_json::to_string_pretty(&sprite) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to serialize sprite: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Write output
    match output {
        Some(path) => {
            let mut file = match std::fs::File::create(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: Cannot create '{}': {}", path.display(), e);
                    return ExitCode::from(EXIT_ERROR);
                }
            };
            if let Err(e) = writeln!(file, "{}", json_output) {
                eprintln!("Error: Cannot write to '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        }
        None => {
            println!("{}", json_output);
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Transform sprites (mirror, rotate, tile, etc.)
///
/// Applies transforms to sprite grids and outputs new source files.
#[allow(clippy::too_many_arguments)]
fn run_transform(
    input: &Path,
    mirror: Option<&str>,
    rotate: Option<u16>,
    tile: Option<&str>,
    pad: Option<u32>,
    outline: Option<Option<String>>,
    outline_width: u32,
    crop: Option<&str>,
    shift: Option<&str>,
    shadow: Option<&str>,
    shadow_token: Option<&str>,
    sprite_name: Option<&str>,
    output: &Path,
    stdin: bool,
    allow_large: bool,
) -> ExitCode {
    use std::io::{self, Cursor, Read, Write};

    // Read input
    let content = if stdin {
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error: Cannot read from stdin: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
        buffer
    } else {
        match std::fs::read_to_string(input) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Cannot read '{}': {}", input.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        }
    };

    // Parse the file
    let reader = Cursor::new(content.as_bytes());
    let parse_result = parse_stream(reader);
    let objects = parse_result.objects;

    // Report any parsing warnings
    for warning in &parse_result.warnings {
        eprintln!("Warning: {}", warning.message);
    }

    // Find the target sprite
    let sprites: Vec<&Sprite> = objects
        .iter()
        .filter_map(|obj| match obj {
            TtpObject::Sprite(s) => Some(s),
            _ => None,
        })
        .collect();

    if sprites.is_empty() {
        eprintln!("Error: No sprites found in input file");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    let target_sprite = match sprite_name {
        Some(name) => match sprites.iter().find(|s| s.name == name) {
            Some(s) => *s,
            None => {
                eprintln!("Error: Sprite '{}' not found in input file", name);
                eprintln!(
                    "Available sprites: {}",
                    sprites.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")
                );
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        },
        None => {
            if sprites.len() > 1 {
                eprintln!("Warning: Multiple sprites found, using '{}'", sprites[0].name);
                eprintln!("Use --sprite to specify which sprite to transform");
            }
            sprites[0]
        }
    };

    // Get the grid (grid is Vec<String>, not Option)
    let mut grid: Vec<String> = if target_sprite.grid.is_empty() {
        eprintln!("Error: Sprite '{}' has no grid data", target_sprite.name);
        return ExitCode::from(EXIT_ERROR);
    } else {
        target_sprite.grid.clone()
    };

    // Apply transforms in order of specification (simulate CLI order by checking each)
    // Note: In a real CLI implementation, we'd track the order flags were specified.
    // For now, we apply in a logical order: geometric -> expansion -> effects

    // Geometric transforms: mirror
    if let Some(axis) = mirror {
        match axis.to_lowercase().as_str() {
            "h" | "horizontal" => {
                grid = apply_mirror_horizontal(&grid);
            }
            "v" | "vertical" => {
                grid = apply_mirror_vertical(&grid);
            }
            "both" => {
                grid = apply_mirror_horizontal(&grid);
                grid = apply_mirror_vertical(&grid);
            }
            _ => {
                eprintln!(
                    "Error: Invalid mirror axis '{}'. Use 'horizontal', 'vertical', or 'both'",
                    axis
                );
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        }
    }

    // Geometric transforms: rotate
    if let Some(degrees) = rotate {
        if degrees != 90 && degrees != 180 && degrees != 270 {
            eprintln!("Error: Invalid rotation degrees {}. Use 90, 180, or 270", degrees);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        grid = apply_rotate(&grid, degrees);
    }

    // Expansion transforms: tile
    if let Some(tile_spec) = tile {
        let parts: Vec<&str> = tile_spec.split('x').collect();
        if parts.len() != 2 {
            eprintln!("Error: Invalid tile format '{}'. Use 'WxH' (e.g., '2x3')", tile_spec);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        let w: u32 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid tile width '{}'. Must be a positive integer", parts[0]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        let h: u32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid tile height '{}'. Must be a positive integer", parts[1]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };

        // Check for large expansion
        if !allow_large && w * h > 100 {
            eprintln!("Warning: Tile {}x{} creates {} copies of the sprite", w, h, w * h);
            eprintln!("Use --allow-large to proceed with large expansions");
            return ExitCode::from(EXIT_INVALID_ARGS);
        }

        grid = apply_tile(&grid, w, h);
    }

    // Expansion transforms: pad
    if let Some(size) = pad {
        grid = apply_pad(&grid, size);
    }

    // Expansion transforms: crop
    if let Some(crop_spec) = crop {
        let parts: Vec<&str> = crop_spec.split(',').collect();
        if parts.len() != 4 {
            eprintln!(
                "Error: Invalid crop format '{}'. Use 'X,Y,W,H' (e.g., '0,0,8,8')",
                crop_spec
            );
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        let x: u32 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid crop X '{}'", parts[0]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        let y: u32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid crop Y '{}'", parts[1]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        let w: u32 = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid crop width '{}'", parts[2]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        let h: u32 = match parts[3].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid crop height '{}'", parts[3]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        grid = apply_crop(&grid, x, y, w, h);
    }

    // Effect transforms: shift
    if let Some(shift_spec) = shift {
        let parts: Vec<&str> = shift_spec.split(',').collect();
        if parts.len() != 2 {
            eprintln!("Error: Invalid shift format '{}'. Use 'X,Y' (e.g., '4,0')", shift_spec);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        let x: i32 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid shift X '{}'", parts[0]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        let y: i32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid shift Y '{}'", parts[1]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        grid = apply_shift(&grid, x, y);
    }

    // Effect transforms: shadow
    if let Some(shadow_spec) = shadow {
        let parts: Vec<&str> = shadow_spec.split(',').collect();
        if parts.len() < 2 {
            eprintln!("Error: Invalid shadow format '{}'. Use 'X,Y' (e.g., '2,2')", shadow_spec);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        let x: i32 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid shadow X '{}'", parts[0]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        let y: i32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Error: Invalid shadow Y '{}'", parts[1]);
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        };
        grid = apply_shadow(&grid, x, y, shadow_token);
    }

    // Effect transforms: outline
    if let Some(outline_opt) = outline {
        // outline_opt is Option<String> - Some("token") or Some("") for --outline with value, None for bare --outline
        let token = outline_opt.as_deref().filter(|s| !s.is_empty());
        grid = apply_outline(&grid, token, outline_width);
    }

    // Build the output sprite
    let mut output_sprite = target_sprite.clone();
    output_sprite.grid = grid;

    // Serialize to JSON Lines format
    let sprite_json = match serde_json::to_string(&serde_json::json!({
        "type": "sprite",
        "name": output_sprite.name,
        "palette": output_sprite.palette,
        "grid": output_sprite.grid,
    })) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to serialize sprite: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Find all non-sprite objects (palettes, animations, etc.) to preserve
    let mut output_lines: Vec<String> = Vec::new();

    // Add all palettes first
    for obj in &objects {
        if let TtpObject::Palette(p) = obj {
            match serde_json::to_string(&serde_json::json!({
                "type": "palette",
                "name": p.name,
                "colors": p.colors,
            })) {
                Ok(line) => output_lines.push(line),
                Err(e) => {
                    eprintln!("Error: Failed to serialize palette '{}': {}", p.name, e);
                    return ExitCode::from(EXIT_ERROR);
                }
            }
        }
    }

    // Add the transformed sprite
    output_lines.push(sprite_json);

    // Add any other sprites that weren't transformed
    for obj in &objects {
        if let TtpObject::Sprite(s) = obj {
            if s.name != target_sprite.name {
                match serde_json::to_string(&serde_json::json!({
                    "type": "sprite",
                    "name": s.name,
                    "palette": s.palette,
                    "grid": s.grid,
                })) {
                    Ok(line) => output_lines.push(line),
                    Err(e) => {
                        eprintln!("Error: Failed to serialize sprite '{}': {}", s.name, e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                }
            }
        }
    }

    // Write output
    let output_content = output_lines.join("\n");

    let mut file = match std::fs::File::create(output) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Cannot create '{}': {}", output.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    if let Err(e) = writeln!(file, "{}", output_content) {
        eprintln!("Error: Cannot write to '{}': {}", output.display(), e);
        return ExitCode::from(EXIT_ERROR);
    }

    ExitCode::from(EXIT_SUCCESS)
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
