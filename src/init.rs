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
                write!(f, "Unknown preset '{}'. Available: minimal, artist, animator, game", preset)
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
    let preset =
        Preset::from_str(preset).ok_or_else(|| InitError::UnknownPreset(preset.to_string()))?;

    // Check if directory already exists and is not empty
    if path.exists() {
        let is_empty = path.read_dir().map(|mut d| d.next().is_none()).unwrap_or(false);
        if !is_empty {
            return Err(InitError::DirectoryExists(path.display().to_string()));
        }
    }

    // Create project based on preset
    match preset {
        Preset::Minimal => init_minimal(path, name),
        Preset::Artist => init_artist(path, name),
        Preset::Animator => init_animator(path, name),
        Preset::Game => init_game(path, name),
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

// ============================================================================
// Artist Preset
// ============================================================================

/// Initialize an artist project structure.
///
/// Static art workflow with palettes, sprites, and variant support.
fn init_artist(path: &Path, name: &str) -> Result<(), InitError> {
    // Create directory structure
    create_dir(path)?;
    create_dir(&path.join("src/pxl/palettes"))?;
    create_dir(&path.join("src/pxl/sprites"))?;
    create_dir(&path.join("src/pxl/variants"))?;
    create_dir(&path.join("build"))?;

    // Write pxl.toml
    let pxl_toml = generate_artist_config(name);
    write_file(&path.join("pxl.toml"), &pxl_toml)?;

    // Write .gitignore
    let gitignore = generate_gitignore();
    write_file(&path.join(".gitignore"), &gitignore)?;

    // Write palettes
    let main_palette = generate_main_palette();
    write_file(&path.join("src/pxl/palettes/main.pxl"), &main_palette)?;

    let alt_palette = generate_alt_palette();
    write_file(&path.join("src/pxl/palettes/alt.pxl"), &alt_palette)?;

    // Write example sprite
    let sprite = generate_character_sprite();
    write_file(&path.join("src/pxl/sprites/character.pxl"), &sprite)?;

    // Write variant example
    let variant = generate_variant_example();
    write_file(&path.join("src/pxl/variants/character_alt.pxl"), &variant)?;

    // Write build/.gitkeep
    write_file(&path.join("build/.gitkeep"), "")?;

    Ok(())
}

/// Generate artist preset pxl.toml configuration.
fn generate_artist_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[defaults]
scale = 1
padding = 0

[validate]
strict = false
unused_palettes = "warn"
"#,
        name
    )
}

/// Generate alternate palette file content.
fn generate_alt_palette() -> String {
    r##"{"type": "palette", "name": "alt", "colors": {"{_}": "#00000000", "{black}": "#1a1a2e", "{white}": "#eaeaea", "{primary}": "#e94560", "{secondary}": "#533483"}}"##.to_string()
}

/// Generate character sprite file content.
fn generate_character_sprite() -> String {
    r#"{"type": "sprite", "name": "character", "palette": "main", "grid": [
  "{_}{_}{primary}{primary}{_}{_}",
  "{_}{primary}{white}{white}{primary}{_}",
  "{primary}{white}{black}{black}{white}{primary}",
  "{primary}{white}{white}{white}{white}{primary}",
  "{_}{primary}{secondary}{secondary}{primary}{_}",
  "{_}{_}{primary}{primary}{_}{_}"
]}"#
    .to_string()
}

/// Generate variant example file content.
fn generate_variant_example() -> String {
    r#"{"type": "sprite", "name": "character_alt", "palette": "alt", "grid": [
  "{_}{_}{primary}{primary}{_}{_}",
  "{_}{primary}{white}{white}{primary}{_}",
  "{primary}{white}{black}{black}{white}{primary}",
  "{primary}{white}{white}{white}{white}{primary}",
  "{_}{primary}{secondary}{secondary}{primary}{_}",
  "{_}{_}{primary}{primary}{_}{_}"
]}"#
    .to_string()
}

// ============================================================================
// Animator Preset
// ============================================================================

