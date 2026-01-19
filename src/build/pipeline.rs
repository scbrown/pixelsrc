//! Build pipeline orchestration.
//!
//! The pipeline coordinates the execution of build targets in the correct order.

use crate::atlas::{pack_atlas, AtlasBox, AtlasConfig as PackerConfig, SpriteInput};
use crate::build::{BuildContext, BuildPlan, BuildResult, BuildTarget, TargetKind, TargetResult};
use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::registry::PaletteRegistry;
use crate::renderer::render_sprite;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;

/// Error during build execution.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// Discovery error
    #[error("Discovery error: {0}")]
    Discovery(#[from] crate::build::DiscoveryError),
    /// Build order error (circular dependencies)
    #[error("Build order error: {0}")]
    BuildOrder(#[from] crate::build::target::BuildOrderError),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Generic build error
    #[error("Build error: {0}")]
    Build(String),
}

/// Build pipeline for executing builds.
pub struct BuildPipeline {
    /// Build context
    context: BuildContext,
    /// Whether to stop on first error
    fail_fast: bool,
    /// Whether to do a dry run (don't actually build)
    dry_run: bool,
}

impl BuildPipeline {
    /// Create a new build pipeline.
    pub fn new(context: BuildContext) -> Self {
        Self { context, fail_fast: false, dry_run: false }
    }

    /// Set fail-fast mode (stop on first error).
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set dry-run mode (don't actually build).
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Run the build pipeline.
    ///
    /// Discovers sources, creates a build plan, and executes it.
    pub fn build(&self) -> Result<BuildResult, BuildError> {
        let start = Instant::now();

        // Create build plan from config
        let plan = crate::build::create_build_plan(&self.context)?;

        // Apply target filter if specified
        let plan = if let Some(filter) = self.context.target_filter() {
            plan.filter(filter)
        } else {
            plan
        };

        // Execute the plan
        let mut result = self.execute_plan(&plan)?;
        result.total_duration = start.elapsed();

        Ok(result)
    }

    /// Run the build pipeline with a pre-created plan.
    pub fn build_plan(&self, plan: &BuildPlan) -> Result<BuildResult, BuildError> {
        let start = Instant::now();
        let mut result = self.execute_plan(plan)?;
        result.total_duration = start.elapsed();
        Ok(result)
    }

    /// Execute a build plan.
    fn execute_plan(&self, plan: &BuildPlan) -> Result<BuildResult, BuildError> {
        let mut result = BuildResult::new();

        // Get targets in build order
        let ordered = plan.build_order()?;

        if self.context.is_verbose() {
            println!("Build plan: {} targets", ordered.len());
            for target in &ordered {
                println!("  - {} ({})", target.id, target.kind);
            }
        }

        // Ensure output directory exists
        if !self.dry_run {
            fs::create_dir_all(self.context.out_dir())?;
        }

        // Execute each target
        for target in ordered {
            let target_result = self.execute_target(target);

            if target_result.status.is_failure() && self.fail_fast {
                result.add_result(target_result);
                return Ok(result);
            }

            result.add_result(target_result);
        }

        Ok(result)
    }

    /// Execute a single build target.
    fn execute_target(&self, target: &BuildTarget) -> TargetResult {
        let start = Instant::now();

        if self.context.is_verbose() {
            println!("Building: {} ...", target.id);
        }

        if self.dry_run {
            return TargetResult::skipped(target.id.clone());
        }

        // Ensure parent directory exists for output
        if let Some(parent) = target.output.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return TargetResult::failed(
                    target.id.clone(),
                    format!("Failed to create output directory: {}", e),
                    start.elapsed(),
                );
            }
        }

        // Execute based on target kind
        let build_result = match target.kind {
            TargetKind::Sprite => self.build_sprite(target),
            TargetKind::Atlas => self.build_atlas(target),
            TargetKind::Animation => self.build_animation(target),
            TargetKind::AnimationPreview => self.build_animation_preview(target),
            TargetKind::Export => self.build_export(target),
        };

        let duration = start.elapsed();

        match build_result {
            Ok(outputs) => {
                if self.context.is_verbose() {
                    println!("  Done in {:?}", duration);
                }
                TargetResult::success(target.id.clone(), outputs, duration)
            }
            Err(e) => {
                if self.context.is_verbose() {
                    println!("  Failed: {}", e);
                }
                TargetResult::failed(target.id.clone(), e, duration)
            }
        }
    }

    /// Build a sprite target.
    ///
    /// Parses the source .pxl file, resolves palettes, renders the sprite,
    /// and saves it as a PNG file.
    fn build_sprite(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        // Validate sources exist
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }

        // Get the source file (sprites have exactly one source)
        let source = target
            .sources
            .first()
            .ok_or_else(|| "No source file specified for sprite target".to_string())?;

        // Parse the source file
        let file = File::open(source)
            .map_err(|e| format!("Failed to open {}: {}", source.display(), e))?;
        let reader = BufReader::new(file);
        let parse_result = parse_stream(reader);

        // Check for parse warnings (these may indicate problems)
        if !parse_result.warnings.is_empty() && self.context.is_strict() {
            let warnings: Vec<String> =
                parse_result.warnings.iter().map(|w| w.message.clone()).collect();
            return Err(format!("Parse warnings in {}: {}", source.display(), warnings.join("; ")));
        }

        // Build palette registry from parsed objects
        let mut registry = PaletteRegistry::new();
        let mut sprites = Vec::new();
        for obj in parse_result.objects {
            match obj {
                TtpObject::Palette(p) => {
                    registry.register(p);
                }
                TtpObject::Sprite(s) => {
                    sprites.push(s);
                }
                _ => {
                    // Ignore other object types for now (animations, compositions, etc.)
                }
            }
        }

        // Find the sprite to render (use the target name or first sprite)
        let sprite = if sprites.len() == 1 {
            sprites.into_iter().next().unwrap()
        } else {
            // Try to find sprite by target name
            sprites.into_iter().find(|s| s.name == target.name).ok_or_else(|| {
                format!("Sprite '{}' not found in {}", target.name, source.display())
            })?
        };

        // Resolve the palette for this sprite
        let resolved = if self.context.is_strict() {
            registry
                .resolve_strict(&sprite)
                .map_err(|e| format!("Failed to resolve palette for '{}': {}", sprite.name, e))?
        } else {
            let result = registry.resolve_lenient(&sprite);
            if let Some(warning) = result.warning {
                if self.context.is_verbose() {
                    eprintln!("Warning: {}", warning.message);
                }
            }
            result.palette
        };

        // Render the sprite
        let (image, render_warnings) = render_sprite(&sprite, &resolved.colors);

        // Handle render warnings
        if !render_warnings.is_empty() {
            if self.context.is_strict() {
                let warnings: Vec<String> =
                    render_warnings.iter().map(|w| w.message.clone()).collect();
                return Err(format!(
                    "Render warnings for '{}': {}",
                    sprite.name,
                    warnings.join("; ")
                ));
            } else if self.context.is_verbose() {
                for warning in &render_warnings {
                    eprintln!("Warning: sprite '{}': {}", sprite.name, warning.message);
                }
            }
        }

        // Apply scale if configured
        let scale = self.context.default_scale();
        let final_image = if scale > 1 {
            image::imageops::resize(
                &image,
                image.width() * scale,
                image.height() * scale,
                image::imageops::FilterType::Nearest,
            )
        } else {
            image
        };

        // Save the PNG
        final_image
            .save(&target.output)
            .map_err(|e| format!("Failed to save {}: {}", target.output.display(), e))?;

        Ok(vec![target.output.clone()])
    }

    /// Build an atlas target.
    ///
    /// Parses all source files, renders sprites in parallel, packs them into a
    /// texture atlas, and saves the atlas image and metadata JSON.
    fn build_atlas(&self, target: &BuildTarget) -> Result<Vec<std::path::PathBuf>, String> {
        // Validate sources exist
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }

        // Look up the atlas configuration
        let atlas_config = self
            .context
            .config()
            .atlases
            .get(&target.name)
            .ok_or_else(|| format!("Atlas config '{}' not found", target.name))?;

        // Create packer config from atlas config
        let padding = self.context.config().effective_padding(atlas_config);
        let packer_config = PackerConfig {
            max_size: (atlas_config.max_size[0], atlas_config.max_size[1]),
            padding,
            power_of_two: atlas_config.power_of_two,
        };

        let scale = self.context.default_scale();
        let is_strict = self.context.is_strict();
        let is_verbose = self.context.is_verbose();
        let multi_source = target.sources.len() > 1;

        // Phase 1: Parse all source files and collect render tasks
        // Each render task contains: (sprite, resolved_palette, qualified_name)
        struct RenderTask {
            sprite: crate::models::Sprite,
            colors: HashMap<String, String>,
            qualified_name: String,
        }

        let mut render_tasks: Vec<RenderTask> = Vec::new();

        for source in &target.sources {
            // Parse the source file
            let file = File::open(source)
                .map_err(|e| format!("Failed to open {}: {}", source.display(), e))?;
            let reader = BufReader::new(file);
            let parse_result = parse_stream(reader);

            // Build palette registry and collect sprites
            let mut registry = PaletteRegistry::new();
            let mut sprites = Vec::new();

            for obj in parse_result.objects {
                match obj {
                    TtpObject::Palette(p) => {
                        registry.register(p);
                    }
                    TtpObject::Sprite(s) => {
                        sprites.push(s);
                    }
                    _ => {
                        // Ignore animations, compositions, etc. for atlas building
                    }
                }
            }

            // Create render tasks for each sprite
            let file_stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

            for sprite in sprites {
                // Resolve palette for sprite
                let resolved = if is_strict {
                    registry.resolve_strict(&sprite).map_err(|e| {
                        format!(
                            "Failed to resolve palette for '{}' in {}: {}",
                            sprite.name,
                            source.display(),
                            e
                        )
                    })?
                } else {
                    let result = registry.resolve_lenient(&sprite);
                    if let Some(warning) = result.warning {
                        if is_verbose {
                            eprintln!("Warning: {}", warning.message);
                        }
                    }
                    result.palette
                };

                let qualified_name = if multi_source {
                    format!("{}:{}", file_stem, sprite.name)
                } else {
                    sprite.name.clone()
                };

                render_tasks.push(RenderTask {
                    sprite,
                    colors: resolved.colors.clone(),
                    qualified_name,
                });
            }
        }

        // Phase 2: Render sprites in parallel using Rayon
        let render_results: Vec<Result<SpriteInput, String>> = render_tasks
            .into_par_iter()
            .map(|task| {
                // Render the sprite
                let (image, render_warnings) = render_sprite(&task.sprite, &task.colors);

                // Handle render warnings
                if !render_warnings.is_empty() {
                    if is_strict {
                        let warnings: Vec<String> =
                            render_warnings.iter().map(|w| w.message.clone()).collect();
                        return Err(format!(
                            "Render warnings for '{}': {}",
                            task.sprite.name,
                            warnings.join("; ")
                        ));
                    } else if is_verbose {
                        for warning in &render_warnings {
                            eprintln!(
                                "Warning: sprite '{}': {}",
                                task.sprite.name, warning.message
                            );
                        }
                    }
                }

                // Apply scale if configured
                let final_image = if scale > 1 {
                    image::imageops::resize(
                        &image,
                        image.width() * scale,
                        image.height() * scale,
                        image::imageops::FilterType::Nearest,
                    )
                } else {
                    image
                };

                // Extract metadata (origin and boxes)
                let origin = task.sprite.metadata.as_ref().and_then(|m| m.origin);
                let boxes = task.sprite.metadata.as_ref().and_then(|m| {
                    m.boxes.as_ref().map(|b| {
                        b.iter()
                            .map(|(name, cb)| {
                                (name.clone(), AtlasBox { x: cb.x, y: cb.y, w: cb.w, h: cb.h })
                            })
                            .collect::<HashMap<_, _>>()
                    })
                });

                Ok(SpriteInput { name: task.qualified_name, image: final_image, origin, boxes })
            })
            .collect();

        // Collect results, propagating any errors
        let sprite_inputs: Vec<SpriteInput> =
            render_results.into_iter().collect::<Result<Vec<_>, _>>()?;

        if sprite_inputs.is_empty() {
            return Err(format!("No sprites found in source files for atlas '{}'", target.name));
        }

        // Pack sprites into atlas
        let base_name = target.output.file_stem().and_then(|s| s.to_str()).unwrap_or(&target.name);
        let result = pack_atlas(&sprite_inputs, &packer_config, base_name);

        if result.atlases.is_empty() {
            return Err("Failed to pack any sprites into atlas".to_string());
        }

        // Save atlas images and metadata
        let mut outputs = Vec::new();
        let out_dir = target.output.parent().unwrap_or_else(|| std::path::Path::new("."));

        for (image, metadata) in &result.atlases {
            // Save the PNG
            let png_path = out_dir.join(&metadata.image);
            image
                .save(&png_path)
                .map_err(|e| format!("Failed to save atlas PNG {}: {}", png_path.display(), e))?;
            outputs.push(png_path);

            // Save the JSON metadata
            let json_name = metadata.image.replace(".png", ".json");
            let json_path = out_dir.join(&json_name);
            let json_content = serde_json::to_string_pretty(&metadata)
                .map_err(|e| format!("Failed to serialize atlas metadata: {}", e))?;
            fs::write(&json_path, json_content).map_err(|e| {
                format!("Failed to write atlas JSON {}: {}", json_path.display(), e)
            })?;
            outputs.push(json_path);
        }

        Ok(outputs)
    }

    /// Build an animation target.
    fn build_animation(&self, target: &BuildTarget) -> Result<Vec<std::path::PathBuf>, String> {
        // Animation building will be implemented by downstream tasks
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
    }

    /// Build an animation preview target.
    fn build_animation_preview(
        &self,
        target: &BuildTarget,
    ) -> Result<Vec<std::path::PathBuf>, String> {
        // Preview building will be implemented by downstream tasks
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
    }

    /// Build an export target.
    ///
    /// Export targets transform atlas metadata into game engine-specific formats.
    /// The target ID format is "export:{format}:{atlas_name}".
    fn build_export(&self, target: &BuildTarget) -> Result<Vec<std::path::PathBuf>, String> {
        use crate::atlas::AtlasMetadata;
        use crate::export::{
            godot::{GodotExportOptions, GodotExporter},
            libgdx::{LibGdxExportOptions, LibGdxExporter},
            unity::{UnityExportOptions, UnityExporter, UnityFilterMode},
        };

        // Parse format from target ID (export:format:name)
        let parts: Vec<&str> = target.id.split(':').collect();
        if parts.len() < 3 {
            return Err(format!("Invalid export target ID: {}", target.id));
        }
        let format = parts[1];
        let atlas_name = parts[2];

        // Find the atlas JSON metadata file
        let atlas_json_path = self.context.out_dir().join(format!("{}.json", atlas_name));
        if !atlas_json_path.exists() {
            return Err(format!(
                "Atlas metadata not found: {}. Build the atlas first.",
                atlas_json_path.display()
            ));
        }

        // Load the atlas metadata
        let json_content = fs::read_to_string(&atlas_json_path)
            .map_err(|e| format!("Failed to read atlas metadata: {}", e))?;
        let metadata: AtlasMetadata = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse atlas metadata: {}", e))?;

        // Ensure output directory exists
        if let Some(parent) = target.output.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        // Export based on format
        let outputs = match format {
            "godot" => {
                let config = &self.context.config().exports.godot;
                let exporter = GodotExporter::new()
                    .with_resource_path(&config.resource_path)
                    .with_sprite_frames(config.sprite_frames)
                    .with_animation_player(config.animation_player);

                let options = GodotExportOptions {
                    resource_path: config.resource_path.clone(),
                    sprite_frames: config.sprite_frames,
                    animation_player: config.animation_player,
                    atlas_textures: true,
                    ..Default::default()
                };

                // Godot exports to a directory, not a single file
                let output_dir =
                    target.output.parent().unwrap_or_else(|| std::path::Path::new("."));

                exporter
                    .export_godot(&metadata, output_dir, &options)
                    .map_err(|e| format!("Godot export failed: {}", e))?
            }
            "unity" => {
                let config = &self.context.config().exports.unity;
                let filter_mode = UnityFilterMode::from_config(&config.filter_mode);
                let exporter = UnityExporter::new()
                    .with_pixels_per_unit(config.pixels_per_unit)
                    .with_filter_mode(filter_mode);

                let options = UnityExportOptions {
                    pixels_per_unit: config.pixels_per_unit,
                    filter_mode,
                    ..Default::default()
                };

                exporter
                    .export_unity(&metadata, &target.output, &options)
                    .map_err(|e| format!("Unity export failed: {}", e))?;

                vec![target.output.clone()]
            }
            "libgdx" => {
                let exporter = LibGdxExporter::new();
                let options = LibGdxExportOptions::default();

                exporter
                    .export_libgdx(&metadata, &target.output, &options)
                    .map_err(|e| format!("libGDX export failed: {}", e))?;

                vec![target.output.clone()]
            }
            _ => {
                return Err(format!("Unknown export format: {}", format));
            }
        };

        Ok(outputs)
    }
}

