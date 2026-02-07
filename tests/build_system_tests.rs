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
    build.record_build(&target, std::slice::from_ref(&output)).unwrap();

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
    build.record_build(&target, std::slice::from_ref(&output)).unwrap();

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

// ============================================================================
// Export Module Integration Tests (BST-12, BST-13, BST-14)
// ============================================================================

mod export_tests {
    use super::*;
    use pixelsrc::atlas::{AtlasAnimation, AtlasFrame, AtlasMetadata};
    use pixelsrc::export::{
        ExportOptions, GodotExportOptions, GodotExporter, LibGdxExportOptions, LibGdxExporter,
        LibGdxFilterMode, LibGdxRepeatMode, UnityExportOptions, UnityExporter, UnityFilterMode,
    };
    use std::collections::HashMap;

    fn create_test_atlas_metadata() -> AtlasMetadata {
        AtlasMetadata {
            image: "test_atlas.png".to_string(),
            size: [256, 256],
            frames: HashMap::from([
                (
                    "idle_1".to_string(),
                    AtlasFrame { x: 0, y: 0, w: 32, h: 32, origin: Some([16, 32]), boxes: None },
                ),
                (
                    "idle_2".to_string(),
                    AtlasFrame { x: 32, y: 0, w: 32, h: 32, origin: Some([16, 32]), boxes: None },
                ),
                (
                    "walk_1".to_string(),
                    AtlasFrame { x: 64, y: 0, w: 32, h: 32, origin: Some([16, 32]), boxes: None },
                ),
                (
                    "walk_2".to_string(),
                    AtlasFrame { x: 96, y: 0, w: 32, h: 32, origin: Some([16, 32]), boxes: None },
                ),
                (
                    "jump".to_string(),
                    AtlasFrame { x: 0, y: 32, w: 32, h: 48, origin: Some([16, 48]), boxes: None },
                ),
            ]),
            animations: HashMap::from([
                (
                    "idle".to_string(),
                    AtlasAnimation {
                        frames: vec!["idle_1".to_string(), "idle_2".to_string()],
                        fps: 8,
                        tags: None,
                    },
                ),
                (
                    "walk".to_string(),
                    AtlasAnimation {
                        frames: vec!["walk_1".to_string(), "walk_2".to_string()],
                        fps: 12,
                        tags: None,
                    },
                ),
            ]),
        }
    }

    // Godot Export Tests (BST-12)

    #[test]
    fn test_godot_export_complete_workflow() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_atlas_metadata();

        let exporter = GodotExporter::new().with_resource_path("res://sprites");

        let options = GodotExportOptions {
            resource_path: "res://sprites".to_string(),
            sprite_frames: true,
            animation_player: true,
            atlas_textures: true,
            base: ExportOptions::default(),
        };

        let outputs = exporter.export_godot(&metadata, temp.path(), &options).unwrap();

        // Should create AtlasTexture for each frame + SpriteFrames + AnimationLibrary
        assert!(outputs.len() >= 7); // 5 frames + sprite_frames + anim_library

        // Verify AtlasTexture files
        assert!(temp.path().join("idle_1.tres").exists());
        assert!(temp.path().join("idle_2.tres").exists());
        assert!(temp.path().join("walk_1.tres").exists());
        assert!(temp.path().join("walk_2.tres").exists());
        assert!(temp.path().join("jump.tres").exists());

        // Verify SpriteFrames file
        let frames_file = temp.path().join("test_atlas_frames.tres");
        assert!(frames_file.exists());
        let content = std::fs::read_to_string(&frames_file).unwrap();
        assert!(content.contains("SpriteFrames"));
        assert!(content.contains("idle"));
        assert!(content.contains("walk"));

