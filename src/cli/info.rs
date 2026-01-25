//! Info command implementations (prime, prompts, palettes)

use clap::Subcommand;
use std::process::ExitCode;

use crate::palettes;
use crate::prime::{get_primer, list_sections, PrimerSection};
use crate::suggest::{format_suggestion, suggest};

use super::{EXIT_ERROR, EXIT_SUCCESS};

// Embedded prompt templates
const TEMPLATE_CHARACTER: &str = include_str!("../../docs/prompts/templates/character.txt");
const TEMPLATE_ITEM: &str = include_str!("../../docs/prompts/templates/item.txt");
const TEMPLATE_TILESET: &str = include_str!("../../docs/prompts/templates/tileset.txt");
const TEMPLATE_ANIMATION: &str = include_str!("../../docs/prompts/templates/animation.txt");

/// Available template names
const TEMPLATES: &[(&str, &str)] = &[
    ("character", TEMPLATE_CHARACTER),
    ("item", TEMPLATE_ITEM),
    ("tileset", TEMPLATE_TILESET),
    ("animation", TEMPLATE_ANIMATION),
];

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

/// Execute the prime command
pub fn run_prime(brief: bool, section: Option<&str>) -> ExitCode {
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
pub fn run_prompts(template: Option<&str>) -> ExitCode {
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
pub fn run_palettes(action: PaletteAction) -> ExitCode {
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
