//! Asset scaffolding for pixelsrc projects
//!
//! Provides templates for creating new sprites, animations, and palettes.

use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::config::loader::find_config;

/// Error during asset scaffolding
#[derive(Debug, Error)]
#[non_exhaustive]
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

// ============================================================================
// Scaffold generators (for `pxl scaffold` CLI)
// ============================================================================

/// Generate a complete sprite scaffold with palette and grid.
///
/// Produces a self-contained .pxl string with a palette definition followed by
/// a sprite definition with an all-transparent grid.
pub fn generate_sprite(
    name: &str,
    width: u32,
    height: u32,
    palette_name: Option<&str>,
    tokens: &[String],
) -> String {
    let pal_name = palette_name
        .map(|p| format!("{}_palette", p))
        .unwrap_or_else(|| format!("{}_palette", name));

    // Build palette colors
    let mut colors = Vec::new();
    colors.push(("_".to_string(), "#00000000".to_string()));

    if tokens.is_empty() {
        // Minimal palette with just transparency
    } else {
        // Generate colors spread across the hue wheel
        for (i, token) in tokens.iter().enumerate() {
            let hue = (i as f64 / tokens.len() as f64) * 360.0;
            let color = hsl_to_hex(hue, 0.7, 0.5);
            colors.push((token.clone(), color));
        }
    }

    // Build palette JSON
    let color_entries: Vec<String> = colors
        .iter()
        .map(|(tok, hex)| format!("    \"{{{}}}\": \"{}\"", tok, hex))
        .collect();

    let palette_json = format!(
        "{{\n  \"type\": \"palette\",\n  \"name\": \"{}\",\n  \"colors\": {{\n{}\n  }}\n}}",
        pal_name,
        color_entries.join(",\n")
    );

    // Build grid (all transparent)
    let row = format!("\"{}\"", "{_}".repeat(width as usize));
    let rows: Vec<String> = (0..height).map(|_| format!("    {}", row)).collect();

    let sprite_json = format!(
        "{{\n  \"type\": \"sprite\",\n  \"name\": \"{}\",\n  \"size\": [{}, {}],\n  \"palette\": \"{}\",\n  \"grid\": [\n{}\n  ]\n}}",
        name, width, height, pal_name,
        rows.join(",\n")
    );

    format!("{}\n\n{}\n", palette_json, sprite_json)
}

/// Generate a composition scaffold with tile sprites.
///
/// Returns an error if the number of required tiles exceeds 62 (A-Z + a-z + 0-9).
pub fn generate_composition(
    name: &str,
    total_w: u32,
    total_h: u32,
    cell_w: u32,
    cell_h: u32,
    palette_name: Option<&str>,
) -> Result<String, String> {
    let cols = total_w / cell_w;
    let rows = total_h / cell_h;
    let tile_count = cols * rows;

    if tile_count > 62 {
        return Err(format!(
            "composition requires {} tiles but maximum is 62 (A-Z + a-z + 0-9)",
            tile_count
        ));
    }

    let pal_name = palette_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}_palette", name));

    // Symbol assignment: A-Z, a-z, 0-9
    let symbols: Vec<char> = ('A'..='Z').chain('a'..='z').chain('0'..='9').collect();

    let mut output = String::new();

    // Palette
    output.push_str(&format!(
        "{{\n  \"type\": \"palette\",\n  \"name\": \"{}\",\n  \"colors\": {{\n    \"{{_}}\": \"#00000000\"\n  }}\n}}\n",
        pal_name
    ));

    // Generate tile sprites
    let transparent_row = format!("\"{}\"", "{_}".repeat(cell_w as usize));
    let grid_rows: Vec<String> = (0..cell_h)
        .map(|_| format!("    {}", transparent_row))
        .collect();
    let grid_str = grid_rows.join(",\n");

    let mut sprite_map = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let idx = (row * cols + col) as usize;
            let symbol = symbols[idx];
            let tile_name = format!("tile_{}_{}", col, row);

            output.push_str(&format!(
                "\n{{\n  \"type\": \"sprite\",\n  \"name\": \"{}\",\n  \"size\": [{}, {}],\n  \"palette\": \"{}\",\n  \"grid\": [\n{}\n  ]\n}}\n",
                tile_name, cell_w, cell_h, pal_name, grid_str
            ));

            sprite_map.push((symbol, tile_name));
        }
    }

    // Build sprites map entries
    let sprite_entries: Vec<String> = sprite_map
        .iter()
        .map(|(sym, name)| format!("    \"{}\": \"{}\"", sym, name))
        .collect();

    // Build character map
    let mut map_rows = Vec::new();
    for row in 0..rows {
        let mut map_row = String::new();
        for col in 0..cols {
            let idx = (row * cols + col) as usize;
            map_row.push(symbols[idx]);
        }
        map_rows.push(format!("      \"{}\"", map_row));
    }

    output.push_str(&format!(
        "\n{{\n  \"type\": \"composition\",\n  \"name\": \"{}\",\n  \"size\": [{}, {}],\n  \"cell_size\": [{}, {}],\n  \"sprites\": {{\n{}\n  }},\n  \"layers\": [{{\n    \"map\": [\n{}\n    ]\n  }}]\n}}\n",
        name, total_w, total_h, cell_w, cell_h,
        sprite_entries.join(",\n"),
        map_rows.join(",\n")
    ));

    Ok(output)
}