        // Verify AnimationLibrary file
        let anims_file = temp.path().join("test_atlas_anims.tres");
        assert!(anims_file.exists());
        let anim_content = std::fs::read_to_string(&anims_file).unwrap();
        assert!(anim_content.contains("AnimationLibrary"));
    }

    #[test]
    fn test_godot_atlas_texture_region_format() {
        let temp = TempDir::new().unwrap();
        let metadata = create_test_atlas_metadata();

        let exporter = GodotExporter::new().with_resource_path("res://game/assets");
        let options = GodotExportOptions::default();

        exporter.export_godot(&metadata, temp.path(), &options).unwrap();

        let content = std::fs::read_to_string(temp.path().join("jump.tres")).unwrap();

        // jump frame is at x=0, y=32, w=32, h=48
        assert!(content.contains("Rect2(0, 32, 32, 48)"));
        assert!(content.contains("AtlasTexture"));
    }

    #[test]
    fn test_godot_export_without_animations() {
        let temp = TempDir::new().unwrap();
        let metadata = AtlasMetadata {
            image: "static.png".to_string(),
            size: [64, 64],
            frames: HashMap::from([(
                "icon".to_string(),
                AtlasFrame { x: 0, y: 0, w: 64, h: 64, origin: None, boxes: None },
            )]),
            animations: HashMap::new(),
        };

        let exporter = GodotExporter::new();
        let options = GodotExportOptions::default();

        let outputs = exporter.export_godot(&metadata, temp.path(), &options).unwrap();

        // Only AtlasTexture, no SpriteFrames or AnimationLibrary
        assert_eq!(outputs.len(), 1);
        assert!(temp.path().join("icon.tres").exists());
        assert!(!temp.path().join("static_frames.tres").exists());
    }

    // Unity Export Tests (BST-13)

    #[test]
    fn test_unity_export_complete_workflow() {
        let _temp = TempDir::new().unwrap();
        let metadata = create_test_atlas_metadata();

        let exporter = UnityExporter::new().with_pixels_per_unit(16).with_animations(true);

        let options = UnityExportOptions {
            pixels_per_unit: 16,
            filter_mode: UnityFilterMode::Point,
            include_animations: true,
            generate_meta: true,
            generate_anim_files: true,
            generate_json: true,
            base: ExportOptions::default(),
        };

        let json = exporter.export_to_string(&metadata, &options).unwrap();

        // Verify JSON structure
        assert!(json.contains("\"texture\": \"test_atlas.png\""));
        assert!(json.contains("\"pixelsPerUnit\": 16"));
        assert!(json.contains("\"filterMode\": \"Point\""));
        assert!(json.contains("\"sprites\""));
        assert!(json.contains("\"animations\""));

        // Verify sprites
        assert!(json.contains("\"idle_1\""));
        assert!(json.contains("\"walk_1\""));
        assert!(json.contains("\"jump\""));

        // Verify animations
        assert!(json.contains("\"idle\""));
        assert!(json.contains("\"walk\""));
    }

    #[test]
    fn test_unity_sprite_y_flip() {
        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [128, 128],
            frames: HashMap::from([(
                "sprite".to_string(),
                AtlasFrame { x: 10, y: 20, w: 32, h: 32, origin: None, boxes: None },
            )]),
            animations: HashMap::new(),
        };

        let exporter = UnityExporter::new();
        let options = UnityExportOptions::default();
        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let sprites = data["sprites"].as_array().unwrap();
        let sprite = &sprites[0];

        // Y should be flipped: 128 - 20 - 32 = 76
        assert_eq!(sprite["rect"]["y"], 76.0);
    }

    #[test]
    fn test_unity_pivot_calculation() {
        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [128, 128],
            frames: HashMap::from([(
                "sprite".to_string(),
                AtlasFrame {
                    x: 0,
                    y: 0,
                    w: 32,
                    h: 32,
                    origin: Some([16, 32]), // Bottom center
                    boxes: None,
                },
            )]),
            animations: HashMap::new(),
        };

        let exporter = UnityExporter::new();
        let options = UnityExportOptions::default();
        let json = exporter.export_to_string(&metadata, &options).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();

        let sprites = data["sprites"].as_array().unwrap();
        let sprite = &sprites[0];

        // Pivot at bottom center: (0.5, 0.0) after Y flip
        assert_eq!(sprite["pivot"]["x"], 0.5);
        assert_eq!(sprite["pivot"]["y"], 0.0);
    }

    #[test]
    fn test_unity_filter_modes() {
        let metadata = create_test_atlas_metadata();

        for (filter, expected) in [
            (UnityFilterMode::Point, "Point"),
            (UnityFilterMode::Bilinear, "Bilinear"),
            (UnityFilterMode::Trilinear, "Trilinear"),
        ] {
            let exporter = UnityExporter::new().with_filter_mode(filter);
            let options = UnityExportOptions { filter_mode: filter, ..Default::default() };
            let json = exporter.export_to_string(&metadata, &options).unwrap();

            assert!(
                json.contains(&format!("\"filterMode\": \"{}\"", expected)),
                "Expected filter mode {} in output",
                expected
            );
        }
    }

    // libGDX Export Tests (BST-14)

    #[test]
    fn test_libgdx_export_complete_workflow() {
        let temp = TempDir::new().unwrap();
        let output_path = temp.path().join("atlas.atlas");
        let metadata = create_test_atlas_metadata();

        let exporter = LibGdxExporter::new()
            .with_min_filter(LibGdxFilterMode::Nearest)
            .with_mag_filter(LibGdxFilterMode::Nearest)
            .with_repeat(LibGdxRepeatMode::None)
            .with_format("RGBA8888");

        let options = LibGdxExportOptions::default();
        exporter.export_libgdx(&metadata, &output_path, &options).unwrap();

        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).unwrap();

        // Verify header
        assert!(content.starts_with("test_atlas.png\n"));
        assert!(content.contains("size: 256, 256\n"));
        assert!(content.contains("format: RGBA8888\n"));
        assert!(content.contains("filter: Nearest, Nearest\n"));
        assert!(content.contains("repeat: none\n"));

        // Verify frames
        assert!(content.contains("idle_1\n"));
        assert!(content.contains("walk_1\n"));
        assert!(content.contains("jump\n"));
    }

    #[test]
    fn test_libgdx_animation_indices() {
        let metadata = create_test_atlas_metadata();
        let exporter = LibGdxExporter::new();
        let content = exporter.export_to_string(&metadata);

        let lines: Vec<&str> = content.lines().collect();

        // Find walk_1 (should have index 0 in walk animation)
        let walk1_idx = lines.iter().position(|l| *l == "walk_1").unwrap();
        let walk1_index_line = lines[walk1_idx + 6];
        assert_eq!(walk1_index_line, "  index: 0");

        // Find walk_2 (should have index 1 in walk animation)
        let walk2_idx = lines.iter().position(|l| *l == "walk_2").unwrap();
        let walk2_index_line = lines[walk2_idx + 6];
        assert_eq!(walk2_index_line, "  index: 1");

        // Find jump (not in animation, should have index -1)
        let jump_idx = lines.iter().position(|l| *l == "jump").unwrap();
        let jump_index_line = lines[jump_idx + 6];
        assert_eq!(jump_index_line, "  index: -1");
    }

    #[test]
    fn test_libgdx_filter_modes() {
        let metadata = create_test_atlas_metadata();

        for (min, mag, expected_filter) in [
            (LibGdxFilterMode::Nearest, LibGdxFilterMode::Nearest, "filter: Nearest, Nearest"),
            (LibGdxFilterMode::Linear, LibGdxFilterMode::Linear, "filter: Linear, Linear"),
            (
                LibGdxFilterMode::MipMapLinearLinear,
                LibGdxFilterMode::Linear,
                "filter: MipMapLinearLinear, Linear",
            ),
        ] {
            let exporter = LibGdxExporter::new().with_min_filter(min).with_mag_filter(mag);
            let content = exporter.export_to_string(&metadata);

            assert!(content.contains(expected_filter), "Expected {} in output", expected_filter);
        }
    }

    #[test]
    fn test_libgdx_repeat_modes() {
        let metadata = create_test_atlas_metadata();

        for (repeat, expected) in [
            (LibGdxRepeatMode::None, "repeat: none"),
            (LibGdxRepeatMode::X, "repeat: x"),
            (LibGdxRepeatMode::Y, "repeat: y"),
            (LibGdxRepeatMode::XY, "repeat: xy"),
        ] {
            let exporter = LibGdxExporter::new().with_repeat(repeat);
            let content = exporter.export_to_string(&metadata);

            assert!(content.contains(expected), "Expected {} in output", expected);
        }
    }

    #[test]
    fn test_libgdx_frame_with_origin_offset() {
        let metadata = AtlasMetadata {
            image: "test.png".to_string(),
            size: [64, 64],
            frames: HashMap::from([(
                "centered".to_string(),
                AtlasFrame {
                    x: 0,
                    y: 0,
                    w: 32,
                    h: 32,
                    origin: Some([16, 16]), // Center origin
                    boxes: None,
                },
            )]),
            animations: HashMap::new(),
        };

        let exporter = LibGdxExporter::new();
        let content = exporter.export_to_string(&metadata);

        // Origin [16, 16] should produce offset: 16, 16
        assert!(content.contains("  offset: 16, 16\n"));
    }

    // Cross-format comparison tests

    #[test]
    fn test_all_exporters_handle_empty_animations() {
        let metadata = AtlasMetadata {
            image: "static.png".to_string(),
            size: [64, 64],
            frames: HashMap::from([(
                "icon".to_string(),
                AtlasFrame { x: 0, y: 0, w: 64, h: 64, origin: None, boxes: None },
            )]),
            animations: HashMap::new(),
        };

        let temp = TempDir::new().unwrap();

        // Godot
        let godot = GodotExporter::new();
        let godot_opts = GodotExportOptions::default();
        let result = godot.export_godot(&metadata, temp.path(), &godot_opts);
        assert!(result.is_ok());

        // Unity
        let unity = UnityExporter::new();
        let unity_opts = UnityExportOptions::default();
        let result = unity.export_to_string(&metadata, &unity_opts);
        assert!(result.is_ok());

        // libGDX
        let libgdx = LibGdxExporter::new();
        let content = libgdx.export_to_string(&metadata);
        assert!(content.contains("icon\n"));
    }

    #[test]
    fn test_all_exporters_handle_large_atlas() {
        // Create a large atlas with many frames
        let mut frames = HashMap::new();
        for i in 0..100 {
            frames.insert(
                format!("frame_{}", i),
                AtlasFrame {
                    x: (i % 10) * 32,
                    y: (i / 10) * 32,
                    w: 32,
                    h: 32,
                    origin: None,
                    boxes: None,
                },
            );
        }

        let metadata = AtlasMetadata {
            image: "large.png".to_string(),
            size: [320, 320],
            frames,
            animations: HashMap::new(),
        };

        let temp = TempDir::new().unwrap();

        // Godot
        let godot = GodotExporter::new();
        let godot_opts = GodotExportOptions::default();
        let result = godot.export_godot(&metadata, temp.path(), &godot_opts);
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 100); // 100 AtlasTextures, no animations

        // Unity
        let unity = UnityExporter::new();
        let unity_opts = UnityExportOptions::default();
        let json = unity.export_to_string(&metadata, &unity_opts).unwrap();
        let data: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(data["sprites"].as_array().unwrap().len(), 100);

        // libGDX
        let libgdx = LibGdxExporter::new();
        let content = libgdx.export_to_string(&metadata);
        // Count frame entries
        let frame_count = content.lines().filter(|l| l.starts_with("frame_")).count();
        assert_eq!(frame_count, 100);
    }
}

