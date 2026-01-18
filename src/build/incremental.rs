//! Incremental build support.
//!
//! Provides incremental build capabilities by tracking source file changes
//! and skipping targets that are already up-to-date.
//!
//! # How It Works
//!
//! The incremental build system uses a manifest file (`.pxl-manifest.json`)
//! to track the state of previous builds. For each target:
//!
//! 1. Check if the target was previously built
//! 2. Compare current source file hashes to recorded hashes
//! 3. Verify all output files still exist
//! 4. Skip the target if everything matches
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::build::{BuildContext, IncrementalBuild};
//!
//! let context = BuildContext::new(config, project_root);
//! let result = IncrementalBuild::new(context)
//!     .run()?;
//!
//! println!("Built: {}, Skipped: {}", result.success_count(), result.skipped_count());
//! ```

use crate::build::{
    BuildContext, BuildError, BuildManifest, BuildPlan, BuildResult, BuildTarget, ManifestError,
    TargetResult,
};
use std::path::PathBuf;
use std::time::Instant;

/// Incremental build pipeline.
///
/// Wraps the standard build pipeline with manifest-based change detection
/// to skip targets that are already up-to-date.
pub struct IncrementalBuild {
    /// Build context
    context: BuildContext,
    /// Build manifest for tracking changes
    manifest: BuildManifest,
    /// Whether to force rebuild all targets (ignore manifest)
    force: bool,
    /// Whether to stop on first error
    fail_fast: bool,
    /// Whether to save the manifest after building
    save_manifest: bool,
}

impl IncrementalBuild {
    /// Create a new incremental build.
    ///
    /// Loads the manifest from the output directory if it exists.
    pub fn new(context: BuildContext) -> Self {
        let manifest =
            BuildManifest::load_from_dir(&context.out_dir()).ok().flatten().unwrap_or_default();

        Self { context, manifest, force: false, fail_fast: false, save_manifest: true }
    }

    /// Create an incremental build with a pre-loaded manifest.
    pub fn with_manifest(context: BuildContext, manifest: BuildManifest) -> Self {
        Self { context, manifest, force: false, fail_fast: false, save_manifest: true }
    }

    /// Set force mode (rebuild all targets regardless of manifest).
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Set fail-fast mode (stop on first error).
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set whether to save the manifest after building.
    pub fn with_save_manifest(mut self, save: bool) -> Self {
        self.save_manifest = save;
        self
    }

    /// Get a reference to the manifest.
    pub fn manifest(&self) -> &BuildManifest {
        &self.manifest
    }

    /// Get a mutable reference to the manifest.
    pub fn manifest_mut(&mut self) -> &mut BuildManifest {
        &mut self.manifest
    }

    /// Run the incremental build.
    pub fn run(&mut self) -> Result<BuildResult, BuildError> {
        let start = Instant::now();

        // Create build plan from config
        let plan = crate::build::create_build_plan(&self.context)?;

        // Apply target filter if specified
        let plan = if let Some(filter) = self.context.target_filter() {
            plan.filter(filter)
        } else {
            plan
        };

        // Execute the plan with incremental checks
        let mut result = self.execute_plan(&plan)?;
        result.total_duration = start.elapsed();

        // Save the manifest
        if self.save_manifest {
            self.save_manifest_to_disk()?;
        }

        Ok(result)
    }

    /// Run the build with a pre-created plan.
    pub fn run_plan(&mut self, plan: &BuildPlan) -> Result<BuildResult, BuildError> {
        let start = Instant::now();
        let mut result = self.execute_plan(plan)?;
        result.total_duration = start.elapsed();

        if self.save_manifest {
            self.save_manifest_to_disk()?;
        }

        Ok(result)
    }

    /// Check if a target needs to be rebuilt.
    pub fn needs_rebuild(&self, target: &BuildTarget) -> Result<bool, ManifestError> {
        if self.force {
            return Ok(true);
        }

        self.manifest.needs_rebuild(&target.id, &target.sources)
    }

