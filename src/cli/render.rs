//! Render command implementation and helpers

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::antialias::{AAAlgorithm, AnchorMode};
use crate::atlas::{add_animation_to_atlas, pack_atlas, AtlasBox, AtlasConfig, SpriteInput};
use crate::build::project_registry::ProjectRegistry;
use crate::composition::render_composition;
use crate::config::loader::{find_config_from, load_config};
use crate::gif::render_gif;
use crate::include::{is_include_ref, parse_include_ref, resolve_include_with_detection};
use crate::models::{Animation, Composition, PaletteRef, Sprite, TtpObject};
use crate::output::{generate_output_path, save_png, scale_image};
use crate::palette_cycle::{generate_cycle_frames, get_cycle_duration};
use crate::parser::parse_stream;
use crate::registry::{PaletteRegistry, PaletteSource, ResolvedPalette, SpriteRegistry};
use crate::renderer::{render_resolved, render_sprite};
use crate::spritesheet::render_spritesheet;
use crate::suggest::{format_suggestion, suggest};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the render command
#[allow(clippy::too_many_arguments)]
pub fn run_render(
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
    nine_slice_arg: Option<&str>,
    _antialias: Option<AAAlgorithm>,
    _aa_strength: f32,
    _anchor_mode: AnchorMode,
    _no_semantic_aa: bool,
    _gradient_shadows: bool,
    no_project: bool,
) -> ExitCode {
    // Parse nine-slice target size if provided
    let nine_slice_size = if let Some(size_str) = nine_slice_arg {
        let parts: Vec<&str> = size_str.split('x').collect();
        if parts.len() != 2 {
            eprintln!(
                "Error: Invalid nine-slice size format '{}'. Use WxH format (e.g., '64x32')",
                size_str
            );
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        match (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            (Ok(w), Ok(h)) if w > 0 && h > 0 => Some((w, h)),
            _ => {
                eprintln!(
                    "Error: Invalid nine-slice size '{}'. Width and height must be positive integers",
                    size_str
                );
                return ExitCode::from(EXIT_INVALID_ARGS);
            }
        }
    } else {
        None
    };

    // Auto-detect project context from pxl.toml in parent directories
    let project_registry = if no_project {
        None
    } else {
        // Canonicalize the input path so parent-dir walking works from the file's real location
        let input_abs = std::fs::canonicalize(input).unwrap_or_else(|_| input.clone());
        let start_dir = input_abs.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        if let Some(config_path) = find_config_from(start_dir) {
            match load_config(Some(&config_path)) {
                Ok(config) => {
                    let project_root = config_path.parent().unwrap();
                    let src_dir = project_root.join(&config.project.src);
                    if src_dir.exists() {
                        let mut registry =
                            ProjectRegistry::new(config.project.name.clone(), src_dir);
                        match registry.load_all(strict) {
                            Ok(()) => {
                                for warning in registry.warnings() {
                                    eprintln!("Warning: {}", warning.message);
                                }
                                Some(registry)
                            }
                            Err(e) => {
                                if strict {
                                    eprintln!("Error: Failed to load project registry: {}", e);
                                    return ExitCode::from(EXIT_ERROR);
                                }
                                eprintln!("Warning: Failed to load project registry: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                Err(e) => {
                    if strict {
                        eprintln!("Error: Failed to load pxl.toml: {}", e);
                        return ExitCode::from(EXIT_ERROR);
                    }
                    eprintln!("Warning: Failed to load pxl.toml: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

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

    // Build local palette registry and sprite registry, and collect sprites, animations, and compositions
    let mut local_palette_registry = PaletteRegistry::new();
    let mut local_sprite_registry = SpriteRegistry::new();
    let mut sprites_by_name: HashMap<String, Sprite> = HashMap::new();
    let mut animations_by_name: HashMap<String, Animation> = HashMap::new();
    let mut compositions_by_name: HashMap<String, Composition> = HashMap::new();

    for obj in parse_result.objects {
        match obj {
            TtpObject::Palette(palette) => {
                local_palette_registry.register(palette);
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
                local_sprite_registry.register_sprite(sprite.clone());
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
                local_sprite_registry.register_variant(variant);
            }
            TtpObject::Particle(_) => {
                // Particle systems are runtime constructs, not rendered statically
            }
            TtpObject::Transform(_) => {
                // User-defined transforms are stored in transform registry
                // (future: register in transform_registry)
            }
            TtpObject::StateRules(_) => {
                // State rules are runtime styling, applied during rendering
            }
            TtpObject::Import(_) => {
                // Import declarations are resolved during loading
            }
        }
    }

    // Select registries: project-wide (two-pass) or file-local (single-pass)
    let registry =
        project_registry.as_ref().map(|r| &r.palettes).unwrap_or(&local_palette_registry);
    let sprite_registry =
        project_registry.as_ref().map(|r| &r.sprites).unwrap_or(&local_sprite_registry);

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
            &compositions_by_name,
            sprite_registry,
            registry,
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
                sprite_registry,
                registry,
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
            sprite_registry,
            registry,
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
            let final_palette = if uses_include_palette {
                // Handle @include: palette specially
                let (include_path, palette_name) = if let PaletteRef::Named(name) = &sprite.palette
                {
                    parse_include_ref(name).expect("is_include_ref validated prefix")
                } else {
                    unreachable!()
                };

                match resolve_include_with_detection(
                    include_path,
                    input_dir,
                    &mut include_visited,
                    palette_name,
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
                }
            } else {
                // Normal path: sprite_registry handles source resolution and palette
                let resolved = match sprite_registry.resolve(&sprite.name, registry, strict) {
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
                resolved.palette
            };

            // Get regions from resolved source if sprite has a source reference
            // This is critical for derived sprites that reference a regions-based source
            let resolved_regions = if sprite.source.is_some() {
                // Need to re-resolve to get the regions (palette was already extracted above)
                match sprite_registry.resolve(&sprite.name, registry, false) {
                    Ok(r) => r.regions,
                    Err(_) => sprite.regions.clone(),
                }
            } else {
                sprite.regions.clone()
            };

            // Create resolved sprite for rendering with correct regions
            let render_sprite_data = crate::registry::ResolvedSprite {
                name: sprite.name.clone(),
                size: sprite.size.or_else(|| {
                    // For derived sprites, get size from resolved source
                    if sprite.source.is_some() {
                        sprite_registry
                            .resolve(&sprite.name, registry, false)
                            .ok()
                            .and_then(|r| r.size)
                    } else {
                        None
                    }
                }),
                palette: final_palette.clone(),
                warnings: vec![],
                nine_slice: sprite.nine_slice.clone(),
                regions: resolved_regions,
            };

            // Render the resolved sprite
            let (mut image, render_warnings) = render_resolved(&render_sprite_data);

            // Apply transforms from sprite.transform if present
            if let Some(ref transform_specs) = sprite.transform {
                use crate::models::TransformSpec;
                use crate::transforms::{apply_image_transform, parse_transform_str};

                for spec in transform_specs {
                    let transform_result = match spec {
                        TransformSpec::String(s) => parse_transform_str(s),
                        TransformSpec::Object { op, params } => {
                            // Convert object to JSON and parse
                            let mut obj = serde_json::Map::new();
                            obj.insert("op".to_string(), serde_json::Value::String(op.clone()));
                            for (k, v) in params {
                                obj.insert(k.clone(), v.clone());
                            }
                            crate::transforms::parse_transform_value(&serde_json::Value::Object(
                                obj,
                            ))
                        }
                    };

                    match transform_result {
                        Ok(transform) => {
                            // Skip animation transforms (they don't apply to images)
                            if crate::transforms::is_animation_transform(&transform) {
                                continue;
                            }
                            match apply_image_transform(&image, &transform, Some(&final_palette)) {
                                Ok(transformed) => image = transformed,
                                Err(e) => {
                                    let msg =
                                        format!("sprite '{}': transform error: {}", sprite.name, e);
                                    if strict {
                                        eprintln!("Error: {}", msg);
                                        return ExitCode::from(EXIT_ERROR);
                                    }
                                    all_warnings.push(msg);
                                }
                            }
                        }
                        Err(e) => {
                            let msg = format!("sprite '{}': invalid transform: {}", sprite.name, e);
                            if strict {
                                eprintln!("Error: {}", msg);
                                return ExitCode::from(EXIT_ERROR);
                            }
                            all_warnings.push(msg);
                        }
                    }
                }
            }

            // Apply nine-slice rendering if requested
            if let Some((target_w, target_h)) = nine_slice_size {
                if let Some(ref nine_slice) = sprite.nine_slice {
                    let (ns_image, ns_warnings) =
                        crate::renderer::render_nine_slice(&image, nine_slice, target_w, target_h);
                    image = ns_image;
                    for warning in ns_warnings {
                        all_warnings.push(format!("sprite '{}': {}", sprite.name, warning.message));
                    }
                } else {
                    eprintln!(
                        "Warning: --nine-slice specified but sprite '{}' has no nine_slice attribute",
                        sprite.name
                    );
                }
            }

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
                sprite_registry,
                registry,
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
    compositions: &HashMap<String, Composition>,
    sprites: &HashMap<String, Sprite>,
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
    sprites: &HashMap<String, Sprite>,
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
    let mut rendered_sprites: HashMap<String, RgbaImage> = HashMap::new();

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
                        let (include_path, palette_name) =
                            parse_include_ref(name).expect("is_include_ref validated prefix");
                        match resolve_include_with_detection(
                            include_path,
                            input_dir,
                            include_visited,
                            palette_name,
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
            palette: final_palette,
            warnings: vec![],
            nine_slice: resolved_sprite.nine_slice.clone(),
            regions: resolved_sprite.regions.clone(),
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
// TTP-9qjwr: Added compositions parameter to support compositions as animation frames
#[allow(clippy::too_many_arguments)]
fn run_animation_render(
    input: &std::path::Path,
    output: Option<&std::path::Path>,
    animations: &HashMap<String, Animation>,
    sprites: &HashMap<String, Sprite>,
    compositions: &HashMap<String, Composition>,
    sprite_registry: &SpriteRegistry,
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

    // Validate animation: check that all frame references exist (sprites OR compositions)
    // TTP-9qjwr: Now also checks compositions as valid frame references
    let mut missing_frames = Vec::new();
    for frame_name in &animation.frames {
        if !sprites.contains_key(frame_name) && !compositions.contains_key(frame_name) {
            missing_frames.push(frame_name.clone());
        }
    }

    if !missing_frames.is_empty() {
        let warning_msg = format!(
            "Animation '{}' references missing sprites/compositions: {}",
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
                let (include_path, palette_name) =
                    parse_include_ref(name).expect("is_include_ref validated prefix");
                match resolve_include_with_detection(
                    include_path,
                    input_dir,
                    include_visited,
                    palette_name,
                ) {
                    Ok(palette) => ResolvedPalette {
                        colors: palette.colors,
                        source: PaletteSource::Named(name.clone()),
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
        // TTP-9qjwr: Now supports both sprites and compositions as frames
        let mut frame_images = Vec::new();
        for frame_name in &animation.frames {
            // First try to get as sprite
            if let Some(sprite) = sprites.get(frame_name) {
                // Resolve palette
                let resolved = match &sprite.palette {
                    PaletteRef::Named(name) if is_include_ref(name) => {
                        let (include_path, palette_name) =
                            parse_include_ref(name).expect("is_include_ref validated prefix");
                        match resolve_include_with_detection(
                            include_path,
                            input_dir,
                            include_visited,
                            palette_name,
                        ) {
                            Ok(palette) => ResolvedPalette {
                                colors: palette.colors,
                                source: PaletteSource::Named(name.clone()),
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
            } else if let Some(comp) = compositions.get(frame_name) {
                // TTP-9qjwr: Render composition as animation frame
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

                match result {
                    Ok(image) => {
                        // Apply scaling if requested
                        let image = scale_image(image, scale);
                        frame_images.push(image);
                    }
                    Err(code) => return code,
                }
            }
            // If neither sprite nor composition found, skip (warned above)
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
    sprites: &HashMap<String, Sprite>,
    animations: &HashMap<String, Animation>,
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
                let (include_path, palette_name) =
                    parse_include_ref(name).expect("is_include_ref validated prefix");
                match resolve_include_with_detection(
                    include_path,
                    input_dir,
                    include_visited,
                    palette_name,
                ) {
                    Ok(palette) => ResolvedPalette {
                        colors: palette.colors,
                        source: PaletteSource::Named(name.clone()),
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
            "atlas" => serde_json::to_string_pretty(&metadata).expect("metadata serialization"),
            "atlas-aseprite" => generate_aseprite_json(&metadata),
            "atlas-godot" => generate_godot_json(&metadata),
            "atlas-unity" => generate_unity_json(&metadata),
            "atlas-libgdx" => generate_libgdx_atlas(&metadata),
            _ => serde_json::to_string_pretty(&metadata).expect("metadata serialization"),
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
    .expect("JSON value serialization")
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
    .expect("JSON value serialization")
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
    .expect("JSON value serialization")
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
