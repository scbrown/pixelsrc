//! Build progress reporting.
//!
//! Provides a flexible progress reporting system for build operations.
//! Supports multiple output formats including console (with colors) and JSON.
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::build::progress::{ProgressReporter, ConsoleProgress, ProgressEvent};
//!
//! let reporter = ConsoleProgress::new();
//! reporter.report(ProgressEvent::BuildStarted { total_targets: 10 });
//! reporter.report(ProgressEvent::TargetStarted { target_id: "sprite:player".to_string() });
//! reporter.report(ProgressEvent::TargetCompleted {
//!     target_id: "sprite:player".to_string(),
//!     status: TargetStatus::Success,
//!     duration_ms: 150,
//! });
//! reporter.report(ProgressEvent::BuildCompleted { success: true, duration_ms: 1500 });
//! ```

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Status of a target in progress events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetStatus {
    /// Target built successfully
    Success,
    /// Target was skipped (up to date)
    Skipped,
    /// Target build failed
    Failed(String),
}

impl std::fmt::Display for TargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetStatus::Success => write!(f, "success"),
            TargetStatus::Skipped => write!(f, "skipped"),
            TargetStatus::Failed(e) => write!(f, "failed: {}", e),
        }
    }
}

/// Events that can be reported during a build.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Build process started
    BuildStarted {
        /// Total number of targets to build
        total_targets: usize,
    },
    /// A target build started
    TargetStarted {
        /// Target identifier
        target_id: String,
    },
    /// A target build completed
    TargetCompleted {
        /// Target identifier
        target_id: String,
        /// Build status
        status: TargetStatus,
        /// Duration in milliseconds
        duration_ms: u64,
    },
    /// Build process completed
    BuildCompleted {
        /// Whether the overall build succeeded
        success: bool,
        /// Total duration in milliseconds
        duration_ms: u64,
        /// Number of successful targets
        succeeded: usize,
        /// Number of skipped targets
        skipped: usize,
        /// Number of failed targets
        failed: usize,
    },
    /// A warning was generated
    Warning {
        /// Target that generated the warning (if applicable)
        target_id: Option<String>,
        /// Warning message
        message: String,
    },
    /// An error occurred
    Error {
        /// Target that generated the error (if applicable)
        target_id: Option<String>,
        /// Error message
        message: String,
    },
}

/// Trait for progress reporters.
pub trait ProgressReporter: Send + Sync {
    /// Report a progress event.
    fn report(&self, event: ProgressEvent);

    /// Check if this reporter wants verbose output.
    fn is_verbose(&self) -> bool {
        false
    }
}

/// A progress reporter that discards all events.
#[derive(Debug, Default)]
pub struct NullProgress;

impl NullProgress {
    /// Create a new null progress reporter.
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for NullProgress {
    fn report(&self, _event: ProgressEvent) {
        // Discard all events
    }
}

/// Console progress reporter with optional colors.
pub struct ConsoleProgress {
    /// Whether to use colors
    use_colors: bool,
    /// Whether to show verbose output
    verbose: bool,
    /// Current target count
    current: AtomicUsize,
    /// Total target count
    total: AtomicUsize,
    /// Output writer (for testing)
    output: Mutex<Box<dyn Write + Send>>,
}

impl std::fmt::Debug for ConsoleProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsoleProgress")
            .field("use_colors", &self.use_colors)
            .field("verbose", &self.verbose)
            .field("current", &self.current)
            .field("total", &self.total)
            .finish()
    }
}

impl ConsoleProgress {
    /// Create a new console progress reporter.
    pub fn new() -> Self {
        Self {
            use_colors: true,
            verbose: false,
            current: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
            output: Mutex::new(Box::new(std::io::stderr())),
        }
    }