    /// Record a successful build in the manifest.
    pub fn record_build(
        &mut self,
        target: &BuildTarget,
        outputs: &[PathBuf],
    ) -> Result<(), ManifestError> {
        self.manifest.record_build(&target.id, &target.sources, outputs)
    }

    /// Save the manifest to disk.
    fn save_manifest_to_disk(&mut self) -> Result<(), BuildError> {
        self.manifest
            .save_to_dir(&self.context.out_dir())
            .map_err(|e| BuildError::Io(std::io::Error::other(e.to_string())))
    }

    /// Execute a build plan with incremental checking.
    fn execute_plan(&mut self, plan: &BuildPlan) -> Result<BuildResult, BuildError> {
        let mut result = BuildResult::new();

        // Get targets in build order
        let ordered = plan.build_order()?;

        if self.context.is_verbose() {
            println!("Incremental build: {} targets", ordered.len());
        }

        // Ensure output directory exists
        std::fs::create_dir_all(self.context.out_dir())?;

        // Execute each target
        for target in ordered {
            let target_result = self.execute_target(target)?;

            if target_result.status.is_failure() && self.fail_fast {
                result.add_result(target_result);
                return Ok(result);
            }

            result.add_result(target_result);
        }

        Ok(result)
    }

    /// Execute a single build target with incremental check.
    fn execute_target(&mut self, target: &BuildTarget) -> Result<TargetResult, BuildError> {
        // Check if rebuild is needed
        let needs_rebuild = match self.needs_rebuild(target) {
            Ok(needs) => needs,
            Err(e) => {
                // If we can't determine status, assume rebuild needed
                if self.context.is_verbose() {
                    println!("Warning: Could not check manifest for {}: {}", target.id, e);
                }
                true
            }
        };

        if !needs_rebuild {
            if self.context.is_verbose() {
                println!("Skipping {} (up to date)", target.id);
            }
            return Ok(TargetResult::skipped(target.id.clone()));
        }

        // Execute the build
        let start = Instant::now();

        if self.context.is_verbose() {
            println!("Building: {} ...", target.id);
        }

        // Ensure parent directory exists for output
        if let Some(parent) = target.output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Execute based on target kind
        let build_result = self.execute_target_kind(target);
        let duration = start.elapsed();

        match build_result {
            Ok(outputs) => {
                // Record the successful build
                if let Err(e) = self.record_build(target, &outputs) {
                    if self.context.is_verbose() {
                        println!("Warning: Could not record build for {}: {}", target.id, e);
                    }
                }

                if self.context.is_verbose() {
                    println!("  Done in {:?}", duration);
                }
                Ok(TargetResult::success(target.id.clone(), outputs, duration))
            }
            Err(e) => {
                if self.context.is_verbose() {
                    println!("  Failed: {}", e);
                }
                Ok(TargetResult::failed(target.id.clone(), e, duration))
            }
        }
    }

    /// Execute the actual build for a target kind.
    fn execute_target_kind(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        use crate::build::TargetKind;

        match target.kind {
            TargetKind::Sprite => self.build_sprite(target),
            TargetKind::Atlas => self.build_atlas(target),
            TargetKind::Animation => self.build_animation(target),
            TargetKind::AnimationPreview => self.build_animation_preview(target),
            TargetKind::Export => self.build_export(target),
        }
    }

    /// Build a sprite target.
    fn build_sprite(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
    }

    /// Build an atlas target.
    fn build_atlas(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
    }

    /// Build an animation target.
    fn build_animation(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
    }

    /// Build an animation preview target.
    fn build_animation_preview(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        for source in &target.sources {
            if !source.exists() {
                return Err(format!("Source file not found: {}", source.display()));
            }
        }
        Ok(vec![target.output.clone()])
    }

