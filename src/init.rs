//! Project initialization for pixelsrc
//!
//! Provides scaffolding for new pixelsrc projects with various presets.

use std::fs;
use std::path::Path;

/// Error during project initialization
#[derive(Debug)]
pub enum InitError {
    /// Directory already exists
    DirectoryExists(String),
    /// Failed to create directory
    CreateDir(std::io::Error),
    /// Failed to write file
    WriteFile(std::io::Error),
    /// Unknown preset
    UnknownPreset(String),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::DirectoryExists(path) => {
                write!(f, "Directory already exists: {}", path)
            }
            InitError::CreateDir(e) => write!(f, "Failed to create directory: {}", e),
            InitError::WriteFile(e) => write!(f, "Failed to write file: {}", e),
            InitError::UnknownPreset(preset) => {
                write!(
                    f,
                    "Unknown preset '{}'. Available: minimal, artist, animator, game",
                    preset
                )
            }
        }
    }
}

impl std::error::Error for InitError {}

/// Available project presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Single sprite, basic render
    Minimal,
    /// Static art workflow with palettes, sprites, variants
    Artist,
    /// Animation workflow with sprites, animations, justfile
    Animator,
    /// Full game asset pipeline with atlases, multi-export
    Game,
}

impl Preset {
    /// Parse preset name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" => Some(Preset::Minimal),
            "artist" => Some(Preset::Artist),
            "animator" => Some(Preset::Animator),
            "game" => Some(Preset::Game),
            _ => None,
        }
    }
}

/// Initialize a new pixelsrc project.
///
/// Creates the project directory structure and starter files based on the
/// selected preset.
///
/// # Arguments
/// - `path` - Directory to create the project in
/// - `name` - Project name (used in pxl.toml)
/// - `preset` - Preset template name ("minimal", "artist", "animator", "game")
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(InitError)` if initialization fails
///
/// # Example
/// ```ignore
/// init_project(Path::new("my-game"), "my-game", "minimal")?;
/// ```
pub fn init_project(path: &Path, name: &str, preset: &str) -> Result<(), InitError> {
    let preset = Preset::from_str(preset).ok_or_else(|| InitError::UnknownPreset(preset.to_string()))?;

    // Check if directory already exists and is not empty
    if path.exists() {
        let is_empty = path
            .read_dir()
            .map(|mut d| d.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            return Err(InitError::DirectoryExists(path.display().to_string()));
        }
    }

    // Create project based on preset
    match preset {
        Preset::Minimal => init_minimal(path, name),
        // Other presets will be implemented in BST-9
        Preset::Artist | Preset::Animator | Preset::Game => {
            // For now, fall back to minimal
            init_minimal(path, name)
        }
    }
}

/// Initialize a minimal project structure.
fn init_minimal(path: &Path, name: &str) -> Result<(), InitError> {
    // Create directory structure
    create_dir(path)?;
    create_dir(&path.join("src/pxl/palettes"))?;
    create_dir(&path.join("src/pxl/sprites"))?;
    create_dir(&path.join("build"))?;

    // Write pxl.toml
    let pxl_toml = generate_minimal_config(name);
    write_file(&path.join("pxl.toml"), &pxl_toml)?;

    // Write .gitignore
    let gitignore = generate_gitignore();
    write_file(&path.join(".gitignore"), &gitignore)?;

    // Write example palette
    let palette = generate_main_palette();
    write_file(&path.join("src/pxl/palettes/main.pxl"), &palette)?;

    // Write example sprite
    let sprite = generate_example_sprite();
    write_file(&path.join("src/pxl/sprites/example.pxl"), &sprite)?;

    // Write build/.gitkeep
    write_file(&path.join("build/.gitkeep"), "")?;

    Ok(())
}

/// Create a directory and all parent directories.
fn create_dir(path: &Path) -> Result<(), InitError> {
    fs::create_dir_all(path).map_err(InitError::CreateDir)
}

/// Write content to a file.
fn write_file(path: &Path, content: &str) -> Result<(), InitError> {
    fs::write(path, content).map_err(InitError::WriteFile)
}

/// Generate minimal pxl.toml configuration.
fn generate_minimal_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{}"
version = "0.1.0"
"#,
        name
    )
}

/// Generate .gitignore content.
fn generate_gitignore() -> String {
    r#"# Pixelsrc build output
build/

# OS files
.DS_Store
Thumbs.db
"#
    .to_string()
}

