//! Build System Test Suite (BST-19)
//!
//! Comprehensive integration tests for the pixelsrc build system.
//! Tests cover the full build pipeline including:
//!
//! - Pipeline orchestration (BST-5)
//! - Godot export (BST-12)
//! - Unity export (BST-13)
//! - libGDX export (BST-14)
//! - Incremental builds (BST-15)
//! - Parallel builds (BST-16)
//! - Progress reporting (BST-17)
//! - Watch error recovery (BST-18)

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;

use pixelsrc::build::progress::{
    ConsoleProgress, JsonProgress, NullProgress, ProgressEvent, ProgressReporter, ProgressTracker,
    TargetStatus,
};
use pixelsrc::build::{
    Build, BuildContext, BuildManifest, BuildPipeline, BuildPlan, BuildResult, BuildTarget,
    IncrementalBuild, IncrementalStats, ParallelBuild, ParallelStats, TargetKind,
};
use pixelsrc::config::default_config;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test build context with a temporary directory.
fn create_test_context() -> (TempDir, BuildContext) {
    let temp = TempDir::new().unwrap();
    let config = default_config();
    let ctx = BuildContext::new(config, temp.path().to_path_buf());

    // Create required directories
    let src_dir = temp.path().join("src/pxl");
    fs::create_dir_all(&src_dir).unwrap();
    let build_dir = temp.path().join("build");
    fs::create_dir_all(&build_dir).unwrap();

    (temp, ctx)
}

/// Create a test file with content.
fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// Test writer for capturing output.
struct TestWriter(Arc<Mutex<Vec<u8>>>);

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Pipeline Integration Tests
// ============================================================================

