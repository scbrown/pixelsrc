//! Build result types.
//!
//! Contains types for representing the outcome of build operations.

use std::path::PathBuf;
use std::time::Duration;

/// Status of a single build target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    /// Build succeeded
    Success,
    /// Build skipped (already up to date)
    Skipped,
    /// Build failed with error
    Failed(String),
}

impl BuildStatus {
    /// Check if the status indicates success.
    pub fn is_success(&self) -> bool {
        matches!(self, BuildStatus::Success | BuildStatus::Skipped)
    }

    /// Check if the status indicates failure.
    pub fn is_failure(&self) -> bool {
        matches!(self, BuildStatus::Failed(_))
    }
}

impl std::fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildStatus::Success => write!(f, "success"),
            BuildStatus::Skipped => write!(f, "skipped"),
            BuildStatus::Failed(err) => write!(f, "failed: {}", err),
        }
    }
}

/// Result of building a single target.
#[derive(Debug, Clone)]
pub struct TargetResult {
    /// Target ID that was built
    pub target_id: String,
    /// Build status
    pub status: BuildStatus,
    /// Output files produced
    pub outputs: Vec<PathBuf>,
    /// Build duration
    pub duration: Duration,
    /// Warning messages (if any)
    pub warnings: Vec<String>,
}

impl TargetResult {
    /// Create a successful result.
    pub fn success(target_id: String, outputs: Vec<PathBuf>, duration: Duration) -> Self {
        Self { target_id, status: BuildStatus::Success, outputs, duration, warnings: vec![] }
    }

    /// Create a skipped result.
    pub fn skipped(target_id: String) -> Self {
        Self {
            target_id,
            status: BuildStatus::Skipped,
            outputs: vec![],
            duration: Duration::ZERO,
            warnings: vec![],
        }
    }

    /// Create a failed result.
    pub fn failed(target_id: String, error: String, duration: Duration) -> Self {
        Self {
            target_id,
            status: BuildStatus::Failed(error),
            outputs: vec![],
            duration,
            warnings: vec![],
        }
    }

    /// Add warnings to the result.
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    /// Check if this result is successful.
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

/// Result of a complete build run.
#[derive(Debug, Default)]
pub struct BuildResult {
    /// Results for each target
    pub targets: Vec<TargetResult>,
    /// Total build duration
    pub total_duration: Duration,
}

impl BuildResult {
    /// Create a new empty build result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a target result.
    pub fn add_result(&mut self, result: TargetResult) {
        self.targets.push(result);
    }

    /// Set the total duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.total_duration = duration;
        self
    }

    /// Get the number of successful targets.
    pub fn success_count(&self) -> usize {
        self.targets.iter().filter(|r| matches!(r.status, BuildStatus::Success)).count()
    }

    /// Get the number of skipped targets.
    pub fn skipped_count(&self) -> usize {
        self.targets.iter().filter(|r| matches!(r.status, BuildStatus::Skipped)).count()
    }

    /// Get the number of failed targets.
    pub fn failed_count(&self) -> usize {
        self.targets.iter().filter(|r| r.status.is_failure()).count()
    }

    /// Check if the overall build succeeded (no failures).
    pub fn is_success(&self) -> bool {
        self.failed_count() == 0
    }

    /// Get all outputs produced.
    pub fn all_outputs(&self) -> Vec<&PathBuf> {
        self.targets.iter().flat_map(|r| r.outputs.iter()).collect()
    }

    /// Get all warnings.
    pub fn all_warnings(&self) -> Vec<&String> {
        self.targets.iter().flat_map(|r| r.warnings.iter()).collect()
    }

    /// Get failed target results.
    pub fn failures(&self) -> Vec<&TargetResult> {
        self.targets.iter().filter(|r| r.status.is_failure()).collect()
    }

    /// Format a summary of the build result.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        let success = self.success_count();
        let skipped = self.skipped_count();
        let failed = self.failed_count();
        let total = self.targets.len();