    /// Create a console progress reporter that writes to a custom output.
    pub fn with_output<W: Write + Send + 'static>(output: W) -> Self {
        Self {
            use_colors: false, // Disable colors for custom output
            verbose: false,
            current: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
            output: Mutex::new(Box::new(output)),
        }
    }

    /// Set whether to use colors.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// Set verbose mode.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Format a colored string.
    fn color(&self, text: &str, color: &str) -> String {
        if self.use_colors {
            format!("{}{}\x1b[0m", color, text)
        } else {
            text.to_string()
        }
    }

    /// Green color code.
    fn green(&self, text: &str) -> String {
        self.color(text, "\x1b[32m")
    }

    /// Yellow color code.
    fn yellow(&self, text: &str) -> String {
        self.color(text, "\x1b[33m")
    }

    /// Red color code.
    fn red(&self, text: &str) -> String {
        self.color(text, "\x1b[31m")
    }

    /// Cyan color code.
    fn cyan(&self, text: &str) -> String {
        self.color(text, "\x1b[36m")
    }

    /// Bold text.
    fn bold(&self, text: &str) -> String {
        self.color(text, "\x1b[1m")
    }

    /// Write a line to output.
    fn writeln(&self, line: &str) {
        if let Ok(mut output) = self.output.lock() {
            let _ = writeln!(output, "{}", line);
        }
    }
}

impl Default for ConsoleProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressReporter for ConsoleProgress {
    fn report(&self, event: ProgressEvent) {
        match event {
            ProgressEvent::BuildStarted { total_targets } => {
                self.total.store(total_targets, Ordering::SeqCst);
                self.current.store(0, Ordering::SeqCst);
                if total_targets > 0 {
                    self.writeln(&format!(
                        "{} Building {} target{}...",
                        self.cyan("[build]"),
                        total_targets,
                        if total_targets == 1 { "" } else { "s" }
                    ));
                }
            }
            ProgressEvent::TargetStarted { target_id } => {
                if self.verbose {
                    let current = self.current.load(Ordering::SeqCst) + 1;
                    let total = self.total.load(Ordering::SeqCst);
                    self.writeln(&format!(
                        "{} [{}/{}] Building {}...",
                        self.cyan("[build]"),
                        current,
                        total,
                        target_id
                    ));
                }
            }
            ProgressEvent::TargetCompleted { target_id, status, duration_ms } => {
                self.current.fetch_add(1, Ordering::SeqCst);
                let current = self.current.load(Ordering::SeqCst);
                let total = self.total.load(Ordering::SeqCst);

                let status_str = match &status {
                    TargetStatus::Success => self.green("ok"),
                    TargetStatus::Skipped => self.yellow("skipped"),
                    TargetStatus::Failed(_) => self.red("FAILED"),
                };

                let duration_str = format_duration(duration_ms);

                self.writeln(&format!(
                    "{} [{}/{}] {} {} ({})",
                    self.cyan("[build]"),
                    current,
                    total,
                    status_str,
                    target_id,
                    duration_str
                ));

                if let TargetStatus::Failed(err) = status {
                    self.writeln(&format!("        {}", self.red(&err)));
                }
            }
            ProgressEvent::BuildCompleted { success, duration_ms, succeeded, skipped, failed } => {
                let duration_str = format_duration(duration_ms);
                let total = succeeded + skipped + failed;

                if success {
                    self.writeln(&format!(
                        "\n{} {} {} built, {} skipped in {}",
                        self.green("[done]"),
                        self.bold(&format!("{}", total)),
                        if total == 1 { "target" } else { "targets" },
                        skipped,
                        duration_str
                    ));
                } else {
                    self.writeln(&format!(
                        "\n{} Build failed: {} succeeded, {} skipped, {} {} in {}",
                        self.red("[error]"),
                        succeeded,
                        skipped,
                        failed,
                        if failed == 1 { "failure" } else { "failures" },
                        duration_str
                    ));
                }
            }
            ProgressEvent::Warning { target_id, message } => {
                let prefix = match target_id {
                    Some(id) => format!("{}: ", id),
                    None => String::new(),
                };
                self.writeln(&format!("{} {}{}", self.yellow("[warn]"), prefix, message));
            }
            ProgressEvent::Error { target_id, message } => {
                let prefix = match target_id {
                    Some(id) => format!("{}: ", id),
                    None => String::new(),
                };
                self.writeln(&format!("{} {}{}", self.red("[error]"), prefix, message));
            }
        }
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }
}