/// Initialize an animator project structure.
///
/// Animation workflow with sprites, animations, and justfile with GIF recipes.
fn init_animator(path: &Path, name: &str) -> Result<(), InitError> {
    // Create directory structure
    create_dir(path)?;
    create_dir(&path.join("src/pxl/palettes"))?;
    create_dir(&path.join("src/pxl/sprites"))?;
    create_dir(&path.join("src/pxl/animations"))?;
    create_dir(&path.join("build/preview"))?;

    // Write pxl.toml
    let pxl_toml = generate_animator_config(name);
    write_file(&path.join("pxl.toml"), &pxl_toml)?;

    // Write .gitignore
    let gitignore = generate_gitignore();
    write_file(&path.join(".gitignore"), &gitignore)?;

    // Write justfile
    let justfile = generate_animator_justfile();
    write_file(&path.join("justfile"), &justfile)?;

    // Write palette
    let palette = generate_main_palette();
    write_file(&path.join("src/pxl/palettes/main.pxl"), &palette)?;

    // Write animation frames
    let frames = generate_animation_frames();
    write_file(&path.join("src/pxl/sprites/walk.pxl"), &frames)?;

    // Write animation definition
    let animation = generate_animation_definition();
    write_file(&path.join("src/pxl/animations/walk.pxl"), &animation)?;

    // Write build/.gitkeep
    write_file(&path.join("build/.gitkeep"), "")?;

    Ok(())
}

/// Generate animator preset pxl.toml configuration.
fn generate_animator_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[defaults]
scale = 2
padding = 0

[animations]
sources = ["animations/**"]
preview = true
preview_scale = 4

[watch]
debounce_ms = 100
clear_screen = true
"#,
        name
    )
}

/// Generate animator justfile.
fn generate_animator_justfile() -> String {
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

/// Generate animation frame sprites.
fn generate_animation_frames() -> String {
    r##"{"type": "palette", "name": "walk", "colors": {"{_}": "#00000000", "{b}": "#2d2d2d", "{s}": "#4a90d9", "{h}": "#ffcc00"}}
{"type": "sprite", "name": "walk_1", "palette": "walk", "grid": ["{_}{h}{h}{_}", "{_}{s}{s}{_}", "{b}{s}{s}{b}", "{_}{b}{b}{_}"]}
{"type": "sprite", "name": "walk_2", "palette": "walk", "grid": ["{_}{h}{h}{_}", "{_}{s}{s}{_}", "{_}{s}{s}{_}", "{b}{_}{_}{b}"]}
{"type": "sprite", "name": "walk_3", "palette": "walk", "grid": ["{_}{h}{h}{_}", "{_}{s}{s}{_}", "{b}{s}{s}{b}", "{_}{b}{b}{_}"]}
{"type": "sprite", "name": "walk_4", "palette": "walk", "grid": ["{_}{h}{h}{_}", "{_}{s}{s}{_}", "{_}{s}{s}{_}", "{b}{_}{_}{b}"]}"##
        .to_string()
}

/// Generate animation definition.
fn generate_animation_definition() -> String {
    r#"{"type": "include", "path": "../sprites/walk.pxl"}
{"type": "animation", "name": "walk_cycle", "frames": ["walk_1", "walk_2", "walk_3", "walk_4"], "duration": 150}"#
        .to_string()
}

// ============================================================================
// Game Preset
// ============================================================================

/// Initialize a game project structure.
///
/// Full game asset pipeline with atlases, multi-export, and justfile.
fn init_game(path: &Path, name: &str) -> Result<(), InitError> {
    // Create directory structure
    create_dir(path)?;
    create_dir(&path.join("src/pxl/palettes"))?;
    create_dir(&path.join("src/pxl/sprites/player"))?;
    create_dir(&path.join("src/pxl/sprites/items"))?;
    create_dir(&path.join("src/pxl/animations"))?;
    create_dir(&path.join("src/pxl/ui"))?;
    create_dir(&path.join("build/atlases"))?;
    create_dir(&path.join("build/preview"))?;

    // Write pxl.toml
    let pxl_toml = generate_game_config(name);
    write_file(&path.join("pxl.toml"), &pxl_toml)?;

    // Write .gitignore
    let gitignore = generate_gitignore();
    write_file(&path.join(".gitignore"), &gitignore)?;

    // Write justfile
    let justfile = generate_game_justfile();
    write_file(&path.join("justfile"), &justfile)?;

    // Write palettes
    let main_palette = generate_main_palette();
    write_file(&path.join("src/pxl/palettes/main.pxl"), &main_palette)?;

    let ui_palette = generate_ui_palette();
    write_file(&path.join("src/pxl/palettes/ui.pxl"), &ui_palette)?;

    // Write player sprites
    let player = generate_player_sprites();
    write_file(&path.join("src/pxl/sprites/player/idle.pxl"), &player)?;

    // Write item sprites
    let items = generate_item_sprites();
    write_file(&path.join("src/pxl/sprites/items/coin.pxl"), &items)?;

    // Write animation
    let animation = generate_player_animation();
    write_file(&path.join("src/pxl/animations/player.pxl"), &animation)?;

    // Write UI elements
    let ui = generate_ui_elements();
    write_file(&path.join("src/pxl/ui/button.pxl"), &ui)?;

    // Write .gitkeep files
    write_file(&path.join("build/.gitkeep"), "")?;

    Ok(())
}

/// Generate game preset pxl.toml configuration.
fn generate_game_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{}"
version = "0.1.0"
src = "src/pxl"
out = "build"

[defaults]
scale = 1
padding = 1

[atlases.sprites]
sources = ["sprites/**"]
max_size = [1024, 1024]
padding = 1
power_of_two = true

[atlases.ui]
sources = ["ui/**"]
max_size = [512, 512]
padding = 0

[animations]
sources = ["animations/**"]
preview = true
preview_scale = 4
sheet_layout = "horizontal"

[export.generic]
enabled = true

[validate]
strict = true
unused_palettes = "warn"
missing_refs = "error"

[watch]
debounce_ms = 100
clear_screen = true
"#,
        name
    )
}

