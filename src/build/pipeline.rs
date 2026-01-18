//! Build pipeline orchestration.
//!
//! The pipeline coordinates the execution of build targets in the correct order.

use crate::build::{BuildContext, BuildPlan, BuildResult, BuildTarget, TargetKind, TargetResult};
use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::registry::PaletteRegistry;
use crate::renderer::render_sprite;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;

/// Error during build execution.
#[derive(Debug)]
pub enum BuildError {
    /// Discovery error
    Discovery(crate::build::DiscoveryError),
    /// Build order error (circular dependencies)
    BuildOrder(crate::build::target::BuildOrderError),
    /// IO error
    Io(std::io::Error),
    /// Generic build error
    Build(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Discovery(e) => write!(f, "Discovery error: {}", e),
            BuildError::BuildOrder(e) => write!(f, "Build order error: {}", e),
            BuildError::Io(e) => write!(f, "IO error: {}", e),
            BuildError::Build(e) => write!(f, "Build error: {}", e),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<crate::build::DiscoveryError> for BuildError {
    fn from(e: crate::build::DiscoveryError) -> Self {
        BuildError::Discovery(e)
    }
}

impl From<crate::build::target::BuildOrderError> for BuildError {
    fn from(e: crate::build::target::BuildOrderError) -> Self {
        BuildError::BuildOrder(e)
    }
}

impl From<std::io::Error> for BuildError {
    fn from(e: std::io::Error) -> Self {
        BuildError::Io(e)
    }
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
            sprites
                .into_iter()
                .find(|s| s.name == target.name)
                .ok_or_else(|| {
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
    fn build_atlas(&self, target: &BuildTarget) -> Result<Vec<std::path::PathBuf>, String> {
        // Atlas building will be implemented by BST-5
        // For now, validate sources exist
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
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
    fn build_export(&self, target: &BuildTarget) -> Result<Vec<std::path::PathBuf>, String> {
        // Export building will be implemented by downstream tasks
        Ok(vec![target.output.clone()])
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

        let target = BuildTarget::sprite(
            "red_dot".to_string(),
            sprite_file,
            output_file.clone(),
        );

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

        let target = BuildTarget::sprite(
            "green_pixel".to_string(),
            sprite_file,
            output_file.clone(),
        );

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

        let target = BuildTarget::sprite(
            "checkerboard".to_string(),
            sprite_file,
            output_file.clone(),
        );

        let result = pipeline.execute_target(&target);
        assert!(result.status.is_success(), "Expected success, got: {:?}", result.status);

        // Verify the dimensions
        let img = image::open(&output_file).expect("Should open as valid PNG");
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
    }
}