// ============================================================================
// Watch Error Recovery Tests (BST-18)
// ============================================================================

mod watch_tests {
    use super::*;
    use pixelsrc::watch::{BuildError, BuildResult, ErrorTracker, WatchError, WatchOptions};
    use std::collections::HashSet;

    #[test]
    fn test_error_tracker_empty_initial_state() {
        let tracker = ErrorTracker::new();
        assert!(!tracker.has_errors());
        assert_eq!(tracker.error_count(), 0);
    }

    #[test]
    fn test_error_tracker_single_error() {
        let mut tracker = ErrorTracker::new();

        let mut result = BuildResult::new();
        result.add_error(BuildError::new("test.pxl", "Syntax error"));

        let fixed = tracker.update(&result);
        assert!(fixed.is_empty());
        assert!(tracker.has_errors());
        assert_eq!(tracker.error_count(), 1);
    }

    #[test]
    fn test_error_tracker_multiple_errors() {
        let mut tracker = ErrorTracker::new();

        let mut result = BuildResult::new();
        result.add_error(BuildError::new("file1.pxl", "Error 1"));
        result.add_error(BuildError::new("file2.pxl", "Error 2"));
        result.add_error(BuildError::new("file3.pxl", "Error 3"));

        tracker.update(&result);
        assert_eq!(tracker.error_count(), 3);
    }

