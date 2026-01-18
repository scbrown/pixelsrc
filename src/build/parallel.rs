//! Parallel build execution.
//!
//! Provides parallel execution of build targets by grouping independent
//! targets into waves that can be built concurrently.
//!
//! # How It Works
//!
//! 1. Analyze the dependency graph to find the "depth" of each target
//! 2. Group targets into levels where all dependencies are in earlier levels
//! 3. Execute each level in parallel using a thread pool
//! 4. Wait for all targets in a level to complete before starting the next
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::build::{BuildContext, ParallelBuild};
//!
//! let context = BuildContext::new(config, project_root);
//! let result = ParallelBuild::new(context)
//!     .with_jobs(4)  // Use 4 parallel workers
//!     .run()?;
//!
//! println!("Built {} targets in {:?}", result.success_count(), result.total_duration);
//! ```

use crate::build::{
    BuildContext, BuildError, BuildPlan, BuildResult, BuildTarget, TargetKind, TargetResult,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Default number of parallel jobs (uses available parallelism).
fn default_jobs() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

/// Parallel build executor.
pub struct ParallelBuild {
    /// Build context
    context: BuildContext,
    /// Number of parallel jobs
    jobs: usize,
    /// Whether to stop on first error
    fail_fast: bool,
}

impl ParallelBuild {
    /// Create a new parallel build.
    pub fn new(context: BuildContext) -> Self {
        Self { context, jobs: default_jobs(), fail_fast: false }
    }

    /// Set the number of parallel jobs.
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs.max(1);
        self
    }

    /// Set fail-fast mode (stop on first error).
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Get the number of parallel jobs.
    pub fn jobs(&self) -> usize {
        self.jobs
    }

    /// Run the parallel build.
    pub fn run(&self) -> Result<BuildResult, BuildError> {
        let start = Instant::now();

        // Create build plan from config
        let plan = crate::build::create_build_plan(&self.context)?;

        // Apply target filter if specified
        let plan = if let Some(filter) = self.context.target_filter() {
            plan.filter(filter)
        } else {
            plan
        };

        // Execute the plan in parallel
        let mut result = self.execute_plan(&plan)?;
        result.total_duration = start.elapsed();

        Ok(result)
    }

    /// Run the build with a pre-created plan.
    pub fn run_plan(&self, plan: &BuildPlan) -> Result<BuildResult, BuildError> {
        let start = Instant::now();
        let mut result = self.execute_plan(plan)?;
        result.total_duration = start.elapsed();
        Ok(result)
    }

    /// Execute a build plan in parallel.
    fn execute_plan(&self, plan: &BuildPlan) -> Result<BuildResult, BuildError> {
        // Get targets grouped by dependency level
        let levels = self.compute_levels(plan)?;

        if self.context.is_verbose() {
            println!(
                "Parallel build: {} targets in {} levels ({} workers)",
                plan.len(),
                levels.len(),
                self.jobs
            );
            for (i, level) in levels.iter().enumerate() {
                let ids: Vec<_> = level.iter().map(|t| &t.id).collect();
                println!("  Level {}: {:?}", i, ids);
            }
        }

        // Ensure output directory exists
        std::fs::create_dir_all(self.context.out_dir())?;

        // Execute each level in parallel
        let mut result = BuildResult::new();
        let failed = Arc::new(Mutex::new(false));

        for level in levels {
            if self.fail_fast && *failed.lock().unwrap() {
                break;
            }

            let level_results = self.execute_level(&level, Arc::clone(&failed))?;

            for target_result in level_results {
                if target_result.status.is_failure() && self.fail_fast {
                    *failed.lock().unwrap() = true;
                }
                result.add_result(target_result);
            }
        }

        Ok(result)
    }

    /// Compute dependency levels for the build plan.
    ///
    /// Returns a vector of vectors, where each inner vector contains targets
    /// that can be built in parallel (all their dependencies are in earlier levels).
    fn compute_levels<'a>(
        &self,
        plan: &'a BuildPlan,
    ) -> Result<Vec<Vec<&'a BuildTarget>>, BuildError> {
        let targets = plan.targets();

        if targets.is_empty() {
            return Ok(vec![]);
        }

        // Build a map from target ID to target
        let target_map: HashMap<&str, &BuildTarget> =
            targets.iter().map(|t| (t.id.as_str(), t)).collect();

        // Build a map from target ID to its dependencies (that exist in the plan)
        let deps_map: HashMap<&str, Vec<&str>> = targets
            .iter()
            .map(|t| {
                let deps: Vec<&str> = t
                    .dependencies
                    .iter()
                    .filter_map(|d| {
                        if target_map.contains_key(d.as_str()) {
                            Some(d.as_str())
                        } else {
                            None
                        }
                    })
                    .collect();
                (t.id.as_str(), deps)
            })
            .collect();

        // Compute the level of each target (0 = no dependencies)
        let mut levels_map: HashMap<&str, usize> = HashMap::new();
        let mut remaining: HashSet<&str> = targets.iter().map(|t| t.id.as_str()).collect();
        let mut current_level = 0;

        while !remaining.is_empty() {
            let mut this_level: Vec<&str> = Vec::new();

            for &id in &remaining {
                let deps = deps_map.get(id).map(|v| v.as_slice()).unwrap_or(&[]);
                let all_deps_resolved = deps.iter().all(|d| levels_map.contains_key(d));

                if all_deps_resolved {
                    this_level.push(id);
                }
            }

            if this_level.is_empty() {
                // Circular dependency - shouldn't happen if build_order works
                return Err(BuildError::Build(
                    "Unable to compute build levels - possible circular dependency".to_string(),
                ));
            }

            for id in &this_level {
                levels_map.insert(id, current_level);
                remaining.remove(id);
            }

            current_level += 1;
        }

        // Group targets by level
        let max_level = levels_map.values().copied().max().unwrap_or(0);
        let mut result: Vec<Vec<&BuildTarget>> = vec![Vec::new(); max_level + 1];

        for target in targets {
            if let Some(&level) = levels_map.get(target.id.as_str()) {
                result[level].push(target);
            }
        }

        Ok(result)
    }

    /// Execute a single level of targets in parallel.
    fn execute_level(
        &self,
        targets: &[&BuildTarget],
        failed: Arc<Mutex<bool>>,
    ) -> Result<Vec<TargetResult>, BuildError> {
        if targets.is_empty() {
            return Ok(vec![]);
        }

        // For single-threaded or single-target levels, just execute sequentially
        if self.jobs == 1 || targets.len() == 1 {
            return Ok(targets.iter().map(|t| self.execute_target(t)).collect());
        }

        // Execute in parallel using scoped threads
        let results = Arc::new(Mutex::new(Vec::new()));
        let context = &self.context;
        let fail_fast = self.fail_fast;
        let next_idx = std::sync::atomic::AtomicUsize::new(0);

        std::thread::scope(|s| {
            // Spawn worker threads
            let num_workers = self.jobs.min(targets.len());

            for _ in 0..num_workers {
                let results = Arc::clone(&results);
                let failed = Arc::clone(&failed);
                let next_idx = &next_idx;

                s.spawn(move || {
                    loop {
                        // Check if we should stop
                        if fail_fast && *failed.lock().unwrap() {
                            break;
                        }

                        // Get next work item
                        let idx = next_idx.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if idx >= targets.len() {
                            break;
                        }

                        let target = targets[idx];
                        let result = self.execute_target_internal(target, context);

                        if result.status.is_failure() && fail_fast {
                            *failed.lock().unwrap() = true;
                        }

                        results.lock().unwrap().push((idx, result));
                    }
                });
            }
        });

        // Sort results by original index to maintain deterministic order
        let mut results = Arc::try_unwrap(results)
            .map(|mutex| mutex.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone());
        results.sort_by_key(|(idx, _)| *idx);

        Ok(results.into_iter().map(|(_, r)| r).collect())
    }

    /// Execute a single build target.
    fn execute_target(&self, target: &BuildTarget) -> TargetResult {
        self.execute_target_internal(target, &self.context)
    }

    /// Execute a single build target (internal, takes context reference).
    fn execute_target_internal(
        &self,
        target: &BuildTarget,
        context: &BuildContext,
    ) -> TargetResult {
        let start = Instant::now();

        if context.is_verbose() {
            println!("Building: {} ...", target.id);
        }

        // Ensure parent directory exists for output
        if let Some(parent) = target.output.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
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
                if context.is_verbose() {
                    println!("  Done in {:?}", duration);
                }
                TargetResult::success(target.id.clone(), outputs, duration)
            }
            Err(e) => {
                if context.is_verbose() {
                    println!("  Failed: {}", e);
                }
                TargetResult::failed(target.id.clone(), e, duration)
            }
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

