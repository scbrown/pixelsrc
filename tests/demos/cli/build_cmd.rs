//! Build Command Demo Tests
//!
//! Demonstrates the `pxl build` command functionality for building
//! pixelsrc projects from pxl.toml configuration.

use pixelsrc::build::{BuildPlan, BuildResult, BuildStatus, TargetResult};
use std::path::PathBuf;
use std::time::Duration;

// ============================================================================
// Build Result Tests
// ============================================================================

/// @demo cli/build#result_success
/// @title Successful Build Result
/// @description Build result tracks successful target completions.
#[test]
fn test_build_result_success() {
    let mut result = BuildResult::new();

    // Add successful target
    let target_result = TargetResult::success(
        "test_sprite".to_string(),
        vec![PathBuf::from("dist/test_sprite.png")],
        Duration::from_millis(50),
    );
    result.add_result(target_result);

    assert!(result.is_success(), "Result should be success");
    assert_eq!(result.success_count(), 1, "Should have 1 success");
    assert_eq!(result.failed_count(), 0, "Should have 0 failures");
}

/// @demo cli/build#result_failure
/// @title Failed Build Result
/// @description Build result tracks failed targets with error messages.
#[test]
fn test_build_result_failure() {
    let mut result = BuildResult::new();

    // Add failed target
    let target_result = TargetResult::failed(
        "broken_sprite".to_string(),
        "Palette 'missing' not found".to_string(),
        Duration::from_millis(10),
    );
    result.add_result(target_result);

    assert!(!result.is_success(), "Result should not be success");
    assert_eq!(result.success_count(), 0, "Should have 0 successes");
    assert_eq!(result.failed_count(), 1, "Should have 1 failure");
}

/// @demo cli/build#result_mixed
/// @title Mixed Build Results
/// @description Build tracks both successful and failed targets.
#[test]
fn test_build_result_mixed() {
    let mut result = BuildResult::new();

    // Add successful target
    result.add_result(TargetResult::success(
        "sprite_a".to_string(),
        vec![PathBuf::from("dist/sprite_a.png")],
        Duration::from_millis(30),
    ));

    // Add failed target
    result.add_result(TargetResult::failed(
        "sprite_b".to_string(),
        "Parse error".to_string(),
        Duration::from_millis(5),
    ));

    // Add another success
    result.add_result(TargetResult::success(
        "sprite_c".to_string(),
        vec![PathBuf::from("dist/sprite_c.png")],
        Duration::from_millis(25),
    ));

    assert!(!result.is_success(), "Result should not be success (has failures)");
    assert_eq!(result.success_count(), 2, "Should have 2 successes");
    assert_eq!(result.failed_count(), 1, "Should have 1 failure");
}

/// @demo cli/build#result_skipped
/// @title Skipped Build Targets
/// @description Build can skip targets (e.g., up-to-date, filtered out).
#[test]
fn test_build_result_skipped() {
    let mut result = BuildResult::new();

    result.add_result(TargetResult::skipped("cached_sprite".to_string()));

    assert!(result.is_success(), "Skipped counts as success");
    assert_eq!(result.skipped_count(), 1, "Should have 1 skipped");
}

// ============================================================================
// Target Result Tests
// ============================================================================

/// @demo cli/build#target_success
/// @title Successful Target Result
/// @description Target result for successful build with outputs.
#[test]
fn test_target_result_success() {
    let outputs = vec![
        PathBuf::from("dist/hero.png"),
        PathBuf::from("dist/hero@2x.png"),
    ];

    let result = TargetResult::success(
        "hero".to_string(),
        outputs.clone(),
        Duration::from_millis(100),
    );

    assert!(result.is_success(), "Should be success");
    assert_eq!(result.target_id, "hero");
    assert_eq!(result.outputs, outputs);
}

/// @demo cli/build#target_failure
/// @title Failed Target Result
/// @description Target result for failed build with error message.
#[test]
fn test_target_result_failure() {
    let result = TargetResult::failed(
        "broken".to_string(),
        "Sprite 'missing' not found in palette".to_string(),
        Duration::from_millis(5),
    );

    assert!(result.status.is_failure(), "Should be failure");
    assert_eq!(result.target_id, "broken");
    match &result.status {
        BuildStatus::Failed(err) => assert!(err.contains("not found")),
        _ => panic!("Expected Failed status"),
    }
}

/// @demo cli/build#target_with_warnings
/// @title Target Result with Warnings
/// @description Target can have warnings even on success.
#[test]
fn test_target_result_with_warnings() {
    let result = TargetResult::success(
        "warning_sprite".to_string(),
        vec![PathBuf::from("dist/warning_sprite.png")],
        Duration::from_millis(50),
    )
    .with_warnings(vec![
        "Color '#FFF' shortened to '#FFFFFF'".to_string(),
        "Unused palette entry '{unused}'".to_string(),
    ]);

    assert!(result.is_success(), "Should still be success");
    assert_eq!(result.warnings.len(), 2, "Should have 2 warnings");
}

// ============================================================================
// Build Result Summary Tests
// ============================================================================

/// @demo cli/build#result_summary
/// @title Build Result Summary
/// @description Generate human-readable build summary.
#[test]
fn test_build_result_summary() {
    let mut result = BuildResult::new();

    result.add_result(TargetResult::success(
        "sprite_1".to_string(),
        vec![],
        Duration::from_millis(50),
    ));
    result.add_result(TargetResult::success(
        "sprite_2".to_string(),
        vec![],
        Duration::from_millis(30),
    ));

    let summary = result.summary();

    // Summary should mention success count
    assert!(
        summary.contains("2") || summary.contains("success"),
        "Summary should mention successes: {}",
        summary
    );
}

/// @demo cli/build#result_outputs
/// @title Collect All Build Outputs
/// @description Gather all output files from build result.
#[test]
fn test_build_result_all_outputs() {
    let mut result = BuildResult::new();

    result.add_result(TargetResult::success(
        "sprite_a".to_string(),
        vec![PathBuf::from("dist/a.png")],
        Duration::from_millis(20),
    ));
    result.add_result(TargetResult::success(
        "sprite_b".to_string(),
        vec![PathBuf::from("dist/b.png"), PathBuf::from("dist/b@2x.png")],
        Duration::from_millis(30),
    ));

    let outputs = result.all_outputs();

    assert_eq!(outputs.len(), 3, "Should have 3 total outputs");
}

// ============================================================================
// Build Plan Tests
// ============================================================================

/// @demo cli/build#plan_empty
/// @title Empty Build Plan
/// @description Build plan with no targets.
#[test]
fn test_build_plan_empty() {
    let plan = BuildPlan::new();

    assert!(plan.is_empty(), "Empty plan should report empty");
    assert_eq!(plan.targets().len(), 0, "Should have 0 targets");
}

/// @demo cli/build#result_duration
/// @title Build Duration Tracking
/// @description Build result tracks total build duration.
#[test]
fn test_build_result_duration() {
    let duration = Duration::from_secs(5);
    let result = BuildResult::new().with_duration(duration);

    assert_eq!(result.total_duration, duration, "Duration should match");
}
