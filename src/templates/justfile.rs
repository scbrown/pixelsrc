//! Justfile template generation for pixelsrc projects.
//!
//! Generates justfiles (command runner scripts) tailored to different
//! pixelsrc workflows: rendering, animation, or full game asset pipelines.
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::templates::{JustfileTemplate, generate_justfile};
//!
//! // Generate a game-focused justfile
//! let content = generate_justfile(JustfileTemplate::Game, "my-game");
//! std::fs::write("justfile", content)?;
//!
//! // Or generate for an existing project
//! let content = generate_justfile_for_project(&config)?;
//! ```

use std::path::Path;

/// Justfile template variants for different workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustfileTemplate {
    /// Minimal template with basic render commands
    Minimal,
    /// Static art workflow with palette swaps
    Artist,
    /// Animation workflow with GIF generation
    Animator,
    /// Full game pipeline with atlases and exports
    Game,
}

impl JustfileTemplate {
    /// Parse template name from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" => Some(Self::Minimal),
            "artist" => Some(Self::Artist),
            "animator" => Some(Self::Animator),
            "game" => Some(Self::Game),
            _ => None,
        }
    }

    /// Get the template name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Artist => "artist",
            Self::Animator => "animator",
            Self::Game => "game",
        }
    }

    /// Get a description of this template.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Minimal => "Basic render commands only",
            Self::Artist => "Static art workflow with palette variants",
            Self::Animator => "Animation workflow with GIF previews",
            Self::Game => "Full game pipeline with atlases and exports",
        }
    }
}

/// Generate a justfile for the given template and project name.
pub fn generate_justfile(template: JustfileTemplate, project_name: &str) -> String {
    match template {
        JustfileTemplate::Minimal => generate_minimal_justfile(project_name),
        JustfileTemplate::Artist => generate_artist_justfile(project_name),
        JustfileTemplate::Animator => generate_animator_justfile(project_name),
        JustfileTemplate::Game => generate_game_justfile(project_name),
    }
}

/// Options for customizing justfile generation.
#[derive(Debug, Clone, Default)]
pub struct JustfileOptions {
    /// Source directory (default: src/pxl)
    pub src_dir: Option<String>,
    /// Build/output directory (default: build)
    pub out_dir: Option<String>,
    /// Include watch commands
    pub include_watch: bool,
    /// Include CI commands
    pub include_ci: bool,
    /// Include atlas-specific commands
    pub include_atlas: bool,
    /// Include GIF generation commands
    pub include_gif: bool,
    /// Atlas names for atlas-specific commands
    pub atlas_names: Vec<String>,
}

