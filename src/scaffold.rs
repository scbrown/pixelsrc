//! Asset scaffolding for pixelsrc projects
//!
//! Provides templates for creating new sprites, animations, and palettes.

use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::config::loader::find_config;

/// Error during asset scaffolding
#[derive(Debug, Error)]
pub enum ScaffoldError {
    /// File already exists
    #[error("File already exists: {}", .0.display())]
    FileExists(PathBuf),
    /// Failed to create directory
    #[error("Failed to create directory: {0}")]
    CreateDir(std::io::Error),
    /// Failed to write file
    #[error("Failed to write file: {0}")]
    WriteFile(std::io::Error),
    /// Not in a pixelsrc project (no pxl.toml found)
    #[error("Not in a pixelsrc project (no pxl.toml found)")]
    NotInProject,
    /// Invalid asset name
    #[error("Invalid asset name '{0}'. Use lowercase letters, numbers, and underscores.")]
    InvalidName(String),
}

/// Validate an asset name.
///
/// Names must:
/// - Start with a lowercase letter
/// - Contain only lowercase letters, numbers, and underscores
/// - Not be empty
fn validate_name(name: &str) -> Result<(), ScaffoldError> {
    if name.is_empty() {
        return Err(ScaffoldError::InvalidName(name.to_string()));
    }

    // SAFETY: We just verified name.is_empty() is false, so first char exists
    let first = name.chars().next().expect("name is non-empty");
    if !first.is_ascii_lowercase() {
        return Err(ScaffoldError::InvalidName(name.to_string()));
    }

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' {
            return Err(ScaffoldError::InvalidName(name.to_string()));
        }
    }

    Ok(())
}

/// Find the project root by locating pxl.toml.
fn find_project_root() -> Result<PathBuf, ScaffoldError> {
    find_config()
        .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
        .ok_or(ScaffoldError::NotInProject)
}

/// Create a new sprite file.
///
/// Creates a sprite template at `src/pxl/sprites/<name>.pxl`.
///
/// # Arguments
/// * `name` - Sprite name (used for filename and sprite name in file)
/// * `palette` - Optional palette name (default: "main")
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the created file
/// * `Err(ScaffoldError)` - If creation fails
///
/// # Example
/// ```ignore
/// let path = new_sprite("hero", Some("characters"))?;
/// println!("Created sprite at: {}", path.display());
/// ```
pub fn new_sprite(name: &str, palette: Option<&str>) -> Result<PathBuf, ScaffoldError> {
    validate_name(name)?;

    let project_root = find_project_root()?;
    let sprites_dir = project_root.join("src/pxl/sprites");
    let file_path = sprites_dir.join(format!("{}.pxl", name));

    // Check if file already exists
    if file_path.exists() {
        return Err(ScaffoldError::FileExists(file_path));
    }

    // Create sprites directory if needed
    fs::create_dir_all(&sprites_dir).map_err(ScaffoldError::CreateDir)?;

    // Generate sprite content
    let palette_name = palette.unwrap_or("main");
    let content = generate_sprite_template(name, palette_name);

    // Write file
    fs::write(&file_path, content).map_err(ScaffoldError::WriteFile)?;

    Ok(file_path)
}

/// Create a new animation file.
///
/// Creates an animation template at `src/pxl/animations/<name>.pxl`.
///
/// # Arguments
/// * `name` - Animation name (used for filename and animation name in file)
/// * `palette` - Optional palette name (default: "main")
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the created file
/// * `Err(ScaffoldError)` - If creation fails
pub fn new_animation(name: &str, palette: Option<&str>) -> Result<PathBuf, ScaffoldError> {
    validate_name(name)?;

    let project_root = find_project_root()?;
    let animations_dir = project_root.join("src/pxl/animations");
    let file_path = animations_dir.join(format!("{}.pxl", name));

    // Check if file already exists
    if file_path.exists() {
        return Err(ScaffoldError::FileExists(file_path));
    }

    // Create animations directory if needed
    fs::create_dir_all(&animations_dir).map_err(ScaffoldError::CreateDir)?;

    // Generate animation content
    let palette_name = palette.unwrap_or("main");
    let content = generate_animation_template(name, palette_name);

    // Write file
    fs::write(&file_path, content).map_err(ScaffoldError::WriteFile)?;

    Ok(file_path)
}