/// Generate a palette scaffold from a preset name or color list.
pub fn generate_palette_scaffold(
    name: &str,
    preset: Option<&str>,
    colors: Option<&str>,
    token_prefix: &str,
) -> Result<String, String> {
    let mut color_map: Vec<(String, String)> = Vec::new();
    color_map.push(("_".to_string(), "#00000000".to_string()));

    if let Some(preset_name) = preset {
        let preset_colors = get_preset_colors(preset_name)?;
        color_map.extend(preset_colors);
    } else if let Some(color_list) = colors {
        let hex_colors: Vec<&str> = color_list.split(',').map(|s| s.trim()).collect();
        for (i, hex) in hex_colors.iter().enumerate() {
            // Validate hex color
            if !hex.starts_with('#') || (hex.len() != 7 && hex.len() != 9) {
                return Err(format!("invalid hex color '{}', expected #RRGGBB or #RRGGBBAA", hex));
            }
            let token = format!("{}{}", token_prefix, i + 1);
            color_map.push((token, hex.to_string()));
        }
    }

    let color_entries: Vec<String> = color_map
        .iter()
        .map(|(tok, hex)| format!("    \"{{{}}}\": \"{}\"", tok, hex))
        .collect();

    let output = format!(
        "{{\n  \"type\": \"palette\",\n  \"name\": \"{}\",\n  \"colors\": {{\n{}\n  }}\n}}\n",
        name,
        color_entries.join(",\n")
    );

    Ok(output)
}

/// Get colors for a built-in palette preset.
fn get_preset_colors(name: &str) -> Result<Vec<(String, String)>, String> {
    match name {
        "forest" => Ok(vec![
            ("trunk".into(), "#8B4513".into()),
            ("bark".into(), "#654321".into()),
            ("leaf".into(), "#228B22".into()),
            ("moss".into(), "#6B8E23".into()),
            ("sky".into(), "#87CEEB".into()),
            ("earth".into(), "#8B7355".into()),
        ]),
        "medieval" => Ok(vec![
            ("stone".into(), "#808080".into()),
            ("iron".into(), "#434343".into()),
            ("gold".into(), "#FFD700".into()),
            ("leather".into(), "#8B4513".into()),
            ("cloth".into(), "#800020".into()),
            ("skin".into(), "#FFCC99".into()),
        ]),
        "synthwave" => Ok(vec![
            ("neon_pink".into(), "#FF1493".into()),
            ("neon_blue".into(), "#00BFFF".into()),
            ("purple".into(), "#9400D3".into()),
            ("dark_bg".into(), "#1A0033".into()),
            ("grid".into(), "#FF00FF".into()),
            ("sun".into(), "#FF6347".into()),
        ]),
        "ocean" => Ok(vec![
            ("deep".into(), "#003366".into()),
            ("water".into(), "#0077BE".into()),
            ("wave".into(), "#00CED1".into()),
            ("foam".into(), "#F0F8FF".into()),
            ("sand".into(), "#F4A460".into()),
            ("coral".into(), "#FF7F50".into()),
        ]),
        _ => {
            Err(format!(
                "unknown preset '{}'. Available presets: forest, medieval, synthwave, ocean",
                name
            ))
        }
    }
}

/// Convert HSL to hex color string.
fn hsl_to_hex(hue: f64, saturation: f64, lightness: f64) -> String {
    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = lightness - c / 2.0;

    let (r, g, b) = match hue as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r + m) * 255.0).round() as u8;
    let g = ((g + m) * 255.0).round() as u8;
    let b = ((b + m) * 255.0).round() as u8;

    format!("#{:02X}{:02X}{:02X}", r, g, b)
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