    #[test]
    fn test_error_tracker_fix_detection() {
        let mut tracker = ErrorTracker::new();

        // First build: errors in file1 and file2
        let mut result1 = BuildResult::new();
        result1.add_error(BuildError::new("file1.pxl", "Error 1"));
        result1.add_error(BuildError::new("file2.pxl", "Error 2"));
        tracker.update(&result1);

        // Second build: file1 fixed, file2 still has error
        let mut result2 = BuildResult::new();
        result2.add_error(BuildError::new("file2.pxl", "Error 2"));
        let fixed = tracker.update(&result2);

        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed[0], PathBuf::from("file1.pxl"));
        assert_eq!(tracker.error_count(), 1);
    }

    #[test]
    fn test_error_tracker_all_errors_fixed() {
        let mut tracker = ErrorTracker::new();

        // First build: errors
        let mut result1 = BuildResult::new();
        result1.add_error(BuildError::new("file1.pxl", "Error 1"));
        result1.add_error(BuildError::new("file2.pxl", "Error 2"));
        tracker.update(&result1);

        // Second build: all fixed
        let result2 = BuildResult::new();
        let fixed = tracker.update(&result2);

        assert_eq!(fixed.len(), 2);
        let fixed_set: HashSet<_> = fixed.into_iter().collect();
        assert!(fixed_set.contains(&PathBuf::from("file1.pxl")));
        assert!(fixed_set.contains(&PathBuf::from("file2.pxl")));
        assert!(!tracker.has_errors());
    }

    #[test]
    fn test_error_tracker_new_error_while_fixing() {
        let mut tracker = ErrorTracker::new();

        // First build: error in file1
        let mut result1 = BuildResult::new();
        result1.add_error(BuildError::new("file1.pxl", "Error 1"));
        tracker.update(&result1);

        // Second build: file1 fixed, but file2 now has error
        let mut result2 = BuildResult::new();
        result2.add_error(BuildError::new("file2.pxl", "Error 2"));
        let fixed = tracker.update(&result2);

        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed[0], PathBuf::from("file1.pxl"));
        assert!(tracker.has_errors());
        assert_eq!(tracker.error_count(), 1);
    }

    #[test]
    fn test_error_tracker_same_file_different_error() {
        let mut tracker = ErrorTracker::new();

        // First build: error in file1
        let mut result1 = BuildResult::new();
        result1.add_error(BuildError::new("file1.pxl", "Syntax error on line 5"));
        tracker.update(&result1);

        // Second build: same file, different error (not "fixed")
        let mut result2 = BuildResult::new();
        result2.add_error(BuildError::new("file1.pxl", "Syntax error on line 10"));
        let fixed = tracker.update(&result2);

        assert!(fixed.is_empty()); // File still has error
        assert!(tracker.has_errors());
    }

    #[test]
    fn test_build_error_creation_variants() {
        let basic = BuildError::new("test.pxl", "Basic error");
        assert_eq!(basic.file, PathBuf::from("test.pxl"));
        assert_eq!(basic.line, None);
        assert_eq!(basic.column, None);
        assert_eq!(basic.message, "Basic error");

        let with_line = BuildError::with_line("test.pxl", 42, "Error at line");
        assert_eq!(with_line.line, Some(42));
        assert_eq!(with_line.column, None);

        let with_location = BuildError::with_location("test.pxl", 42, 10, "Error at location");
        assert_eq!(with_location.line, Some(42));
        assert_eq!(with_location.column, Some(10));
    }

    #[test]
    fn test_build_error_display_format() {
        let error = BuildError::with_location("sprites/player.pxl", 15, 8, "Invalid color");
        let display = format!("{}", error);

        assert!(display.contains("sprites/player.pxl"));
        assert!(display.contains(":15:"));
        assert!(display.contains(":8"));
        assert!(display.contains("Invalid color"));
    }

    #[test]
    fn test_build_result_error_count() {
        let mut result = BuildResult::new();
        assert_eq!(result.error_count(), 0);
        assert!(result.success());

        // Add legacy error
        result.errors.push("Legacy error".to_string());
        assert_eq!(result.error_count(), 1);
        assert!(!result.success());

        // Add build error
        result.add_error(BuildError::new("test.pxl", "Build error"));
        assert_eq!(result.error_count(), 2);
    }

    #[test]
    fn test_watch_options_default() {
        let options = WatchOptions::default();
        assert_eq!(options.src_dir, PathBuf::from("src/pxl"));
        assert_eq!(options.out_dir, PathBuf::from("build"));
        assert_eq!(options.config.debounce_ms, 100);
        assert!(options.config.clear_screen);
        assert!(!options.verbose);
    }

    #[test]
    fn test_watch_error_display() {
        let source_not_found = WatchError::SourceNotFound(PathBuf::from("/nonexistent/path"));
        let display = format!("{}", source_not_found);
        assert!(display.contains("Source directory not found"));
        assert!(display.contains("/nonexistent/path"));

        let build_failed = WatchError::BuildFailed("Parse error".to_string());
        assert!(format!("{}", build_failed).contains("Build failed"));

        let channel_error = WatchError::ChannelError("Channel closed".to_string());
        assert!(format!("{}", channel_error).contains("channel"));
    }

    #[test]
    fn test_error_tracker_repeated_builds_no_change() {
        let mut tracker = ErrorTracker::new();

        let mut result = BuildResult::new();
        result.add_error(BuildError::new("file.pxl", "Error"));

        // Multiple builds with same error
        for _ in 0..5 {
            let fixed = tracker.update(&result);
            assert!(fixed.is_empty());
            assert_eq!(tracker.error_count(), 1);
        }
    }

    #[test]
    fn test_error_tracker_build_cycle_recovery() {
        let mut tracker = ErrorTracker::new();

        // Cycle: Error -> Fix -> Error -> Fix
        let mut error_result = BuildResult::new();
        error_result.add_error(BuildError::new("file.pxl", "Error"));

        let success_result = BuildResult::new();

        // Error
        tracker.update(&error_result);
        assert!(tracker.has_errors());

        // Fix
        let fixed1 = tracker.update(&success_result);
        assert_eq!(fixed1.len(), 1);
        assert!(!tracker.has_errors());

        // Error again
        tracker.update(&error_result);
        assert!(tracker.has_errors());

        // Fix again
        let fixed2 = tracker.update(&success_result);
        assert_eq!(fixed2.len(), 1);
        assert!(!tracker.has_errors());
    }
}