/// Create a new palette file.
///
/// Creates a palette template at `src/pxl/palettes/<name>.pxl`.
///
/// # Arguments
/// * `name` - Palette name (used for filename and palette name in file)
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the created file
/// * `Err(ScaffoldError)` - If creation fails
pub fn new_palette(name: &str) -> Result<PathBuf, ScaffoldError> {
    validate_name(name)?;

    let project_root = find_project_root()?;
    let palettes_dir = project_root.join("src/pxl/palettes");
    let file_path = palettes_dir.join(format!("{}.pxl", name));

    // Check if file already exists
    if file_path.exists() {
        return Err(ScaffoldError::FileExists(file_path));
    }

    // Create palettes directory if needed
    fs::create_dir_all(&palettes_dir).map_err(ScaffoldError::CreateDir)?;

    // Generate palette content
    let content = generate_palette_template(name);

    // Write file
    fs::write(&file_path, content).map_err(ScaffoldError::WriteFile)?;

    Ok(file_path)
}

/// Generate sprite template content.
fn generate_sprite_template(name: &str, palette: &str) -> String {
    format!(
        r#"{{"type": "sprite", "name": "{}", "size": [4, 4], "palette": "{}", "regions": {{}}}}"#,
        name, palette
    )
}

/// Generate animation template content.
fn generate_animation_template(name: &str, palette: &str) -> String {
    let frame1 = format!("{}_1", name);
    let frame2 = format!("{}_2", name);

    format!(
        r#"{{"type": "sprite", "name": "{}", "size": [4, 4], "palette": "{}", "regions": {{}}}}
{{"type": "sprite", "name": "{}", "size": [4, 4], "palette": "{}", "regions": {{}}}}
{{"type": "animation", "name": "{}", "frames": ["{}", "{}"], "duration": 200}}"#,
        frame1, palette, frame2, palette, name, frame1, frame2
    )
}