/// Generate game justfile.
fn generate_game_justfile() -> String {
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

/// Generate UI palette file content.
fn generate_ui_palette() -> String {
    r##"{"type": "palette", "name": "ui", "colors": {"{_}": "#00000000", "{bg}": "#1a1a2e", "{border}": "#4a4a6a", "{text}": "#ffffff", "{accent}": "#e94560"}}"##.to_string()
}

/// Generate player sprites.
fn generate_player_sprites() -> String {
    r#"{"type": "sprite", "name": "player_idle_1", "palette": "main", "grid": [
  "{_}{primary}{primary}{_}",
  "{primary}{white}{white}{primary}",
  "{_}{primary}{primary}{_}",
  "{_}{black}{black}{_}"
]}
{"type": "sprite", "name": "player_idle_2", "palette": "main", "grid": [
  "{_}{primary}{primary}{_}",
  "{primary}{white}{white}{primary}",
  "{_}{primary}{primary}{_}",
  "{black}{_}{_}{black}"
]}"#
    .to_string()
}

/// Generate item sprites.
fn generate_item_sprites() -> String {
    r##"{"type": "palette", "name": "items", "colors": {"{_}": "#00000000", "{gold}": "#FFD700", "{shine}": "#FFFFE0"}}
{"type": "sprite", "name": "coin", "palette": "items", "grid": ["{_}{gold}{gold}{_}", "{gold}{shine}{gold}{gold}", "{gold}{gold}{shine}{gold}", "{_}{gold}{gold}{_}"]}"##
        .to_string()
}

/// Generate player animation.
fn generate_player_animation() -> String {
    r#"{"type": "include", "path": "../sprites/player/idle.pxl"}
{"type": "animation", "name": "player_idle", "frames": ["player_idle_1", "player_idle_2"], "duration": 500}"#
        .to_string()
}