        if failed > 0 {
            lines.push(format!(
                "Build failed: {} succeeded, {} skipped, {} failed ({} total)",
                success, skipped, failed, total
            ));
            for target in self.failures() {
                lines.push(format!("  - {}: {}", target.target_id, target.status));
            }
        } else {
            lines.push(format!(
                "Build succeeded: {} built, {} skipped ({} total) in {:?}",
                success, skipped, total, self.total_duration
            ));
        }

        let warnings = self.all_warnings();
        if !warnings.is_empty() {
            lines.push(format!("Warnings ({}): ", warnings.len()));
            for warning in warnings.iter().take(5) {
                lines.push(format!("  - {}", warning));
            }
            if warnings.len() > 5 {
                lines.push(format!("  ... and {} more", warnings.len() - 5));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_status_display() {
        assert_eq!(BuildStatus::Success.to_string(), "success");
        assert_eq!(BuildStatus::Skipped.to_string(), "skipped");
        assert_eq!(BuildStatus::Failed("error".to_string()).to_string(), "failed: error");
    }

    #[test]
    fn test_build_status_is_success() {
        assert!(BuildStatus::Success.is_success());
        assert!(BuildStatus::Skipped.is_success());
        assert!(!BuildStatus::Failed("error".to_string()).is_success());
    }

    #[test]
    fn test_target_result_success() {
        let result = TargetResult::success(
            "atlas:main".to_string(),
            vec![PathBuf::from("main.png")],
            Duration::from_millis(100),
        );

        assert!(result.is_success());
        assert_eq!(result.outputs.len(), 1);
    }

    #[test]
    fn test_target_result_failed() {
        let result = TargetResult::failed(
            "atlas:main".to_string(),
            "File not found".to_string(),
            Duration::from_millis(50),
        );

        assert!(!result.is_success());
        assert!(result.outputs.is_empty());
    }

    #[test]
    fn test_target_result_with_warnings() {
        let result =
            TargetResult::success("atlas:main".to_string(), vec![], Duration::from_millis(100))
                .with_warnings(vec!["Warning 1".to_string(), "Warning 2".to_string()]);

        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn test_build_result_counts() {
        let mut result = BuildResult::new();
        result.add_result(TargetResult::success("a".to_string(), vec![], Duration::ZERO));
        result.add_result(TargetResult::skipped("b".to_string()));
        result.add_result(TargetResult::failed(
            "c".to_string(),
            "error".to_string(),
            Duration::ZERO,
        ));

        assert_eq!(result.success_count(), 1);
        assert_eq!(result.skipped_count(), 1);
        assert_eq!(result.failed_count(), 1);
        assert!(!result.is_success());
    }

    #[test]
    fn test_build_result_is_success() {
        let mut result = BuildResult::new();
        result.add_result(TargetResult::success("a".to_string(), vec![], Duration::ZERO));
        result.add_result(TargetResult::skipped("b".to_string()));

        assert!(result.is_success());
    }

    #[test]
    fn test_build_result_all_outputs() {
        let mut result = BuildResult::new();
        result.add_result(TargetResult::success(
            "a".to_string(),
            vec![PathBuf::from("a.png")],
            Duration::ZERO,
        ));
        result.add_result(TargetResult::success(
            "b".to_string(),
            vec![PathBuf::from("b.png"), PathBuf::from("b.json")],
            Duration::ZERO,
        ));

        let outputs = result.all_outputs();
        assert_eq!(outputs.len(), 3);
    }

    #[test]
    fn test_build_result_summary() {
        let mut result = BuildResult::new();
        result.add_result(TargetResult::success(
            "atlas:main".to_string(),
            vec![],
            Duration::from_millis(100),
        ));

        let summary = result.with_duration(Duration::from_millis(100)).summary();
        assert!(summary.contains("Build succeeded"));
        assert!(summary.contains("1 built"));
    }
}
