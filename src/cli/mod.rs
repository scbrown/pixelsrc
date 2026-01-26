//! Command-line interface implementation
//!
//! This module provides the CLI entry point and dispatches to submodules
//! for specific command implementations.

mod agent;
mod build;
mod explain;
mod import;
mod info;
mod render;
mod show;
mod validate;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

use glob::glob;

// Re-export subcommand types used in Commands enum
pub use agent::AgentAction;
pub use info::PaletteAction;

/// Exit codes per Pixelsrc spec
pub(crate) const EXIT_SUCCESS: u8 = 0;
pub(crate) const EXIT_ERROR: u8 = 1;
pub(crate) const EXIT_INVALID_ARGS: u8 = 2;

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

        /// Render nine-slice sprite to target size (e.g., "64x32")
        /// Requires sprite to have nine_slice attribute defined
        #[arg(long)]
        nine_slice: Option<String>,
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

        /// Enable role/relationship inference analysis
        #[arg(long)]
        analyze: bool,

        /// Confidence threshold for inferences (0.0-1.0, default: 0.7)
        #[arg(long, default_value = "0.7")]
        confidence: f64,

        /// Show token naming suggestions based on detected features
        #[arg(long)]
        hints: bool,

        /// Disable extraction of structured shapes (polygons, rects)
        #[arg(long)]
        no_shapes: bool,
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

    /// Start the Language Server Protocol server (for editor integration)
    #[cfg(feature = "lsp")]
    #[command(hide = true)]
    Lsp,

    /// Agent-mode validation and diagnostics (for AI/CLI integration)
    Agent {
        /// Subcommand: verify, completions, position
        #[command(subcommand)]
        action: AgentAction,
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
            nine_slice,
        } => render::run_render(
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
            nine_slice.as_deref(),
        ),
        Commands::Import {
            input,
            output,
            max_colors,
            name,
            analyze,
            confidence,
            hints,
            no_shapes,
        } => import::run_import(
            &input,
            output.as_deref(),
            max_colors,
            name.as_deref(),
            analyze,
            confidence,
            hints,
            !no_shapes,
        ),
        Commands::Prompts { template } => info::run_prompts(template.as_deref()),
        Commands::Palettes { action } => info::run_palettes(action),
        Commands::Analyze { files, dir, recursive, format, output } => {
            validate::run_analyze(&files, dir.as_deref(), recursive, &format, output.as_deref())
        }
        Commands::Fmt { files, check, stdout } => validate::run_fmt(&files, check, stdout),
        Commands::Prime { brief, section } => info::run_prime(brief, section.as_deref()),
        Commands::Validate { files, stdin, strict, json } => {
            validate::run_validate(&files, stdin, strict, json)
        }
        Commands::AgentVerify {
            stdin,
            content,
            strict,
            grid_info,
            suggest_tokens,
            resolve_colors,
            analyze_timing,
        } => validate::run_agent_verify(
            stdin,
            content.as_deref(),
            strict,
            grid_info,
            suggest_tokens,
            resolve_colors,
            analyze_timing,
        ),
        Commands::Explain { input, name, json } => {
            explain::run_explain(&input, name.as_deref(), json)
        }
        Commands::Diff { file_a, file_b, sprite, json } => {
            explain::run_diff(&file_a, &file_b, sprite.as_deref(), json)
        }
        Commands::Suggest { files, stdin, json, only } => {
            explain::run_suggest(&files, stdin, json, only.as_deref())
        }
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
        } => show::run_show(
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
            build::run_build(out.as_deref(), src.as_deref(), watch, dry_run, force, verbose)
        }
        Commands::New { asset_type, name, palette } => {
            build::run_new(&asset_type, &name, palette.as_deref())
        }
        Commands::Init { path, name, preset } => {
            build::run_init(path.as_deref(), name.as_deref(), &preset)
        }
        #[cfg(feature = "lsp")]
        Commands::Lsp => agent::run_lsp(),
        Commands::Agent { action } => agent::run_agent(action),
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