/// Statistics about parallel build execution.
#[derive(Debug, Clone, Default)]
pub struct ParallelStats {
    /// Number of dependency levels
    pub levels: usize,
    /// Number of workers used
    pub workers: usize,
    /// Maximum parallelism achieved (targets built simultaneously)
    pub max_parallelism: usize,
    /// Total targets built
    pub total_targets: usize,
}

impl ParallelStats {
    /// Create stats from a build plan and configuration.
    pub fn from_plan(plan: &BuildPlan, jobs: usize) -> Self {
        let targets = plan.targets();

        // Compute levels (simplified version)
        let mut levels = 0;
        let mut max_parallelism = 0;

        if !targets.is_empty() {
            // Count targets with no dependencies as level 0
            let no_deps = targets.iter().filter(|t| t.dependencies.is_empty()).count();
            max_parallelism = no_deps.min(jobs);
            levels = 1; // At least one level

            // Rough estimate: if we have dependencies, we have more levels
            if targets.iter().any(|t| !t.dependencies.is_empty()) {
                levels = 2; // At least 2 levels if there are dependencies
            }
        }

        Self { levels, workers: jobs, max_parallelism, total_targets: targets.len() }
    }

    /// Get the theoretical speedup factor.
    pub fn speedup_factor(&self) -> f64 {
        if self.levels == 0 {
            1.0
        } else {
            self.total_targets as f64 / self.levels as f64
        }
    }
}