#[test]
fn test_build_pipeline_complete_workflow() {
    let (temp, ctx) = create_test_context();

    // Create source files
    create_test_file(temp.path(), "src/pxl/player.pxl", r#"{"name": "player"}"#);
    create_test_file(temp.path(), "src/pxl/enemy.pxl", r#"{"name": "enemy"}"#);

    // Build with the pipeline
    let pipeline = BuildPipeline::new(ctx);
    let result = pipeline.build().unwrap();

    assert!(result.is_success());
    assert!(result.failed_count() == 0);
}

#[test]
fn test_build_pipeline_with_fail_fast() {
    let (temp, ctx) = create_test_context();

    // Create a build plan with a failing target
    let mut plan = BuildPlan::new();
    plan.add_target(BuildTarget::sprite(
        "good".to_string(),
        create_test_file(temp.path(), "src/pxl/good.pxl", "{}"),
        temp.path().join("build/good.png"),
    ));
    plan.add_target(BuildTarget::sprite(
        "bad".to_string(),
        PathBuf::from("/nonexistent/file.pxl"),
        temp.path().join("build/bad.png"),
    ));

    let pipeline = BuildPipeline::new(ctx).with_fail_fast(true);
    let result = pipeline.build_plan(&plan).unwrap();

    // With fail_fast, should stop after first failure
    // Note: Depending on execution order, we may have 1 or 2 results
    assert!(result.failed_count() >= 1);
}

#[test]
fn test_build_pipeline_dry_run() {
    let (temp, ctx) = create_test_context();

    // Create a source file
    create_test_file(temp.path(), "src/pxl/test.pxl", r#"{"name": "test"}"#);

    let pipeline = BuildPipeline::new(ctx).with_dry_run(true);
    let result = pipeline.build().unwrap();

    assert!(result.is_success());
    // Dry run should skip all targets
    assert_eq!(result.skipped_count(), result.targets.len());
}

#[test]
fn test_build_pipeline_with_filter() {
    let (temp, ctx) = create_test_context();

    // Create multiple sources
    create_test_file(temp.path(), "src/pxl/player.pxl", "{}");
    create_test_file(temp.path(), "src/pxl/enemy.pxl", "{}");

    // Build with filter
    let ctx = ctx.with_filter(vec!["sprite:player".to_string()]);
    let pipeline = BuildPipeline::new(ctx);
    let result = pipeline.build().unwrap();

    assert!(result.is_success());
}

// ============================================================================
// Incremental Build Tests (BST-15)
// ============================================================================

#[test]
fn test_incremental_build_skips_up_to_date() {
    let (temp, ctx) = create_test_context();
    let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
    let output = create_test_file(temp.path(), "build/test.png", "output");

    let target = BuildTarget::sprite("test".to_string(), source, output.clone());

    // First build - record in manifest
    let mut build = IncrementalBuild::new(ctx).with_save_manifest(false);
    build.record_build(&target, &[output]).unwrap();

    // Second check - should not need rebuild
    assert!(!build.needs_rebuild(&target).unwrap());
}

#[test]
fn test_incremental_build_detects_source_change() {
    let (temp, ctx) = create_test_context();
    let source = create_test_file(temp.path(), "src/pxl/test.pxl", "original");
    let output = create_test_file(temp.path(), "build/test.png", "output");

    let target = BuildTarget::sprite("test".to_string(), source, output.clone());

    let mut build = IncrementalBuild::new(ctx).with_save_manifest(false);
    build.record_build(&target, &[output]).unwrap();

    // Modify source
    create_test_file(temp.path(), "src/pxl/test.pxl", "modified");

    assert!(build.needs_rebuild(&target).unwrap());
}

#[test]
fn test_incremental_build_detects_output_missing() {
    let (temp, ctx) = create_test_context();
    let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
    let output = create_test_file(temp.path(), "build/test.png", "output");

    let target = BuildTarget::sprite("test".to_string(), source, output.clone());

    let mut build = IncrementalBuild::new(ctx).with_save_manifest(false);
    build.record_build(&target, &[output.clone()]).unwrap();

    // Delete output
    fs::remove_file(&output).unwrap();

    assert!(build.needs_rebuild(&target).unwrap());
}

#[test]
fn test_incremental_build_force_mode() {
    let (temp, ctx) = create_test_context();
    let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
    let output = create_test_file(temp.path(), "build/test.png", "output");

    let target = BuildTarget::sprite("test".to_string(), source, output.clone());

    let mut build = IncrementalBuild::new(ctx).with_force(true).with_save_manifest(false);
    build.record_build(&target, &[output]).unwrap();

    // Force mode should always rebuild
    assert!(build.needs_rebuild(&target).unwrap());
}

#[test]
fn test_incremental_stats() {
    let mut result = BuildResult::new();
    result.add_result(pixelsrc::build::TargetResult::success(
        "a".to_string(),
        vec![],
        Duration::ZERO,
    ));
    result.add_result(pixelsrc::build::TargetResult::skipped("b".to_string()));
    result.add_result(pixelsrc::build::TargetResult::skipped("c".to_string()));
    result.add_result(pixelsrc::build::TargetResult::failed(
        "d".to_string(),
        "error".to_string(),
        Duration::ZERO,
    ));

    let stats = IncrementalStats::from_result(&result);

    assert_eq!(stats.built, 1);
    assert_eq!(stats.skipped, 2);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.total, 4);
    assert!(stats.had_skips());
    assert!(stats.had_rebuilds());
    assert!((stats.skip_percentage() - 50.0).abs() < 0.001);
}

#[test]
fn test_manifest_persistence() {
    let (temp, ctx) = create_test_context();
    let source = create_test_file(temp.path(), "src/pxl/test.pxl", "content");
    let output = create_test_file(temp.path(), "build/test.png", "output");

    let target = BuildTarget::sprite("test".to_string(), source, output.clone());

    // First session - record and save
    {
        let mut build = IncrementalBuild::new(ctx.clone());
        build.record_build(&target, &[output]).unwrap();
        // Explicitly save the manifest to disk
        build.manifest_mut().save_to_dir(&ctx.out_dir()).unwrap();
    }

    // Second session - load and check
    {
        let build = IncrementalBuild::new(ctx);
        assert!(!build.needs_rebuild(&target).unwrap());
    }
}

// ============================================================================
// Parallel Build Tests (BST-16)
// ============================================================================

#[test]
fn test_parallel_build_empty() {
    let (_temp, ctx) = create_test_context();
    let build = ParallelBuild::new(ctx);

    let result = build.run().unwrap();
    assert!(result.is_success());
    assert_eq!(result.targets.len(), 0);
}

#[test]
fn test_parallel_build_single_job() {
    let (temp, ctx) = create_test_context();
    create_test_file(temp.path(), "src/pxl/a.pxl", "{}");
    create_test_file(temp.path(), "src/pxl/b.pxl", "{}");

    let build = ParallelBuild::new(ctx).with_jobs(1);
    let result = build.run().unwrap();

    assert!(result.is_success());
}

#[test]
fn test_parallel_build_multiple_jobs() {
    let (temp, ctx) = create_test_context();

    // Create multiple source files
    for i in 0..10 {
        create_test_file(temp.path(), &format!("src/pxl/sprite{i}.pxl"), "{}");
    }

    let build = ParallelBuild::new(ctx).with_jobs(4);
    let result = build.run().unwrap();

    assert!(result.is_success());
}

#[test]
fn test_parallel_build_dependency_levels() {
    let (temp, ctx) = create_test_context();

    // Create a plan with dependencies
    let walk_src = create_test_file(temp.path(), "src/pxl/walk.pxl", "{}");
    let run_src = create_test_file(temp.path(), "src/pxl/run.pxl", "{}");

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
fn test_parallel_build_with_failure() {
    let (temp, ctx) = create_test_context();

    let mut plan = BuildPlan::new();
    plan.add_target(BuildTarget::sprite(
        "good".to_string(),
        create_test_file(temp.path(), "src/pxl/good.pxl", "{}"),
        temp.path().join("build/good.png"),
    ));
    plan.add_target(BuildTarget::sprite(
        "bad".to_string(),
        PathBuf::from("/nonexistent.pxl"),
        temp.path().join("build/bad.png"),
    ));

    let build = ParallelBuild::new(ctx).with_jobs(2);
    let result = build.run_plan(&plan).unwrap();

    assert!(!result.is_success());
    assert_eq!(result.failed_count(), 1);
}

#[test]
fn test_parallel_stats() {
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

    assert_eq!(stats.total_targets, 2);
    assert_eq!(stats.workers, 4);
    assert!(stats.levels >= 1);
}

// ============================================================================
// Progress Reporting Tests (BST-17)
// ============================================================================

#[test]
fn test_null_progress_reporter() {
    let reporter = NullProgress::new();

    // Should not panic on any event
    reporter.report(ProgressEvent::BuildStarted { total_targets: 10 });
    reporter.report(ProgressEvent::TargetStarted { target_id: "test".to_string() });
    reporter.report(ProgressEvent::TargetCompleted {
        target_id: "test".to_string(),
        status: TargetStatus::Success,
        duration_ms: 100,
    });
    reporter.report(ProgressEvent::BuildCompleted {
        success: true,
        duration_ms: 500,
        succeeded: 1,
        skipped: 0,
        failed: 0,
    });

    assert!(!reporter.is_verbose());
}

#[test]
fn test_console_progress_build_lifecycle() {
    let output = Arc::new(Mutex::new(Vec::new()));
    let reporter = ConsoleProgress::with_output(TestWriter(Arc::clone(&output)))
        .with_colors(false)
        .with_verbose(true);

    reporter.report(ProgressEvent::BuildStarted { total_targets: 2 });
    reporter.report(ProgressEvent::TargetStarted { target_id: "sprite:player".to_string() });
    reporter.report(ProgressEvent::TargetCompleted {
        target_id: "sprite:player".to_string(),
        status: TargetStatus::Success,
        duration_ms: 150,
    });
    reporter.report(ProgressEvent::TargetStarted { target_id: "sprite:enemy".to_string() });
    reporter.report(ProgressEvent::TargetCompleted {
        target_id: "sprite:enemy".to_string(),
        status: TargetStatus::Skipped,
        duration_ms: 0,
    });
    reporter.report(ProgressEvent::BuildCompleted {
        success: true,
        duration_ms: 200,
        succeeded: 1,
        skipped: 1,
        failed: 0,
    });

    let binding = output.lock().unwrap();
    let text = String::from_utf8_lossy(&binding);
    assert!(text.contains("Building 2 targets"));
    assert!(text.contains("sprite:player"));
    assert!(text.contains("ok"));
    assert!(text.contains("skipped"));
    assert!(text.contains("done"));
}

#[test]
fn test_console_progress_with_failure() {
    let output = Arc::new(Mutex::new(Vec::new()));
    let reporter = ConsoleProgress::with_output(TestWriter(Arc::clone(&output))).with_colors(false);

    reporter.report(ProgressEvent::BuildStarted { total_targets: 1 });
    reporter.report(ProgressEvent::TargetCompleted {
        target_id: "sprite:test".to_string(),
        status: TargetStatus::Failed("file not found".to_string()),
        duration_ms: 50,
    });
    reporter.report(ProgressEvent::BuildCompleted {
        success: false,
        duration_ms: 100,
        succeeded: 0,
        skipped: 0,
        failed: 1,
    });

    let binding = output.lock().unwrap();
    let text = String::from_utf8_lossy(&binding);
    assert!(text.contains("FAILED"));
    assert!(text.contains("file not found"));
    assert!(text.contains("error"));
}

#[test]
fn test_console_progress_warnings_and_errors() {
    let output = Arc::new(Mutex::new(Vec::new()));
    let reporter = ConsoleProgress::with_output(TestWriter(Arc::clone(&output))).with_colors(false);

    reporter.report(ProgressEvent::Warning {
        target_id: Some("sprite:test".to_string()),
        message: "deprecated format".to_string(),
    });
    reporter.report(ProgressEvent::Error {
        target_id: None,
        message: "configuration error".to_string(),
    });

    let binding = output.lock().unwrap();
    let text = String::from_utf8_lossy(&binding);
    assert!(text.contains("warn"));
    assert!(text.contains("deprecated format"));
    assert!(text.contains("error"));
    assert!(text.contains("configuration error"));
}

#[test]
fn test_json_progress_reporter() {
    let output = Arc::new(Mutex::new(Vec::new()));
    let reporter = JsonProgress::with_output(TestWriter(Arc::clone(&output)));

    reporter.report(ProgressEvent::BuildStarted { total_targets: 5 });
    reporter.report(ProgressEvent::TargetCompleted {
        target_id: "sprite:test".to_string(),
        status: TargetStatus::Success,
        duration_ms: 100,
    });
    reporter.report(ProgressEvent::TargetCompleted {
        target_id: "sprite:fail".to_string(),
        status: TargetStatus::Failed("error msg".to_string()),
        duration_ms: 50,
    });
    reporter.report(ProgressEvent::BuildCompleted {
        success: false,
        duration_ms: 200,
        succeeded: 1,
        skipped: 0,
        failed: 1,
    });

    let binding = output.lock().unwrap();
    let text = String::from_utf8_lossy(&binding);

    // Verify JSON structure
    assert!(text.contains(r#""event":"build_started""#));
    assert!(text.contains(r#""total_targets":5"#));
    assert!(text.contains(r#""event":"target_completed""#));
    assert!(text.contains(r#""status":"success""#));
    assert!(text.contains(r#""status":"failed""#));
    assert!(text.contains(r#""error":"error msg""#));
    assert!(text.contains(r#""event":"build_completed""#));
}

#[test]
fn test_progress_tracker() {
    let mut tracker = ProgressTracker::new();

    tracker.start(3);
    assert_eq!(tracker.percentage(), 0.0);
    assert!(!tracker.is_complete());

    tracker.target_started("a");
    assert_eq!(tracker.in_progress().len(), 1);

    tracker.target_completed("a", &TargetStatus::Success);
    assert_eq!(tracker.in_progress().len(), 0);
    assert_eq!(tracker.succeeded(), 1);

    tracker.target_started("b");
    tracker.target_completed("b", &TargetStatus::Skipped);
    assert_eq!(tracker.skipped(), 1);

    tracker.target_started("c");
    tracker.target_completed("c", &TargetStatus::Failed("error".to_string()));
    assert_eq!(tracker.failed(), 1);

    assert!(tracker.is_complete());
    assert!(!tracker.is_success()); // Has failures

    let event = tracker.build_completed_event();
    match event {
        ProgressEvent::BuildCompleted { succeeded, skipped, failed, .. } => {
            assert_eq!(succeeded, 1);
            assert_eq!(skipped, 1);
            assert_eq!(failed, 1);
        }
        _ => panic!("Expected BuildCompleted event"),
    }
}

// ============================================================================
// Build Target Tests
// ============================================================================

#[test]
fn test_target_kind_display() {
    assert_eq!(TargetKind::Sprite.to_string(), "sprite");
    assert_eq!(TargetKind::Atlas.to_string(), "atlas");
    assert_eq!(TargetKind::Animation.to_string(), "animation");
    assert_eq!(TargetKind::AnimationPreview.to_string(), "preview");
    assert_eq!(TargetKind::Export.to_string(), "export");
}

#[test]
fn test_target_filter_matching() {
    let target =
        BuildTarget::atlas("characters".to_string(), vec![], PathBuf::from("build/characters.png"));

    // Exact match
    assert!(target.matches_filter("atlas:characters"));
    assert!(!target.matches_filter("atlas:enemies"));

    // Kind match
    assert!(target.matches_filter("atlas"));
    assert!(!target.matches_filter("sprite"));

    // Wildcard match
    assert!(target.matches_filter("atlas:*"));
    assert!(target.matches_filter("*:characters"));
    assert!(!target.matches_filter("*:enemies"));
}

#[test]
fn test_build_plan_filter() {
    let mut plan = BuildPlan::new();
    plan.add_target(BuildTarget::atlas(
        "characters".to_string(),
        vec![],
        PathBuf::from("build/characters.png"),
    ));
    plan.add_target(BuildTarget::atlas(
        "environment".to_string(),
        vec![],
        PathBuf::from("build/environment.png"),
    ));
    plan.add_target(BuildTarget::sprite(
        "player".to_string(),
        PathBuf::from("src/player.pxl"),
        PathBuf::from("build/player.png"),
    ));

    let filtered = plan.filter(&["atlas".to_string()]);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_build_plan_build_order() {
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

    let order = plan.build_order().unwrap();
    assert_eq!(order.len(), 2);
    // Animation should come before preview
    assert_eq!(order[0].id, "animation:walk");
    assert_eq!(order[1].id, "preview:walk");
}

// ============================================================================
// Build Result Tests
// ============================================================================

#[test]
fn test_build_result_summary_success() {
    let mut result = BuildResult::new();
    result.add_result(pixelsrc::build::TargetResult::success(
        "atlas:main".to_string(),
        vec![PathBuf::from("main.png")],
        Duration::from_millis(100),
    ));
    result.add_result(pixelsrc::build::TargetResult::skipped("atlas:ui".to_string()));

    let summary = result.with_duration(Duration::from_millis(150)).summary();
    assert!(summary.contains("Build succeeded"));
    assert!(summary.contains("1 built"));
    assert!(summary.contains("1 skipped"));
}

#[test]
fn test_build_result_summary_failure() {
    let mut result = BuildResult::new();
    result.add_result(pixelsrc::build::TargetResult::success(
        "a".to_string(),
        vec![],
        Duration::ZERO,
    ));
    result.add_result(pixelsrc::build::TargetResult::failed(
        "b".to_string(),
        "file not found".to_string(),
        Duration::ZERO,
    ));

    let summary = result.summary();
    assert!(summary.contains("Build failed"));
    assert!(summary.contains("1 failed"));
}

#[test]
fn test_build_result_outputs() {
    let mut result = BuildResult::new();
    result.add_result(pixelsrc::build::TargetResult::success(
        "a".to_string(),
        vec![PathBuf::from("a.png"), PathBuf::from("a.json")],
        Duration::ZERO,
    ));
    result.add_result(pixelsrc::build::TargetResult::success(
        "b".to_string(),
        vec![PathBuf::from("b.png")],
        Duration::ZERO,
    ));

    let outputs = result.all_outputs();
    assert_eq!(outputs.len(), 3);
}

// ============================================================================
// Manifest Tests
// ============================================================================

#[test]
fn test_manifest_save_and_load() {
    let temp = TempDir::new().unwrap();
    let source = create_test_file(temp.path(), "src/test.pxl", "content");
    let output = temp.path().join("build/test.png");

    let mut manifest = BuildManifest::new();
    manifest.record_build("sprite:test", &[source], &[output]).unwrap();
    manifest.save_to_dir(temp.path()).unwrap();

    let loaded = BuildManifest::load_from_dir(temp.path()).unwrap().unwrap();
    assert_eq!(loaded.len(), 1);
    assert!(loaded.get_target("sprite:test").is_some());
}

#[test]
fn test_manifest_multiple_targets() {
    let temp = TempDir::new().unwrap();

    let mut manifest = BuildManifest::new();

    // Record multiple targets
    for i in 0..5 {
        let source =
            create_test_file(temp.path(), &format!("src/sprite{i}.pxl"), &format!("content{i}"));
        manifest
            .record_build(
                &format!("sprite:sprite{i}"),
                &[source],
                &[temp.path().join(format!("build/sprite{i}.png"))],
            )
            .unwrap();
    }

    assert_eq!(manifest.len(), 5);

    // Clear and verify
    manifest.clear();
    assert!(manifest.is_empty());
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn test_full_build_with_exports() {
    let (temp, ctx) = create_test_context();

    // Create source files
    create_test_file(temp.path(), "src/pxl/player.pxl", r#"{"name": "player"}"#);
    create_test_file(temp.path(), "src/pxl/enemy.pxl", r#"{"name": "enemy"}"#);

    // Create a plan with atlases and exports
    let mut plan = BuildPlan::new();
    plan.add_target(BuildTarget::atlas(
        "characters".to_string(),
        vec![temp.path().join("src/pxl/player.pxl"), temp.path().join("src/pxl/enemy.pxl")],
        temp.path().join("build/characters.png"),
    ));

    // Add export targets that depend on the atlas
    plan.add_target(
        BuildTarget::export(
            "characters".to_string(),
            "godot".to_string(),
            temp.path().join("build/godot/characters.tres"),
        )
        .with_dependency("atlas:characters".to_string()),
    );
    plan.add_target(
        BuildTarget::export(
            "characters".to_string(),
            "unity".to_string(),
            temp.path().join("build/unity/characters.asset"),
        )
        .with_dependency("atlas:characters".to_string()),
    );
    plan.add_target(
        BuildTarget::export(
            "characters".to_string(),
            "libgdx".to_string(),
            temp.path().join("build/libgdx/characters.atlas"),
        )
        .with_dependency("atlas:characters".to_string()),
    );

    // Build with parallel execution
    let build = ParallelBuild::new(ctx).with_jobs(2);
    let result = build.run_plan(&plan).unwrap();

    assert!(result.is_success());
    assert_eq!(result.targets.len(), 4);
}

#[test]
fn test_incremental_build_workflow() {
    let (temp, ctx) = create_test_context();

    // Create initial sources
    let source = create_test_file(temp.path(), "src/pxl/test.pxl", "original content");
    let output = create_test_file(temp.path(), "build/test.png", "output");

    let target = BuildTarget::sprite("test".to_string(), source, output.clone());

    // First build
    let mut build = IncrementalBuild::new(ctx);
    build.record_build(&target, &[output.clone()]).unwrap();

    // Should not need rebuild
    assert!(!build.needs_rebuild(&target).unwrap());

    // Modify source
    create_test_file(temp.path(), "src/pxl/test.pxl", "modified content");

    // Should need rebuild
    assert!(build.needs_rebuild(&target).unwrap());

    // Record new build
    build.record_build(&target, &[output]).unwrap();

    // Should not need rebuild again
    assert!(!build.needs_rebuild(&target).unwrap());
}

#[test]
fn test_parallel_build_with_complex_dependencies() {
    let (temp, ctx) = create_test_context();

    // Create a complex dependency graph:
    // atlas:characters -> export:godot:characters
    //                  -> export:unity:characters
    // atlas:environment -> export:godot:environment
    // animation:walk -> preview:walk

    let mut plan = BuildPlan::new();

    // Atlas targets (no dependencies)
    plan.add_target(BuildTarget::atlas(
        "characters".to_string(),
        vec![create_test_file(temp.path(), "src/pxl/player.pxl", "{}")],
        temp.path().join("build/characters.png"),
    ));
    plan.add_target(BuildTarget::atlas(
        "environment".to_string(),
        vec![create_test_file(temp.path(), "src/pxl/tree.pxl", "{}")],
        temp.path().join("build/environment.png"),
    ));

    // Animation target (no dependencies)
    plan.add_target(BuildTarget::animation(
        "walk".to_string(),
        create_test_file(temp.path(), "src/pxl/walk.pxl", "{}"),
        temp.path().join("build/walk.png"),
    ));

    // Export targets (depend on atlases)
    plan.add_target(
        BuildTarget::export(
            "characters".to_string(),
            "godot".to_string(),
            temp.path().join("build/godot/characters.tres"),
        )
        .with_dependency("atlas:characters".to_string()),
    );
    plan.add_target(
        BuildTarget::export(
            "characters".to_string(),
            "unity".to_string(),
            temp.path().join("build/unity/characters.asset"),
        )
        .with_dependency("atlas:characters".to_string()),
    );
    plan.add_target(
        BuildTarget::export(
            "environment".to_string(),
            "godot".to_string(),
            temp.path().join("build/godot/environment.tres"),
        )
        .with_dependency("atlas:environment".to_string()),
    );

    // Animation preview (depends on animation)
    plan.add_target(BuildTarget::animation_preview(
        "walk".to_string(),
        create_test_file(temp.path(), "src/pxl/walk.pxl", "{}"),
        temp.path().join("build/walk.gif"),
    ));

    // Build with parallel execution
    let build = ParallelBuild::new(ctx).with_jobs(4);
    let result = build.run_plan(&plan).unwrap();

    assert!(result.is_success());
    assert_eq!(result.targets.len(), 7);
}

#[test]
fn test_build_builder_api() {
    let (temp, ctx) = create_test_context();
    create_test_file(temp.path(), "src/pxl/test.pxl", "{}");

    let result =
        Build::new().context(ctx).dry_run(true).verbose(false).fail_fast(true).run().unwrap();

    assert!(result.is_success());
}