/// Builder for configuring and running builds.
pub struct Build {
    context: Option<BuildContext>,
    fail_fast: bool,
    dry_run: bool,
    verbose: bool,
    strict: bool,
    filter: Option<Vec<String>>,
}

impl Build {
    /// Create a new build builder.
    pub fn new() -> Self {
        Self {
            context: None,
            fail_fast: false,
            dry_run: false,
            verbose: false,
            strict: false,
            filter: None,
        }
    }

    /// Set the build context.
    pub fn context(mut self, context: BuildContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Set fail-fast mode.
    pub fn fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set dry-run mode.
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set verbose mode.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set strict mode.
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set target filter.
    pub fn filter(mut self, targets: Vec<String>) -> Self {
        self.filter = Some(targets);
        self
    }

    /// Run the build.
    pub fn run(self) -> Result<BuildResult, BuildError> {
        let mut context = self
            .context
            .ok_or_else(|| BuildError::Build("No build context provided".to_string()))?;

        context = context.with_verbose(self.verbose).with_strict(self.strict);

        if let Some(filter) = self.filter {
            context = context.with_filter(filter);
        }

        BuildPipeline::new(context)
            .with_fail_fast(self.fail_fast)
            .with_dry_run(self.dry_run)
            .build()
    }
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_config;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_context() -> (TempDir, BuildContext) {
        let temp = TempDir::new().unwrap();
        let config = default_config();
        let ctx = BuildContext::new(config, temp.path().to_path_buf());

        // Create source directory
        let src_dir = temp.path().join("src/pxl");
        fs::create_dir_all(&src_dir).unwrap();

        (temp, ctx)
    }

    #[test]
    fn test_build_pipeline_new() {
        let (_temp, ctx) = create_test_context();
        let pipeline = BuildPipeline::new(ctx);
        assert!(!pipeline.fail_fast);
        assert!(!pipeline.dry_run);
    }

    #[test]
    fn test_build_pipeline_with_options() {
        let (_temp, ctx) = create_test_context();
        let pipeline = BuildPipeline::new(ctx).with_fail_fast(true).with_dry_run(true);

        assert!(pipeline.fail_fast);
        assert!(pipeline.dry_run);
    }

    #[test]
    fn test_build_pipeline_empty_build() {
        let (_temp, ctx) = create_test_context();
        let pipeline = BuildPipeline::new(ctx);

        let result = pipeline.build().unwrap();
        assert!(result.is_success());
        assert_eq!(result.targets.len(), 0);
    }

    #[test]
    fn test_build_pipeline_dry_run() {
        let (temp, ctx) = create_test_context();

        // Create a source file
        let src_dir = temp.path().join("src/pxl");
        let sprite_file = src_dir.join("test.pxl");
        File::create(&sprite_file).unwrap().write_all(b"{}").unwrap();

        let pipeline = BuildPipeline::new(ctx).with_dry_run(true);
        let result = pipeline.build().unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn test_build_builder() {
        let (_temp, ctx) = create_test_context();

        let result = Build::new().context(ctx).dry_run(true).verbose(false).run().unwrap();

        assert!(result.is_success());
    }

    #[test]
    fn test_execute_target_missing_source() {
        let (_temp, ctx) = create_test_context();
        let pipeline = BuildPipeline::new(ctx);

        let target = BuildTarget::sprite(
            "missing".to_string(),
            std::path::PathBuf::from("/nonexistent/file.pxl"),
            std::path::PathBuf::from("/output/missing.png"),
        );

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_failure());
    }

    #[test]
    fn test_build_sprite_renders_png() {
        let (temp, ctx) = create_test_context();

        // Create a valid sprite file with inline palette
        let src_dir = temp.path().join("src/pxl");
        let sprite_file = src_dir.join("red_dot.pxl");
        let sprite_content = r##"{"type": "sprite", "name": "red_dot", "palette": {"{r}": "#FF0000"}, "grid": ["{r}"]}"##;
        File::create(&sprite_file).unwrap().write_all(sprite_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("red_dot.png");

        let pipeline = BuildPipeline::new(ctx);

        let target = BuildTarget::sprite("red_dot".to_string(), sprite_file, output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);
        assert!(output_file.exists(), "Output PNG file should exist");

        // Verify the PNG was created and has valid content
        let img = image::open(&output_file).expect("Should open as valid PNG");
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }

    #[test]
    fn test_build_sprite_with_named_palette() {
        let (temp, ctx) = create_test_context();

        // Create a sprite file with a named palette reference
        let src_dir = temp.path().join("src/pxl");
        let sprite_file = src_dir.join("green_pixel.pxl");
        let sprite_content = r##"{"type": "palette", "name": "colors", "colors": {"{g}": "#00FF00"}}
{"type": "sprite", "name": "green_pixel", "palette": "colors", "grid": ["{g}"]}"##;
        File::create(&sprite_file).unwrap().write_all(sprite_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("green_pixel.png");

        let pipeline = BuildPipeline::new(ctx);

        let target =
            BuildTarget::sprite("green_pixel".to_string(), sprite_file, output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);
        assert!(output_file.exists(), "Output PNG file should exist");

        // Verify the PNG was created with correct color
        let img = image::open(&output_file).expect("Should open as valid PNG").to_rgba8();
        let pixel = img.get_pixel(0, 0);
        assert_eq!(pixel[0], 0, "Red channel should be 0");
        assert_eq!(pixel[1], 255, "Green channel should be 255");
        assert_eq!(pixel[2], 0, "Blue channel should be 0");
    }

    #[test]
    fn test_build_sprite_2x2_grid() {
        let (temp, ctx) = create_test_context();

        // Create a 2x2 sprite
        let src_dir = temp.path().join("src/pxl");
        let sprite_file = src_dir.join("checkerboard.pxl");
        let sprite_content = r##"{"type": "sprite", "name": "checkerboard", "palette": {"{b}": "#000000", "{w}": "#FFFFFF"}, "grid": ["{b}{w}", "{w}{b}"]}"##;
        File::create(&sprite_file).unwrap().write_all(sprite_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("checkerboard.png");

        let pipeline = BuildPipeline::new(ctx);

        let target =
            BuildTarget::sprite("checkerboard".to_string(), sprite_file, output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);

        // Verify the dimensions
        let img = image::open(&output_file).expect("Should open as valid PNG");
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
    }

    fn create_atlas_test_context(atlas_name: &str, sources: Vec<&str>) -> (TempDir, BuildContext) {
        use crate::config::{AtlasConfig as ConfigAtlas, ProjectConfig, PxlConfig};

        let temp = TempDir::new().unwrap();

        // Create a config with an atlas definition
        let mut atlases = std::collections::HashMap::new();
        atlases.insert(
            atlas_name.to_string(),
            ConfigAtlas {
                sources: sources.into_iter().map(String::from).collect(),
                max_size: [1024, 1024],
                padding: Some(0),
                power_of_two: false,
                nine_slice: false,
            },
        );

        let config = PxlConfig {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                src: std::path::PathBuf::from("src/pxl"),
                out: std::path::PathBuf::from("build"),
            },
            atlases,
            ..default_config()
        };

        let ctx = BuildContext::new(config, temp.path().to_path_buf());

        // Create source directory
        let src_dir = temp.path().join("src/pxl");
        fs::create_dir_all(&src_dir).unwrap();

        (temp, ctx)
    }

    #[test]
    fn test_build_atlas_single_sprite() {
        let (temp, ctx) = create_atlas_test_context("test_atlas", vec!["sprites/*.pxl"]);

        // Create source directory and sprite file
        let src_dir = temp.path().join("src/pxl/sprites");
        fs::create_dir_all(&src_dir).unwrap();

        let sprite_file = src_dir.join("red.pxl");
        let sprite_content = r##"{"type": "sprite", "name": "red", "palette": {"{r}": "#FF0000"}, "grid": ["{r}{r}", "{r}{r}"]}"##;
        File::create(&sprite_file).unwrap().write_all(sprite_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("test_atlas.png");

        let pipeline = BuildPipeline::new(ctx);

        let target =
            BuildTarget::atlas("test_atlas".to_string(), vec![sprite_file], output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);

        // Verify PNG was created
        let png_path = out_dir.join("test_atlas.png");
        assert!(png_path.exists(), "Atlas PNG should exist");

        let img = image::open(&png_path).expect("Should open as valid PNG");
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);

        // Verify JSON metadata was created
        let json_path = out_dir.join("test_atlas.json");
        assert!(json_path.exists(), "Atlas JSON should exist");

        let json_content = fs::read_to_string(&json_path).expect("Should read JSON");
        assert!(json_content.contains("\"red\""), "JSON should contain sprite name");
        assert!(json_content.contains("\"frames\""), "JSON should contain frames");
    }

    #[test]
    fn test_build_atlas_multiple_sprites() {
        let (temp, ctx) = create_atlas_test_context("chars", vec!["**/*.pxl"]);

        // Create source files
        let src_dir = temp.path().join("src/pxl");

        let red_file = src_dir.join("red.pxl");
        let red_content = r##"{"type": "sprite", "name": "red", "palette": {"{r}": "#FF0000"}, "grid": ["{r}"]}"##;
        File::create(&red_file).unwrap().write_all(red_content.as_bytes()).unwrap();

        let green_file = src_dir.join("green.pxl");
        let green_content = r##"{"type": "sprite", "name": "green", "palette": {"{g}": "#00FF00"}, "grid": ["{g}"]}"##;
        File::create(&green_file).unwrap().write_all(green_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("chars.png");

        let pipeline = BuildPipeline::new(ctx);

        let target = BuildTarget::atlas(
            "chars".to_string(),
            vec![red_file, green_file],
            output_file.clone(),
        );

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);

        // Verify PNG was created
        let png_path = out_dir.join("chars.png");
        assert!(png_path.exists(), "Atlas PNG should exist");

        // Verify JSON contains both sprites
        let json_path = out_dir.join("chars.json");
        assert!(json_path.exists(), "Atlas JSON should exist");

        let json_content = fs::read_to_string(&json_path).expect("Should read JSON");
        // With multiple source files, names are qualified
        assert!(json_content.contains("red"), "JSON should contain red sprite");
        assert!(json_content.contains("green"), "JSON should contain green sprite");
    }

    #[test]
    fn test_build_atlas_with_metadata() {
        let (temp, ctx) = create_atlas_test_context("player", vec!["*.pxl"]);

        // Create source file with metadata
        let src_dir = temp.path().join("src/pxl");
        let sprite_file = src_dir.join("player.pxl");
        let sprite_content = r##"{"type": "sprite", "name": "player", "palette": {"{r}": "#FF0000"}, "grid": ["{r}{r}{r}{r}", "{r}{r}{r}{r}", "{r}{r}{r}{r}", "{r}{r}{r}{r}"], "metadata": {"origin": [2, 4], "boxes": {"hurt": {"x": 0, "y": 0, "w": 4, "h": 4}}}}"##;
        File::create(&sprite_file).unwrap().write_all(sprite_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("player.png");

        let pipeline = BuildPipeline::new(ctx);

        let target =
            BuildTarget::atlas("player".to_string(), vec![sprite_file], output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);

        // Verify JSON contains metadata
        let json_path = out_dir.join("player.json");
        let json_content = fs::read_to_string(&json_path).expect("Should read JSON");

        assert!(json_content.contains("\"origin\""), "JSON should contain origin");
        assert!(json_content.contains("\"boxes\""), "JSON should contain boxes");
        assert!(json_content.contains("\"hurt\""), "JSON should contain hurt box");
    }

    #[test]
    fn test_build_atlas_no_sprites_error() {
        let (temp, ctx) = create_atlas_test_context("empty", vec!["*.pxl"]);

        // Create source file with only a palette (no sprites)
        let src_dir = temp.path().join("src/pxl");
        let palette_file = src_dir.join("colors.pxl");
        let palette_content =
            r##"{"type": "palette", "name": "colors", "colors": {"{r}": "#FF0000"}}"##;
        File::create(&palette_file).unwrap().write_all(palette_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("empty.png");

        let pipeline = BuildPipeline::new(ctx);

        let target =
            BuildTarget::atlas("empty".to_string(), vec![palette_file], output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_failure(), "Should fail when no sprites found");
    }

    #[test]
    fn test_build_atlas_power_of_two() {
        use crate::config::{AtlasConfig as ConfigAtlas, ProjectConfig, PxlConfig};

        let temp = TempDir::new().unwrap();

        // Create a config with power_of_two enabled
        let mut atlases = std::collections::HashMap::new();
        atlases.insert(
            "pot".to_string(),
            ConfigAtlas {
                sources: vec!["*.pxl".to_string()],
                max_size: [1024, 1024],
                padding: Some(0),
                power_of_two: true,
                nine_slice: false,
            },
        );

        let config = PxlConfig {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                src: std::path::PathBuf::from("src/pxl"),
                out: std::path::PathBuf::from("build"),
            },
            atlases,
            ..default_config()
        };

        let ctx = BuildContext::new(config, temp.path().to_path_buf());

        // Create source file (3x3 sprite, should become 4x4 with PoT)
        let src_dir = temp.path().join("src/pxl");
        fs::create_dir_all(&src_dir).unwrap();

        let sprite_file = src_dir.join("small.pxl");
        let sprite_content = r##"{"type": "sprite", "name": "small", "palette": {"{r}": "#FF0000"}, "grid": ["{r}{r}{r}", "{r}{r}{r}", "{r}{r}{r}"]}"##;
        File::create(&sprite_file).unwrap().write_all(sprite_content.as_bytes()).unwrap();

        // Create output directory
        let out_dir = temp.path().join("build");
        fs::create_dir_all(&out_dir).unwrap();
        let output_file = out_dir.join("pot.png");

        let pipeline = BuildPipeline::new(ctx);

        let target = BuildTarget::atlas("pot".to_string(), vec![sprite_file], output_file.clone());

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);

        // Verify PNG has power-of-two dimensions
        let png_path = out_dir.join("pot.png");
        let img = image::open(&png_path).expect("Should open as valid PNG");
        assert_eq!(img.width(), 4, "Width should be power of 2 (4)");
        assert_eq!(img.height(), 4, "Height should be power of 2 (4)");
    }
}