impl std::fmt::Display for ParallelStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} targets in {} levels ({} workers, max {} parallel)",
            self.total_targets, self.levels, self.workers, self.max_parallelism
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_config;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_context() -> (TempDir, BuildContext) {
        let temp = TempDir::new().unwrap();
        let config = default_config();
        let ctx = BuildContext::new(config, temp.path().to_path_buf());

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
    fn test_parallel_build_new() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx);

        assert!(build.jobs >= 1);
        assert!(!build.fail_fast);
    }

    #[test]
    fn test_parallel_build_with_options() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx).with_jobs(4).with_fail_fast(true);

        assert_eq!(build.jobs, 4);
        assert!(build.fail_fast);
    }

    #[test]
    fn test_parallel_build_jobs_minimum() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx).with_jobs(0);

        assert_eq!(build.jobs, 1); // Should be at least 1
    }

    #[test]
    fn test_parallel_build_run_empty() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx);

        let result = build.run().unwrap();
        assert!(result.is_success());
        assert_eq!(result.targets.len(), 0);
    }

    #[test]
    fn test_compute_levels_empty() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx);
        let plan = BuildPlan::new();

        let levels = build.compute_levels(&plan).unwrap();
        assert!(levels.is_empty());
    }

    #[test]
    fn test_compute_levels_no_deps() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx);

        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::sprite(
            "a".to_string(),
            PathBuf::from("a.pxl"),
            PathBuf::from("a.png"),
        ));
        plan.add_target(BuildTarget::sprite(
            "b".to_string(),
            PathBuf::from("b.pxl"),
            PathBuf::from("b.png"),
        ));
        plan.add_target(BuildTarget::sprite(
            "c".to_string(),
            PathBuf::from("c.pxl"),
            PathBuf::from("c.png"),
        ));

        let levels = build.compute_levels(&plan).unwrap();
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 3);
    }

    #[test]
    fn test_compute_levels_with_deps() {
        let (_temp, ctx) = create_test_context();
        let build = ParallelBuild::new(ctx);

        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::animation(
            "walk".to_string(),
            PathBuf::from("walk.pxl"),
            PathBuf::from("walk.png"),
        ));
        plan.add_target(BuildTarget::animation_preview(
            "walk".to_string(),
            PathBuf::from("walk.pxl"),
            PathBuf::from("walk.gif"),
        ));
        plan.add_target(BuildTarget::animation(
            "run".to_string(),
            PathBuf::from("run.pxl"),
            PathBuf::from("run.png"),
        ));
        plan.add_target(BuildTarget::animation_preview(
            "run".to_string(),
            PathBuf::from("run.pxl"),
            PathBuf::from("run.gif"),
        ));

        let levels = build.compute_levels(&plan).unwrap();
        assert_eq!(levels.len(), 2);
        // Level 0: both animations (no deps)
        assert_eq!(levels[0].len(), 2);
        // Level 1: both previews (depend on animations)
        assert_eq!(levels[1].len(), 2);
    }

    #[test]
    fn test_execute_level_single_target() {
        let (temp, ctx) = create_test_context();
        let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");

        let build = ParallelBuild::new(ctx);
        let target =
            BuildTarget::sprite("test".to_string(), source, temp.path().join("build/test.png"));

        let failed = Arc::new(Mutex::new(false));
        let results = build.execute_level(&[&target], failed).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
    }

    #[test]
    fn test_execute_level_multiple_targets() {
        let (temp, ctx) = create_test_context();
        let source_a = create_test_file(temp.path(), "src/pxl/a.pxl", "content a");
        let source_b = create_test_file(temp.path(), "src/pxl/b.pxl", "content b");
        let source_c = create_test_file(temp.path(), "src/pxl/c.pxl", "content c");

        let build = ParallelBuild::new(ctx).with_jobs(2);

        let targets = [
            BuildTarget::sprite("a".to_string(), source_a, temp.path().join("build/a.png")),
            BuildTarget::sprite("b".to_string(), source_b, temp.path().join("build/b.png")),
            BuildTarget::sprite("c".to_string(), source_c, temp.path().join("build/c.png")),
        ];

        let target_refs: Vec<&BuildTarget> = targets.iter().collect();
        let failed = Arc::new(Mutex::new(false));
        let results = build.execute_level(&target_refs, failed).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_success()));

        // Results should be in original order
        assert_eq!(results[0].target_id, "sprite:a");
        assert_eq!(results[1].target_id, "sprite:b");
        assert_eq!(results[2].target_id, "sprite:c");
    }

    #[test]
    fn test_execute_level_with_failure() {
        let (temp, ctx) = create_test_context();
        let source_a = create_test_file(temp.path(), "src/pxl/a.pxl", "content a");

        let build = ParallelBuild::new(ctx).with_jobs(2);

        let targets = [
            BuildTarget::sprite("a".to_string(), source_a, temp.path().join("build/a.png")),
            BuildTarget::sprite(
                "missing".to_string(),
                PathBuf::from("/nonexistent/file.pxl"),
                temp.path().join("build/missing.png"),
            ),
        ];

        let target_refs: Vec<&BuildTarget> = targets.iter().collect();
        let failed = Arc::new(Mutex::new(false));
        let results = build.execute_level(&target_refs, failed).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].is_success());
        assert!(results[1].status.is_failure());
    }

    #[test]
    fn test_parallel_stats_empty() {
        let plan = BuildPlan::new();
        let stats = ParallelStats::from_plan(&plan, 4);

        assert_eq!(stats.levels, 0);
        assert_eq!(stats.total_targets, 0);
        assert_eq!(stats.workers, 4);
    }

    #[test]
    fn test_parallel_stats_with_targets() {
        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::sprite(
            "a".to_string(),
            PathBuf::from("a.pxl"),
            PathBuf::from("a.png"),
        ));
        plan.add_target(BuildTarget::sprite(
            "b".to_string(),
            PathBuf::from("b.pxl"),
            PathBuf::from("b.png"),
        ));

        let stats = ParallelStats::from_plan(&plan, 4);

        assert!(stats.levels >= 1);
        assert_eq!(stats.total_targets, 2);
        assert_eq!(stats.workers, 4);
        assert!(stats.max_parallelism <= 4);
    }

    #[test]
    fn test_parallel_stats_display() {
        let stats = ParallelStats { levels: 3, workers: 4, max_parallelism: 2, total_targets: 10 };

        let display = format!("{}", stats);
        assert!(display.contains("10 targets"));
        assert!(display.contains("3 levels"));
        assert!(display.contains("4 workers"));
    }

    #[test]
    fn test_parallel_stats_speedup() {
        let stats = ParallelStats { levels: 2, workers: 4, max_parallelism: 4, total_targets: 8 };

        assert!((stats.speedup_factor() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_run_plan_with_dependencies() {
        let (temp, ctx) = create_test_context();
        let walk_src = create_test_file(temp.path(), "src/pxl/walk.pxl", "walk content");
        let run_src = create_test_file(temp.path(), "src/pxl/run.pxl", "run content");

        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::animation(
            "walk".to_string(),
            walk_src.clone(),
            temp.path().join("build/walk.png"),
        ));
        plan.add_target(BuildTarget::animation_preview(
            "walk".to_string(),
            walk_src,
            temp.path().join("build/walk.gif"),
        ));
        plan.add_target(BuildTarget::animation(
            "run".to_string(),
            run_src.clone(),
            temp.path().join("build/run.png"),
        ));
        plan.add_target(BuildTarget::animation_preview(
            "run".to_string(),
            run_src,
            temp.path().join("build/run.gif"),
        ));

        let build = ParallelBuild::new(ctx).with_jobs(2);
        let result = build.run_plan(&plan).unwrap();

        assert!(result.is_success());
        assert_eq!(result.targets.len(), 4);
    }

    #[test]
    fn test_default_jobs() {
        let jobs = default_jobs();
        assert!(jobs >= 1);
    }
}
