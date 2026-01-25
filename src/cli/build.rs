//! Build command implementations (build, new, init)

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use super::{EXIT_ERROR, EXIT_SUCCESS};

/// Run the build command
pub fn run_build(
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
pub fn run_new(asset_type: &str, name: &str, palette: Option<&str>) -> ExitCode {
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
pub fn run_init(path: Option<&Path>, name: Option<&str>, preset: &str) -> ExitCode {
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