    /// Build an export target.
    fn build_export(&self, target: &BuildTarget) -> Result<Vec<PathBuf>, String> {
        Ok(vec![target.output.clone()])
    }
}

/// Statistics about an incremental build.
#[derive(Debug, Clone, Default)]
pub struct IncrementalStats {
    /// Number of targets that were built
    pub built: usize,
    /// Number of targets that were skipped (up to date)
    pub skipped: usize,
    /// Number of targets that failed
    pub failed: usize,
    /// Total number of targets
    pub total: usize,
}

impl IncrementalStats {
    /// Create stats from a build result.
    pub fn from_result(result: &BuildResult) -> Self {
        Self {
            built: result.success_count(),
            skipped: result.skipped_count(),
            failed: result.failed_count(),
            total: result.targets.len(),
        }
    }

    /// Check if any targets were skipped.
    pub fn had_skips(&self) -> bool {
        self.skipped > 0
    }

    /// Check if any targets were rebuilt.
    pub fn had_rebuilds(&self) -> bool {
        self.built > 0
    }

    /// Get the percentage of targets that were skipped.
    pub fn skip_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.skipped as f64 / self.total as f64) * 100.0
        }
    }
}

impl std::fmt::Display for IncrementalStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} built, {} skipped, {} failed ({} total)",
            self.built, self.skipped, self.failed, self.total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::BuildTarget;
    use crate::config::default_config;
    use std::fs::{self, File};
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

    fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_incremental_build_new() {
        let (_temp, ctx) = create_test_context();
        let build = IncrementalBuild::new(ctx);

        assert!(!build.force);
        assert!(!build.fail_fast);
        assert!(build.save_manifest);
        assert!(build.manifest.is_empty());
    }

    #[test]
    fn test_incremental_build_with_options() {
        let (_temp, ctx) = create_test_context();
        let build = IncrementalBuild::new(ctx)
            .with_force(true)
            .with_fail_fast(true)
            .with_save_manifest(false);

        assert!(build.force);
        assert!(build.fail_fast);
        assert!(!build.save_manifest);
    }

    #[test]
    fn test_incremental_build_with_manifest() {
        let (_temp, ctx) = create_test_context();
        let mut manifest = BuildManifest::new();
        manifest.targets.insert(
            "test".to_string(),
            crate::build::TargetManifest {
                sources: std::collections::HashMap::new(),
                outputs: vec![],
                built_at: "2024-01-01T00:00:00Z".to_string(),
            },
        );

        let build = IncrementalBuild::with_manifest(ctx, manifest);
        assert_eq!(build.manifest.len(), 1);
    }

    #[test]
    fn test_needs_rebuild_never_built() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");

        let build = IncrementalBuild::new(ctx);
        let target =
            BuildTarget::sprite("test".to_string(), source, temp.path().join("build/test.png"));

        assert!(build.needs_rebuild(&target).unwrap());
    }

    #[test]
    fn test_needs_rebuild_up_to_date() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let target = BuildTarget::sprite("test".to_string(), source.clone(), output.clone());

        let mut build = IncrementalBuild::new(ctx);
        build.record_build(&target, &[output]).unwrap();

        assert!(!build.needs_rebuild(&target).unwrap());
    }

    #[test]
    fn test_needs_rebuild_force_mode() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let target = BuildTarget::sprite("test".to_string(), source.clone(), output.clone());

        let mut build = IncrementalBuild::new(ctx).with_force(true);
        build.record_build(&target, &[output]).unwrap();

        // Force mode should always return true
        assert!(build.needs_rebuild(&target).unwrap());
    }

    #[test]
    fn test_needs_rebuild_source_changed() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "original");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let target = BuildTarget::sprite("test".to_string(), source.clone(), output.clone());

        let mut build = IncrementalBuild::new(ctx);
        build.record_build(&target, &[output]).unwrap();

        // Modify source
        create_test_file(temp.path(), "src/pxl/test.pxl", "modified");

        assert!(build.needs_rebuild(&target).unwrap());
    }

    #[test]
    fn test_needs_rebuild_output_missing() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let target = BuildTarget::sprite("test".to_string(), source.clone(), output.clone());

        let mut build = IncrementalBuild::new(ctx);
        build.record_build(&target, &[output.clone()]).unwrap();

        // Delete output
        fs::remove_file(&output).unwrap();

        assert!(build.needs_rebuild(&target).unwrap());
    }

    #[test]
    fn test_record_build() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
        let output = temp.path().join("build/test.png");

        let target = BuildTarget::sprite("test".to_string(), source, output.clone());

        let mut build = IncrementalBuild::new(ctx);
        build.record_build(&target, &[output]).unwrap();

        assert_eq!(build.manifest.len(), 1);
        assert!(build.manifest.get_target("sprite:test").is_some());
    }

    #[test]
    fn test_incremental_build_run_empty() {
        let (_temp, ctx) = create_test_context();
        let mut build = IncrementalBuild::new(ctx).with_save_manifest(false);

        let result = build.run().unwrap();
        assert!(result.is_success());
        assert_eq!(result.targets.len(), 0);
    }

    #[test]
    fn test_incremental_stats_from_result() {
        let mut result = BuildResult::new();
        result.add_result(TargetResult::success(
            "a".to_string(),
            vec![],
            std::time::Duration::ZERO,
        ));
        result.add_result(TargetResult::skipped("b".to_string()));
        result.add_result(TargetResult::failed(
            "c".to_string(),
            "error".to_string(),
            std::time::Duration::ZERO,
        ));

        let stats = IncrementalStats::from_result(&result);
        assert_eq!(stats.built, 1);
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.total, 3);
    }

    #[test]
    fn test_incremental_stats_percentages() {
        let stats = IncrementalStats { built: 1, skipped: 3, failed: 0, total: 4 };

        assert!(stats.had_skips());
        assert!(stats.had_rebuilds());
        assert!((stats.skip_percentage() - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_incremental_stats_display() {
        let stats = IncrementalStats { built: 5, skipped: 10, failed: 1, total: 16 };

        let display = format!("{}", stats);
        assert!(display.contains("5 built"));
        assert!(display.contains("10 skipped"));
        assert!(display.contains("1 failed"));
        assert!(display.contains("16 total"));
    }

    #[test]
    fn test_incremental_stats_empty() {
        let stats = IncrementalStats::default();

        assert!(!stats.had_skips());
        assert!(!stats.had_rebuilds());
        assert_eq!(stats.skip_percentage(), 0.0);
    }

    #[test]
    fn test_manifest_accessor() {
        let (_temp, ctx) = create_test_context();
        let mut build = IncrementalBuild::new(ctx);

        assert!(build.manifest().is_empty());
        build.manifest_mut().clear();
        assert!(build.manifest().is_empty());
    }

    #[test]
    fn test_execute_target_missing_source() {
        let (temp, ctx) = create_test_context();
        let target = BuildTarget::sprite(
            "missing".to_string(),
            PathBuf::from("/nonexistent/file.pxl"),
            temp.path().join("build/missing.png"),
        );

        let mut build = IncrementalBuild::new(ctx).with_save_manifest(false);
        let result = build.execute_target(&target).unwrap();

        assert!(result.status.is_failure());
    }

    #[test]
    fn test_manifest_persistence() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let target = BuildTarget::sprite("test".to_string(), source, output.clone());

        // First build - record the target
        {
            let mut build = IncrementalBuild::new(ctx.clone());
            build.record_build(&target, &[output.clone()]).unwrap();
            build.save_manifest_to_disk().unwrap();
        }

        // Second build - should load the manifest
        {
            let build = IncrementalBuild::new(ctx);
            assert!(!build.needs_rebuild(&target).unwrap());
        }
    }
}
