//! Install command implementation

use std::process::ExitCode;

use super::{EXIT_ERROR, EXIT_SUCCESS};

/// Run the install command
pub fn run_install(clean: bool, verbose: bool) -> ExitCode {
    use crate::config::loader::{find_config, load_config};
    use crate::install::install_dependencies;

    // Find and load config
    let (config, project_root) = match find_config() {
        Some(config_path) => {
            let cfg = match load_config(Some(&config_path)) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Error loading pxl.toml: {}", e);
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
            eprintln!("Error: No pxl.toml found. Run `pxl init` to create a project.");
            return ExitCode::from(EXIT_ERROR);
        }
    };

    if config.dependencies.is_empty() {
        println!("No dependencies declared in pxl.toml");
        return ExitCode::from(EXIT_SUCCESS);
    }

    let dep_count = config.dependencies.len();
    if clean {
        println!(
            "Installing {} {} (clean)...",
            dep_count,
            if dep_count == 1 { "dependency" } else { "dependencies" }
        );
    } else {
        println!(
            "Installing {} {}...",
            dep_count,
            if dep_count == 1 { "dependency" } else { "dependencies" }
        );
    }

    match install_dependencies(&config, &project_root, clean, verbose) {
        Ok(result) => {
            println!("{}", result.summary());

            if !result.failed.is_empty() {
                eprintln!();
                for (name, err) in &result.failed {
                    eprintln!("  FAIL {}: {}", name, err);
                }
                ExitCode::from(EXIT_ERROR)
            } else {
                ExitCode::from(EXIT_SUCCESS)
            }
        }
        Err(e) => {
            eprintln!("Install failed: {}", e);
            ExitCode::from(EXIT_ERROR)
        }
    }
}