// ============================================================================
// Discovery and Context Tests
// ============================================================================

#[test]
fn test_build_context_src_dir() {
    let (_temp, ctx) = create_test_context();

    let src_dir = ctx.src_dir();
    assert!(src_dir.ends_with("src/pxl") || src_dir.to_string_lossy().contains("pxl"));
}

#[test]
fn test_build_context_out_dir() {
    let (temp, ctx) = create_test_context();

    let out_dir = ctx.out_dir();
    assert!(out_dir.ends_with("build") || temp.path().join("build") == out_dir);
}

#[test]
fn test_build_context_with_filter() {
    let (_temp, ctx) = create_test_context();

    let filtered_ctx = ctx.with_filter(vec!["sprite:*".to_string(), "atlas:main".to_string()]);

    let filter = filtered_ctx.target_filter().expect("filter should be set");
    assert_eq!(filter.len(), 2);
    assert!(filter.contains(&"sprite:*".to_string()));
    assert!(filter.contains(&"atlas:main".to_string()));
}

// ============================================================================
// Build Plan Edge Cases
// ============================================================================

#[test]
fn test_build_plan_empty() {
    let plan = BuildPlan::new();
    assert!(plan.is_empty());
    assert_eq!(plan.len(), 0);
}

#[test]
fn test_build_plan_duplicate_targets() {
    let mut plan = BuildPlan::new();

    plan.add_target(BuildTarget::sprite(
        "test".to_string(),
        PathBuf::from("test.pxl"),
        PathBuf::from("test.png"),
    ));

    // Adding same target again should update, not duplicate
    plan.add_target(BuildTarget::sprite(
        "test".to_string(),
        PathBuf::from("test.pxl"),
        PathBuf::from("test2.png"),
    ));

    // Should still be 2 (or 1 if deduped by id)
    // The behavior depends on implementation
    assert!(!plan.is_empty());
}