/// Generate main palette file content.
fn generate_main_palette() -> String {
    r##"{"type": "palette", "name": "main", "colors": {"{_}": "#00000000", "{black}": "#000000", "{white}": "#FFFFFF", "{primary}": "#4A90D9", "{secondary}": "#D94A4A"}}"##.to_string()
}

/// Generate example sprite file content.
fn generate_example_sprite() -> String {
    r#"{"type": "sprite", "name": "example", "palette": "main", "grid": [
  "{_}{primary}{primary}{_}",
  "{primary}{white}{white}{primary}",
  "{primary}{white}{white}{primary}",
  "{_}{primary}{primary}{_}"
]}"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_preset_from_str() {
        assert_eq!(Preset::from_str("minimal"), Some(Preset::Minimal));
        assert_eq!(Preset::from_str("MINIMAL"), Some(Preset::Minimal));
        assert_eq!(Preset::from_str("artist"), Some(Preset::Artist));
        assert_eq!(Preset::from_str("animator"), Some(Preset::Animator));
        assert_eq!(Preset::from_str("game"), Some(Preset::Game));
        assert_eq!(Preset::from_str("unknown"), None);
    }

    #[test]
    fn test_init_minimal_creates_structure() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test-project");

        init_project(&project_path, "test-project", "minimal").unwrap();

        // Check directories exist
        assert!(project_path.join("src/pxl/palettes").exists());
        assert!(project_path.join("src/pxl/sprites").exists());
        assert!(project_path.join("build").exists());

        // Check files exist
        assert!(project_path.join("pxl.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join("src/pxl/palettes/main.pxl").exists());
        assert!(project_path.join("src/pxl/sprites/example.pxl").exists());
        assert!(project_path.join("build/.gitkeep").exists());
    }

    #[test]
    fn test_init_minimal_config_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("my-game");

        init_project(&project_path, "my-game", "minimal").unwrap();

        let config = fs::read_to_string(project_path.join("pxl.toml")).unwrap();
        assert!(config.contains("name = \"my-game\""));
        assert!(config.contains("version = \"0.1.0\""));
    }

    #[test]
    fn test_init_minimal_palette_valid_json() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test");

        init_project(&project_path, "test", "minimal").unwrap();

        let palette = fs::read_to_string(project_path.join("src/pxl/palettes/main.pxl")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&palette).unwrap();
        assert_eq!(parsed["type"], "palette");
        assert_eq!(parsed["name"], "main");
    }

    #[test]
    fn test_init_minimal_sprite_valid_json() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test");

        init_project(&project_path, "test", "minimal").unwrap();

        let sprite = fs::read_to_string(project_path.join("src/pxl/sprites/example.pxl")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sprite).unwrap();
        assert_eq!(parsed["type"], "sprite");
        assert_eq!(parsed["name"], "example");
        assert_eq!(parsed["palette"], "main");
    }

    #[test]
    fn test_init_gitignore_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test");

        init_project(&project_path, "test", "minimal").unwrap();

        let gitignore = fs::read_to_string(project_path.join(".gitignore")).unwrap();
        assert!(gitignore.contains("build/"));
        assert!(gitignore.contains(".DS_Store"));
    }

    #[test]
    fn test_init_existing_non_empty_dir_fails() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("existing");

        // Create a non-empty directory
        fs::create_dir_all(&project_path).unwrap();
        fs::write(project_path.join("some-file.txt"), "content").unwrap();

        let result = init_project(&project_path, "existing", "minimal");
        assert!(matches!(result, Err(InitError::DirectoryExists(_))));
    }

    #[test]
    fn test_init_existing_empty_dir_succeeds() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("empty");

        // Create an empty directory
        fs::create_dir_all(&project_path).unwrap();

        let result = init_project(&project_path, "empty", "minimal");
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_unknown_preset_fails() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test");

        let result = init_project(&project_path, "test", "nonexistent");
        assert!(matches!(result, Err(InitError::UnknownPreset(_))));
    }

    #[test]
    fn test_init_in_current_dir() {
        let temp = TempDir::new().unwrap();

        // Initialize in the temp directory itself (simulating "pxl init .")
        init_project(temp.path(), "current", "minimal").unwrap();

        assert!(temp.path().join("pxl.toml").exists());
        assert!(temp.path().join("src/pxl").exists());
    }
}