/// JSON progress reporter for machine-readable output.
pub struct JsonProgress {
    /// Output writer
    output: Mutex<Box<dyn Write + Send>>,
}

impl std::fmt::Debug for JsonProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonProgress").finish()
    }
}

impl JsonProgress {
    /// Create a new JSON progress reporter writing to stderr.
    pub fn new() -> Self {
        Self { output: Mutex::new(Box::new(std::io::stderr())) }
    }

    /// Create a JSON progress reporter that writes to a custom output.
    pub fn with_output<W: Write + Send + 'static>(output: W) -> Self {
        Self { output: Mutex::new(Box::new(output)) }
    }

    /// Write a JSON line to output.
    fn write_json(&self, json: &str) {
        if let Ok(mut output) = self.output.lock() {
            let _ = writeln!(output, "{}", json);
        }
    }
}

impl Default for JsonProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressReporter for JsonProgress {
    fn report(&self, event: ProgressEvent) {
        let json = match event {
            ProgressEvent::BuildStarted { total_targets } => {
                format!(r#"{{"event":"build_started","total_targets":{}}}"#, total_targets)
            }
            ProgressEvent::TargetStarted { target_id } => {
                format!(r#"{{"event":"target_started","target_id":"{}"}}"#, escape_json(&target_id))
            }
            ProgressEvent::TargetCompleted { target_id, status, duration_ms } => {
                let status_str = match &status {
                    TargetStatus::Success => "success",
                    TargetStatus::Skipped => "skipped",
                    TargetStatus::Failed(_) => "failed",
                };
                let error = match &status {
                    TargetStatus::Failed(e) => format!(r#","error":"{}""#, escape_json(e)),
                    _ => String::new(),
                };
                format!(
                    r#"{{"event":"target_completed","target_id":"{}","status":"{}","duration_ms":{}{}}}"#,
                    escape_json(&target_id),
                    status_str,
                    duration_ms,
                    error
                )
            }
            ProgressEvent::BuildCompleted { success, duration_ms, succeeded, skipped, failed } => {
                format!(
                    r#"{{"event":"build_completed","success":{},"duration_ms":{},"succeeded":{},"skipped":{},"failed":{}}}"#,
                    success, duration_ms, succeeded, skipped, failed
                )
            }
            ProgressEvent::Warning { target_id, message } => {
                let target = match target_id {
                    Some(id) => format!(r#","target_id":"{}""#, escape_json(&id)),
                    None => String::new(),
                };
                format!(r#"{{"event":"warning","message":"{}"{}}}"#, escape_json(&message), target)
            }
            ProgressEvent::Error { target_id, message } => {
                let target = match target_id {
                    Some(id) => format!(r#","target_id":"{}""#, escape_json(&id)),
                    None => String::new(),
                };
                format!(r#"{{"event":"error","message":"{}"{}}}"#, escape_json(&message), target)
            }
        };
        self.write_json(&json);
    }
}

/// Progress tracker for aggregating build statistics.
#[derive(Debug, Default)]
pub struct ProgressTracker {
    /// Start time of the build
    start_time: Option<Instant>,
    /// Total number of targets
    total: usize,
    /// Number of completed targets
    completed: usize,
    /// Number of successful targets
    succeeded: usize,
    /// Number of skipped targets
    skipped: usize,
    /// Number of failed targets
    failed: usize,
    /// Currently building targets
    in_progress: Vec<String>,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking a build.
    pub fn start(&mut self, total_targets: usize) {
        self.start_time = Some(Instant::now());
        self.total = total_targets;
        self.completed = 0;
        self.succeeded = 0;
        self.skipped = 0;
        self.failed = 0;
        self.in_progress.clear();
    }

    /// Mark a target as started.
    pub fn target_started(&mut self, target_id: &str) {
        self.in_progress.push(target_id.to_string());
    }

    /// Mark a target as completed.
    pub fn target_completed(&mut self, target_id: &str, status: &TargetStatus) {
        self.in_progress.retain(|id| id != target_id);
        self.completed += 1;
        match status {
            TargetStatus::Success => self.succeeded += 1,
            TargetStatus::Skipped => self.skipped += 1,
            TargetStatus::Failed(_) => self.failed += 1,
        }
    }

    /// Get the elapsed time since the build started.
    pub fn elapsed(&self) -> Duration {
        self.start_time.map(|t| t.elapsed()).unwrap_or(Duration::ZERO)
    }

    /// Get the elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Get the completion percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if the build is complete.
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total
    }

    /// Check if the build was successful.
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Get the number of succeeded targets.
    pub fn succeeded(&self) -> usize {
        self.succeeded
    }

    /// Get the number of skipped targets.
    pub fn skipped(&self) -> usize {
        self.skipped
    }

    /// Get the number of failed targets.
    pub fn failed(&self) -> usize {
        self.failed
    }

    /// Get the targets currently in progress.
    pub fn in_progress(&self) -> &[String] {
        &self.in_progress
    }

    /// Generate a BuildCompleted event from current state.
    pub fn build_completed_event(&self) -> ProgressEvent {
        ProgressEvent::BuildCompleted {
            success: self.is_success(),
            duration_ms: self.elapsed_ms(),
            succeeded: self.succeeded,
            skipped: self.skipped,
            failed: self.failed,
        }
    }
}

/// Format a duration in milliseconds to a human-readable string.
fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        format!("{}m {}s", minutes, seconds)
    }
}

/// Escape a string for JSON output.
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_target_status_display() {
        assert_eq!(TargetStatus::Success.to_string(), "success");
        assert_eq!(TargetStatus::Skipped.to_string(), "skipped");
        assert_eq!(TargetStatus::Failed("error".to_string()).to_string(), "failed: error");
    }

    #[test]
    fn test_null_progress() {
        let reporter = NullProgress::new();
        // Should not panic
        reporter.report(ProgressEvent::BuildStarted { total_targets: 10 });
        reporter.report(ProgressEvent::TargetStarted { target_id: "test".to_string() });
        assert!(!reporter.is_verbose());
    }

    #[test]
    fn test_console_progress_build_started() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = ConsoleProgress::with_output(TestWriter(output_clone)).with_colors(false);
        reporter.report(ProgressEvent::BuildStarted { total_targets: 5 });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("Building 5 targets"));
    }

    #[test]
    fn test_console_progress_target_completed_success() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = ConsoleProgress::with_output(TestWriter(output_clone)).with_colors(false);
        reporter.report(ProgressEvent::BuildStarted { total_targets: 1 });
        reporter.report(ProgressEvent::TargetCompleted {
            target_id: "sprite:test".to_string(),
            status: TargetStatus::Success,
            duration_ms: 150,
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("ok"));
        assert!(text.contains("sprite:test"));
        assert!(text.contains("150ms"));
    }

    #[test]
    fn test_console_progress_target_completed_failed() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = ConsoleProgress::with_output(TestWriter(output_clone)).with_colors(false);
        reporter.report(ProgressEvent::BuildStarted { total_targets: 1 });
        reporter.report(ProgressEvent::TargetCompleted {
            target_id: "sprite:test".to_string(),
            status: TargetStatus::Failed("file not found".to_string()),
            duration_ms: 50,
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("FAILED"));
        assert!(text.contains("file not found"));
    }

    #[test]
    fn test_console_progress_build_completed_success() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = ConsoleProgress::with_output(TestWriter(output_clone)).with_colors(false);
        reporter.report(ProgressEvent::BuildCompleted {
            success: true,
            duration_ms: 1500,
            succeeded: 5,
            skipped: 2,
            failed: 0,
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("done"));
        assert!(text.contains("7"));
        assert!(text.contains("2 skipped"));
    }

    #[test]
    fn test_console_progress_build_completed_failed() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = ConsoleProgress::with_output(TestWriter(output_clone)).with_colors(false);
        reporter.report(ProgressEvent::BuildCompleted {
            success: false,
            duration_ms: 500,
            succeeded: 3,
            skipped: 1,
            failed: 2,
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("error"));
        assert!(text.contains("2 failures"));
    }

    #[test]
    fn test_console_progress_warning() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = ConsoleProgress::with_output(TestWriter(output_clone)).with_colors(false);
        reporter.report(ProgressEvent::Warning {
            target_id: Some("sprite:test".to_string()),
            message: "deprecated format".to_string(),
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("warn"));
        assert!(text.contains("sprite:test"));
        assert!(text.contains("deprecated format"));
    }

    #[test]
    fn test_json_progress_build_started() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = JsonProgress::with_output(TestWriter(output_clone));
        reporter.report(ProgressEvent::BuildStarted { total_targets: 10 });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains(r#""event":"build_started""#));
        assert!(text.contains(r#""total_targets":10"#));
    }

    #[test]
    fn test_json_progress_target_completed() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = JsonProgress::with_output(TestWriter(output_clone));
        reporter.report(ProgressEvent::TargetCompleted {
            target_id: "sprite:test".to_string(),
            status: TargetStatus::Success,
            duration_ms: 100,
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains(r#""event":"target_completed""#));
        assert!(text.contains(r#""target_id":"sprite:test""#));
        assert!(text.contains(r#""status":"success""#));
        assert!(text.contains(r#""duration_ms":100"#));
    }

    #[test]
    fn test_json_progress_target_failed() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = Arc::clone(&output);

        let reporter = JsonProgress::with_output(TestWriter(output_clone));
        reporter.report(ProgressEvent::TargetCompleted {
            target_id: "sprite:test".to_string(),
            status: TargetStatus::Failed("not found".to_string()),
            duration_ms: 50,
        });

        let output = output.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains(r#""status":"failed""#));
        assert!(text.contains(r#""error":"not found""#));
    }

    #[test]
    fn test_progress_tracker_new() {
        let tracker = ProgressTracker::new();
        assert_eq!(tracker.total, 0);
        assert_eq!(tracker.completed, 0);
        assert!(tracker.is_complete()); // 0/0 is complete
    }

    #[test]
    fn test_progress_tracker_lifecycle() {
        let mut tracker = ProgressTracker::new();
        tracker.start(3);

        assert_eq!(tracker.total, 3);
        assert!(!tracker.is_complete());
        assert_eq!(tracker.percentage(), 0.0);

        tracker.target_started("a");
        assert_eq!(tracker.in_progress().len(), 1);

        tracker.target_completed("a", &TargetStatus::Success);
        assert_eq!(tracker.in_progress().len(), 0);
        assert_eq!(tracker.succeeded(), 1);
        assert!((tracker.percentage() - 33.333).abs() < 0.1);

        tracker.target_started("b");
        tracker.target_completed("b", &TargetStatus::Skipped);
        assert_eq!(tracker.skipped(), 1);

        tracker.target_started("c");
        tracker.target_completed("c", &TargetStatus::Failed("error".to_string()));
        assert_eq!(tracker.failed(), 1);

        assert!(tracker.is_complete());
        assert!(!tracker.is_success());
    }

    #[test]
    fn test_progress_tracker_build_completed_event() {
        let mut tracker = ProgressTracker::new();
        tracker.start(2);
        tracker.target_completed("a", &TargetStatus::Success);
        tracker.target_completed("b", &TargetStatus::Success);

        let event = tracker.build_completed_event();
        match event {
            ProgressEvent::BuildCompleted { success, succeeded, skipped, failed, .. } => {
                assert!(success);
                assert_eq!(succeeded, 2);
                assert_eq!(skipped, 0);
                assert_eq!(failed, 0);
            }
            _ => panic!("Expected BuildCompleted event"),
        }
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0ms");
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(999), "999ms");
        assert_eq!(format_duration(1000), "1.0s");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(60000), "1m 0s");
        assert_eq!(format_duration(90000), "1m 30s");
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello\"world"), "hello\\\"world");
        assert_eq!(escape_json("hello\\world"), "hello\\\\world");
        assert_eq!(escape_json("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json("hello\tworld"), "hello\\tworld");
    }

    // Helper for testing output
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
}