/// Generate palette template content.
fn generate_palette_template(name: &str) -> String {
    format!(
        r##"{{"type": "palette", "name": "{}", "colors": {{
  "{{_}}": "#00000000",
  "{{black}}": "#000000",
  "{{white}}": "#FFFFFF",
  "{{color1}}": "#FF0000",
  "{{color2}}": "#00FF00",
  "{{color3}}": "#0000FF"
}}}}"##,
        name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Create a minimal project structure for testing
    fn setup_test_project(temp: &TempDir) -> PathBuf {
        let project_path = temp.path().to_path_buf();

        // Create pxl.toml
        let config = r#"[project]
name = "test"
version = "0.1.0"
"#;
        fs::write(project_path.join("pxl.toml"), config).unwrap();

        // Create directory structure
        fs::create_dir_all(project_path.join("src/pxl/sprites")).unwrap();
        fs::create_dir_all(project_path.join("src/pxl/animations")).unwrap();
        fs::create_dir_all(project_path.join("src/pxl/palettes")).unwrap();

        project_path
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("hero").is_ok());
        assert!(validate_name("player_idle").is_ok());
        assert!(validate_name("sprite1").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("walk_cycle_1").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_name("").is_err());
        assert!(validate_name("Hero").is_err()); // uppercase
        assert!(validate_name("1sprite").is_err()); // starts with number
        assert!(validate_name("_underscore").is_err()); // starts with underscore
        assert!(validate_name("my-sprite").is_err()); // contains hyphen
        assert!(validate_name("my sprite").is_err()); // contains space
    }

    #[test]
    fn test_generate_sprite_template() {
        let content = generate_sprite_template("hero", "main");
        assert!(content.contains("\"type\": \"sprite\""));
        assert!(content.contains("\"name\": \"hero\""));
        assert!(content.contains("\"palette\": \"main\""));
        assert!(content.contains("\"regions\""));
    }

    #[test]
    fn test_generate_animation_template() {
        let content = generate_animation_template("walk", "characters");
        assert!(content.contains("\"type\": \"sprite\""));
        assert!(content.contains("\"name\": \"walk_1\""));
        assert!(content.contains("\"name\": \"walk_2\""));
        assert!(content.contains("\"type\": \"animation\""));
        assert!(content.contains("\"name\": \"walk\""));
        assert!(content.contains("\"palette\": \"characters\""));
        assert!(content.contains("\"duration\": 200"));
    }

    #[test]
    fn test_generate_palette_template() {
        let content = generate_palette_template("enemies");
        assert!(content.contains("\"type\": \"palette\""));
        assert!(content.contains("\"name\": \"enemies\""));
        assert!(content.contains("\"{_}\""));
        assert!(content.contains("\"{black}\""));
        assert!(content.contains("\"{white}\""));
    }

    #[test]
    #[serial]
    fn test_new_sprite_creates_file() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_test_project(&temp);

        // Change to project directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_path).unwrap();

        let result = new_sprite("hero", Some("main"));

        // Restore directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.ends_with("src/pxl/sprites/hero.pxl"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"name\": \"hero\""));
        assert!(content.contains("\"palette\": \"main\""));
    }

    #[test]
    #[serial]
    fn test_new_sprite_default_palette() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_test_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_path).unwrap();

        let result = new_sprite("enemy", None);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let path = result.unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"palette\": \"main\""));
    }

    #[test]
    #[serial]
    fn test_new_sprite_file_exists_error() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_test_project(&temp);

        // Create existing file
        let sprite_path = project_path.join("src/pxl/sprites/existing.pxl");
        fs::write(&sprite_path, "existing content").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_path).unwrap();

        let result = new_sprite("existing", None);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(matches!(result, Err(ScaffoldError::FileExists(_))));
    }

    #[test]
    #[serial]
    fn test_new_animation_creates_file() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_test_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_path).unwrap();

        let result = new_animation("walk", Some("characters"));

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.ends_with("src/pxl/animations/walk.pxl"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"name\": \"walk_1\""));
        assert!(content.contains("\"name\": \"walk_2\""));
        assert!(content.contains("\"type\": \"animation\""));
        assert!(content.contains("\"name\": \"walk\""));
    }

    #[test]
    #[serial]
    fn test_new_palette_creates_file() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_test_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_path).unwrap();

        let result = new_palette("enemies");

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.ends_with("src/pxl/palettes/enemies.pxl"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"type\": \"palette\""));
        assert!(content.contains("\"name\": \"enemies\""));
    }

    #[test]
    #[serial]
    fn test_new_sprite_invalid_name() {
        let temp = TempDir::new().unwrap();
        let project_path = setup_test_project(&temp);

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_path).unwrap();

        let result = new_sprite("Invalid-Name", None);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(matches!(result, Err(ScaffoldError::InvalidName(_))));
    }

    #[test]
    #[serial]
    fn test_not_in_project_error() {
        let temp = TempDir::new().unwrap();
        // Don't create pxl.toml

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = new_sprite("test", None);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(matches!(result, Err(ScaffoldError::NotInProject)));
    }

    #[test]
    fn test_sprite_template_valid_json() {
        let content = generate_sprite_template("test", "main");
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(parsed.is_ok(), "Sprite template should be valid JSON");
    }

    #[test]
    fn test_palette_template_valid_json() {
        let content = generate_palette_template("test");
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(parsed.is_ok(), "Palette template should be valid JSON");
    }
}