/// Generate a justfile with custom options.
pub fn generate_justfile_with_options(
    template: JustfileTemplate,
    project_name: &str,
    options: &JustfileOptions,
) -> String {
    let src_dir = options.src_dir.as_deref().unwrap_or("src/pxl");
    let out_dir = options.out_dir.as_deref().unwrap_or("build");

    let mut sections = Vec::new();

    // Header
    sections.push(format!(
        "# {} - Pixelsrc {} project commands\n",
        project_name,
        template.name()
    ));

    // Default target
    let default_target = match template {
        JustfileTemplate::Minimal => "render",
        JustfileTemplate::Artist => "render",
        JustfileTemplate::Animator => "preview",
        JustfileTemplate::Game => "build",
    };
    sections.push(format!("default: {}\n", default_target));

    // Build/render commands
    match template {
        JustfileTemplate::Game => {
            sections.push(format!(
                r#"# Build all assets
build: validate
    pxl build

# Validate all source files
validate:
    pxl validate {}/ --strict
"#,
                src_dir
            ));
        }
        _ => {
            sections.push(format!(
                r#"# Render all sprites
render:
    pxl render {src}/**/*.pxl -o {out}/
"#,
                src = src_dir,
                out = out_dir
            ));
        }
    }

    // Validation for non-game templates
    if !matches!(template, JustfileTemplate::Game) {
        sections.push(format!(
            r#"# Validate all source files
validate:
    pxl validate {}/
"#,
            src_dir
        ));
    }

    // Watch commands
    if options.include_watch || matches!(template, JustfileTemplate::Animator | JustfileTemplate::Game)
    {
        sections.push(
            r#"# Watch for changes and rebuild
watch:
    pxl build --watch
"#
            .to_string(),
        );
    }

    // GIF commands
    if options.include_gif || matches!(template, JustfileTemplate::Animator | JustfileTemplate::Game)
    {
        sections.push(format!(
            r#"# Generate preview GIFs for animations
preview:
    pxl render {}/animations/*.pxl --gif -o {}/preview/
"#,
            src_dir, out_dir
        ));

        if matches!(template, JustfileTemplate::Animator) {
            sections.push(
                r#"# Render specific animation as GIF
gif name:
    pxl render src/pxl/animations/{{name}}.pxl --gif -o build/preview/{{name}}.gif

# Render animation as spritesheet
sheet name:
    pxl render src/pxl/animations/{{name}}.pxl --spritesheet -o build/{{name}}_sheet.png
"#
                .to_string(),
            );
        }
    }

    // Atlas commands
    if options.include_atlas || matches!(template, JustfileTemplate::Game) {
        if !options.atlas_names.is_empty() {
            for atlas_name in &options.atlas_names {
                sections.push(format!(
                    r#"# Build {} atlas only
atlas-{}:
    pxl build --atlas {}
"#,
                    atlas_name, atlas_name, atlas_name
                ));
            }
        } else if matches!(template, JustfileTemplate::Game) {
            sections.push(
                r#"# Build sprites atlas only
atlas-sprites:
    pxl build --atlas sprites

# Build UI atlas only
atlas-ui:
    pxl build --atlas ui
"#
                .to_string(),
            );
        }
    }

    // Clean command
    sections.push(format!(
        r#"# Clean build directory
clean:
    rm -rf {}/*
"#,
        out_dir
    ));

    // CI commands
    if options.include_ci || matches!(template, JustfileTemplate::Game) {
        sections.push(format!(
            r#"# Show project stats
stats:
    pxl analyze {}/

# Format all source files
fmt:
    pxl fmt {}/

# Check formatting without changes
fmt-check:
    pxl fmt {}/ --check

# Full CI build
ci: fmt-check validate build
    @echo "CI build complete"
"#,
            src_dir, src_dir, src_dir
        ));
    }

    sections.join("\n")
}

/// Generate minimal justfile.
fn generate_minimal_justfile(_project_name: &str) -> String {
    r#"# Pixelsrc minimal project commands

default: render

# Render all sprites
render:
    pxl render src/pxl/**/*.pxl -o build/

# Render specific sprite
sprite name:
    pxl render src/pxl/**/{{name}}.pxl -o build/{{name}}.png

# Validate all source files
validate:
    pxl validate src/pxl/

# Clean build directory
clean:
    rm -rf build/*
"#
    .to_string()
}

/// Generate artist justfile.
fn generate_artist_justfile(_project_name: &str) -> String {
    r#"# Pixelsrc artist project commands

default: render

# Render all sprites
render:
    pxl render src/pxl/sprites/*.pxl -o build/

# Render all variants
variants:
    pxl render src/pxl/variants/*.pxl -o build/variants/

# Render everything
all: render variants

# Render at multiple scales
scales:
    pxl render src/pxl/sprites/*.pxl --scale 1 -o build/1x/
    pxl render src/pxl/sprites/*.pxl --scale 2 -o build/2x/
    pxl render src/pxl/sprites/*.pxl --scale 4 -o build/4x/

# Validate all source files
validate:
    pxl validate src/pxl/

# Format source files
fmt:
    pxl fmt src/pxl/

# Clean build directory
clean:
    rm -rf build/*
"#
    .to_string()
}

/// Generate animator justfile.
fn generate_animator_justfile(_project_name: &str) -> String {
    r#"# Pixelsrc animation project commands

default: preview

# Generate preview GIFs for all animations
preview:
    pxl render src/pxl/animations/*.pxl --gif -o build/preview/

# Watch for changes and regenerate previews
watch:
    pxl build --watch

# Render all sprites at 2x scale
render:
    pxl render src/pxl/sprites/*.pxl --scale 2 -o build/

# Render specific animation as GIF
gif name:
    pxl render src/pxl/animations/{{name}}.pxl --gif -o build/preview/{{name}}.gif

# Render animation as spritesheet
sheet name:
    pxl render src/pxl/animations/{{name}}.pxl --spritesheet -o build/{{name}}_sheet.png

# Validate all source files
validate:
    pxl validate src/pxl/ --strict

# Clean build directory
clean:
    rm -rf build/*
"#
    .to_string()
}

/// Generate game justfile.
fn generate_game_justfile(_project_name: &str) -> String {
    r#"# Pixelsrc game project commands

default: build

# Build all assets
build: validate
    pxl build

# Validate all source files
validate:
    pxl validate src/pxl/ --strict

# Watch for changes and rebuild
watch:
    pxl build --watch

# Generate preview GIFs for animations
preview:
    pxl render src/pxl/animations/*.pxl --gif -o build/preview/

# Build sprites atlas only
atlas-sprites:
    pxl build --atlas sprites

# Build UI atlas only
atlas-ui:
    pxl build --atlas ui

# Clean build directory
clean:
    rm -rf build/*
    mkdir -p build/atlases build/preview

# Show project stats
stats:
    pxl analyze src/pxl/

# Format all source files
fmt:
    pxl fmt src/pxl/

# Check formatting without changes
fmt-check:
    pxl fmt src/pxl/ --check

# Full CI build
ci: fmt-check validate build
    @echo "CI build complete"
"#
    .to_string()
}

/// Write a justfile to the specified path.
pub fn write_justfile(path: &Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

/// List all available justfile templates.
pub fn list_templates() -> Vec<(JustfileTemplate, &'static str, &'static str)> {
    vec![
        (
            JustfileTemplate::Minimal,
            "minimal",
            "Basic render commands only",
        ),
        (
            JustfileTemplate::Artist,
            "artist",
            "Static art workflow with palette variants",
        ),
        (
            JustfileTemplate::Animator,
            "animator",
            "Animation workflow with GIF previews",
        ),
        (
            JustfileTemplate::Game,
            "game",
            "Full game pipeline with atlases and exports",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_from_str() {
        assert_eq!(
            JustfileTemplate::from_str("minimal"),
            Some(JustfileTemplate::Minimal)
        );
        assert_eq!(
            JustfileTemplate::from_str("GAME"),
            Some(JustfileTemplate::Game)
        );
        assert_eq!(
            JustfileTemplate::from_str("Animator"),
            Some(JustfileTemplate::Animator)
        );
        assert_eq!(JustfileTemplate::from_str("unknown"), None);
    }

    #[test]
    fn test_template_name() {
        assert_eq!(JustfileTemplate::Minimal.name(), "minimal");
        assert_eq!(JustfileTemplate::Artist.name(), "artist");
        assert_eq!(JustfileTemplate::Animator.name(), "animator");
        assert_eq!(JustfileTemplate::Game.name(), "game");
    }

    #[test]
    fn test_template_description() {
        assert!(!JustfileTemplate::Minimal.description().is_empty());
        assert!(!JustfileTemplate::Game.description().is_empty());
    }

    #[test]
    fn test_generate_minimal_justfile() {
        let content = generate_justfile(JustfileTemplate::Minimal, "test");
        assert!(content.contains("default: render"));
        assert!(content.contains("render:"));
        assert!(content.contains("validate:"));
        assert!(content.contains("clean:"));
    }

    #[test]
    fn test_generate_artist_justfile() {
        let content = generate_justfile(JustfileTemplate::Artist, "test");
        assert!(content.contains("default: render"));
        assert!(content.contains("variants:"));
        assert!(content.contains("scales:"));
    }

    #[test]
    fn test_generate_animator_justfile() {
        let content = generate_justfile(JustfileTemplate::Animator, "test");
        assert!(content.contains("default: preview"));
        assert!(content.contains("--gif"));
        assert!(content.contains("watch:"));
        assert!(content.contains("gif name:"));
        assert!(content.contains("sheet name:"));
    }

    #[test]
    fn test_generate_game_justfile() {
        let content = generate_justfile(JustfileTemplate::Game, "test");
        assert!(content.contains("default: build"));
        assert!(content.contains("build: validate"));
        assert!(content.contains("atlas-sprites:"));
        assert!(content.contains("atlas-ui:"));
        assert!(content.contains("ci:"));
    }

    #[test]
    fn test_generate_with_options() {
        let options = JustfileOptions {
            src_dir: Some("assets/pxl".to_string()),
            out_dir: Some("dist".to_string()),
            include_watch: true,
            include_ci: true,
            include_atlas: false,
            include_gif: false,
            atlas_names: vec![],
        };

        let content =
            generate_justfile_with_options(JustfileTemplate::Minimal, "test", &options);

        assert!(content.contains("assets/pxl"));
        assert!(content.contains("dist"));
        assert!(content.contains("watch:"));
        assert!(content.contains("ci:"));
    }

    #[test]
    fn test_generate_with_custom_atlases() {
        let options = JustfileOptions {
            atlas_names: vec!["characters".to_string(), "items".to_string()],
            include_atlas: true,
            ..Default::default()
        };

        let content =
            generate_justfile_with_options(JustfileTemplate::Minimal, "test", &options);

        assert!(content.contains("atlas-characters:"));
        assert!(content.contains("atlas-items:"));
    }

    #[test]
    fn test_write_justfile() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("justfile");

        let content = generate_justfile(JustfileTemplate::Minimal, "test");
        write_justfile(&path, &content).unwrap();

        assert!(path.exists());
        let read_content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_list_templates() {
        let templates = list_templates();
        assert_eq!(templates.len(), 4);

        let names: Vec<&str> = templates.iter().map(|(_, n, _)| *n).collect();
        assert!(names.contains(&"minimal"));
        assert!(names.contains(&"artist"));
        assert!(names.contains(&"animator"));
        assert!(names.contains(&"game"));
    }

    #[test]
    fn test_justfile_valid_syntax() {
        // Check that generated justfiles have valid basic syntax
        for template in [
            JustfileTemplate::Minimal,
            JustfileTemplate::Artist,
            JustfileTemplate::Animator,
            JustfileTemplate::Game,
        ] {
            let content = generate_justfile(template, "test");

            // Should have a default target
            assert!(
                content.contains("default:"),
                "Missing default in {:?}",
                template
            );

            // Should not have consecutive empty lines (justfile syntax)
            assert!(
                !content.contains("\n\n\n"),
                "Too many empty lines in {:?}",
                template
            );

            // Check recipe structure: after a recipe line (contains :),
            // following indented lines are commands
            let mut in_recipe = false;
            for line in content.lines() {
                if line.is_empty() {
                    in_recipe = false;
                    continue;
                }

                let is_indented = line.starts_with(' ') || line.starts_with('\t');
                let is_comment = line.trim().starts_with('#');

                if is_comment {
                    continue;
                }

                if is_indented {
                    // Indented lines are commands - should be inside a recipe
                    assert!(
                        in_recipe,
                        "Indented line outside recipe in {:?}: {}",
                        template, line
                    );
                } else {
                    // Non-indented, non-empty line should be a recipe definition
                    let trimmed = line.trim();
                    if trimmed.contains(':') || trimmed.contains('=') {
                        in_recipe = true;
                    }
                }
            }
        }
    }
}