/// Generate UI elements.
fn generate_ui_elements() -> String {
    r#"{"type": "sprite", "name": "button_normal", "palette": "ui", "grid": [
  "{border}{border}{border}{border}{border}{border}",
  "{border}{bg}{bg}{bg}{bg}{border}",
  "{border}{bg}{text}{text}{bg}{border}",
  "{border}{bg}{bg}{bg}{bg}{border}",
  "{border}{border}{border}{border}{border}{border}"
]}
{"type": "sprite", "name": "button_hover", "palette": "ui", "grid": [
  "{accent}{accent}{accent}{accent}{accent}{accent}",
  "{accent}{bg}{bg}{bg}{bg}{accent}",
  "{accent}{bg}{text}{text}{bg}{accent}",
  "{accent}{bg}{bg}{bg}{bg}{accent}",
  "{accent}{accent}{accent}{accent}{accent}{accent}"
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

    // ========================================================================
    // Artist Preset Tests
    // ========================================================================

    #[test]
    fn test_init_artist_creates_structure() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("art-project");

        init_project(&project_path, "art-project", "artist").unwrap();

        // Check directories
        assert!(project_path.join("src/pxl/palettes").exists());
        assert!(project_path.join("src/pxl/sprites").exists());
        assert!(project_path.join("src/pxl/variants").exists());
        assert!(project_path.join("build").exists());

        // Check files
        assert!(project_path.join("pxl.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join("src/pxl/palettes/main.pxl").exists());
        assert!(project_path.join("src/pxl/palettes/alt.pxl").exists());
        assert!(project_path.join("src/pxl/sprites/character.pxl").exists());
        assert!(project_path.join("src/pxl/variants/character_alt.pxl").exists());
    }

    #[test]
    fn test_init_artist_config_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("art");

        init_project(&project_path, "art", "artist").unwrap();

        let config = fs::read_to_string(project_path.join("pxl.toml")).unwrap();
        assert!(config.contains("name = \"art\""));
        assert!(config.contains("[defaults]"));
        assert!(config.contains("[validate]"));
    }

    #[test]
    fn test_init_artist_palettes_valid_json() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("art");

        init_project(&project_path, "art", "artist").unwrap();

        // Check main palette
        let main = fs::read_to_string(project_path.join("src/pxl/palettes/main.pxl")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&main).unwrap();
        assert_eq!(parsed["type"], "palette");
        assert_eq!(parsed["name"], "main");

        // Check alt palette
        let alt = fs::read_to_string(project_path.join("src/pxl/palettes/alt.pxl")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&alt).unwrap();
        assert_eq!(parsed["type"], "palette");
        assert_eq!(parsed["name"], "alt");
    }

    // ========================================================================
    // Animator Preset Tests
    // ========================================================================

    #[test]
    fn test_init_animator_creates_structure() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("anim-project");

        init_project(&project_path, "anim-project", "animator").unwrap();

        // Check directories
        assert!(project_path.join("src/pxl/palettes").exists());
        assert!(project_path.join("src/pxl/sprites").exists());
        assert!(project_path.join("src/pxl/animations").exists());
        assert!(project_path.join("build/preview").exists());

        // Check files
        assert!(project_path.join("pxl.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join("justfile").exists());
        assert!(project_path.join("src/pxl/palettes/main.pxl").exists());
        assert!(project_path.join("src/pxl/sprites/walk.pxl").exists());
        assert!(project_path.join("src/pxl/animations/walk.pxl").exists());
    }

    #[test]
    fn test_init_animator_config_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("anim");

        init_project(&project_path, "anim", "animator").unwrap();

        let config = fs::read_to_string(project_path.join("pxl.toml")).unwrap();
        assert!(config.contains("name = \"anim\""));
        assert!(config.contains("[animations]"));
        assert!(config.contains("preview = true"));
        assert!(config.contains("[watch]"));
    }

    #[test]
    fn test_init_animator_justfile_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("anim");

        init_project(&project_path, "anim", "animator").unwrap();

        let justfile = fs::read_to_string(project_path.join("justfile")).unwrap();
        assert!(justfile.contains("preview:"));
        assert!(justfile.contains("--gif"));
        assert!(justfile.contains("watch:"));
    }

    #[test]
    fn test_init_animator_animation_valid() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("anim");

        init_project(&project_path, "anim", "animator").unwrap();

        // Check animation file has include and animation definition
        let anim = fs::read_to_string(project_path.join("src/pxl/animations/walk.pxl")).unwrap();
        assert!(anim.contains("\"type\": \"include\""));
        assert!(anim.contains("\"type\": \"animation\""));
        assert!(anim.contains("walk_cycle"));
    }

    // ========================================================================
    // Game Preset Tests
    // ========================================================================

    #[test]
    fn test_init_game_creates_structure() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("game-project");

        init_project(&project_path, "game-project", "game").unwrap();

        // Check directories
        assert!(project_path.join("src/pxl/palettes").exists());
        assert!(project_path.join("src/pxl/sprites/player").exists());
        assert!(project_path.join("src/pxl/sprites/items").exists());
        assert!(project_path.join("src/pxl/animations").exists());
        assert!(project_path.join("src/pxl/ui").exists());
        assert!(project_path.join("build/atlases").exists());
        assert!(project_path.join("build/preview").exists());

        // Check files
        assert!(project_path.join("pxl.toml").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join("justfile").exists());
        assert!(project_path.join("src/pxl/palettes/main.pxl").exists());
        assert!(project_path.join("src/pxl/palettes/ui.pxl").exists());
        assert!(project_path.join("src/pxl/sprites/player/idle.pxl").exists());
        assert!(project_path.join("src/pxl/sprites/items/coin.pxl").exists());
        assert!(project_path.join("src/pxl/animations/player.pxl").exists());
        assert!(project_path.join("src/pxl/ui/button.pxl").exists());
    }

    #[test]
    fn test_init_game_config_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("game");

        init_project(&project_path, "game", "game").unwrap();

        let config = fs::read_to_string(project_path.join("pxl.toml")).unwrap();
        assert!(config.contains("name = \"game\""));
        assert!(config.contains("[atlases.sprites]"));
        assert!(config.contains("[atlases.ui]"));
        assert!(config.contains("[animations]"));
        assert!(config.contains("[export.generic]"));
        assert!(config.contains("[validate]"));
        assert!(config.contains("strict = true"));
    }

    #[test]
    fn test_init_game_justfile_content() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("game");

        init_project(&project_path, "game", "game").unwrap();

        let justfile = fs::read_to_string(project_path.join("justfile")).unwrap();
        assert!(justfile.contains("build: validate"));
        assert!(justfile.contains("atlas-sprites:"));
        assert!(justfile.contains("atlas-ui:"));
        assert!(justfile.contains("ci: fmt-check validate build"));
    }

    #[test]
    fn test_init_game_sprites_valid_json() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("game");

        init_project(&project_path, "game", "game").unwrap();

        // Check player sprite
        let player =
            fs::read_to_string(project_path.join("src/pxl/sprites/player/idle.pxl")).unwrap();
        assert!(player.contains("\"type\": \"sprite\""));
        assert!(player.contains("player_idle_1"));
        assert!(player.contains("player_idle_2"));

        // Check item sprite (has embedded palette)
        let coin = fs::read_to_string(project_path.join("src/pxl/sprites/items/coin.pxl")).unwrap();
        assert!(coin.contains("\"type\": \"palette\""));
        assert!(coin.contains("\"type\": \"sprite\""));
        assert!(coin.contains("coin"));
    }

    #[test]
    fn test_init_game_ui_valid_json() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("game");

        init_project(&project_path, "game", "game").unwrap();

        let ui = fs::read_to_string(project_path.join("src/pxl/ui/button.pxl")).unwrap();
        assert!(ui.contains("button_normal"));
        assert!(ui.contains("button_hover"));
    }
}