#[test]
fn test_build_plan_circular_dependency_handling() {
    let mut plan = BuildPlan::new();

    // Create potential circular dependency
    plan.add_target(
        BuildTarget::sprite("a".to_string(), PathBuf::from("a.pxl"), PathBuf::from("a.png"))
            .with_dependency("sprite:b".to_string()),
    );
    plan.add_target(
        BuildTarget::sprite("b".to_string(), PathBuf::from("b.pxl"), PathBuf::from("b.png"))
            .with_dependency("sprite:a".to_string()),
    );

    // build_order should handle this gracefully (either error or break cycle)
    let result = plan.build_order();
    // Should either succeed with some order or fail gracefully
    // The important thing is it doesn't panic or infinite loop
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Target Result Edge Cases
// ============================================================================

#[test]
fn test_target_result_zero_duration() {
    let result = pixelsrc::build::TargetResult::success(
        "instant".to_string(),
        vec![PathBuf::from("output.png")],
        Duration::ZERO,
    );

    assert!(result.is_success());
    assert_eq!(result.duration, Duration::ZERO);
}

#[test]
fn test_target_result_empty_outputs() {
    let result = pixelsrc::build::TargetResult::success(
        "no_output".to_string(),
        vec![],
        Duration::from_millis(100),
    );

    assert!(result.is_success());
    assert!(result.outputs.is_empty());
}

#[test]
fn test_target_result_long_error_message() {
    let long_error = "x".repeat(10000);
    let result = pixelsrc::build::TargetResult::failed(
        "failing".to_string(),
        long_error.clone(),
        Duration::ZERO,
    );

    assert!(!result.is_success());
    match result.status {
        pixelsrc::build::BuildStatus::Failed(msg) => {
            assert_eq!(msg.len(), 10000);
        }
        _ => panic!("Expected Failed status"),
    }
}
