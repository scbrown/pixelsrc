//! Watch mode for automatic rebuilds on file changes
//!
//! Provides file system watching with debouncing for the `pxl build --watch` command.

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::config::schema::WatchConfig;

/// Error during watch mode
#[derive(Debug)]
pub enum WatchError {
    /// Failed to initialize file watcher
    WatcherInit(notify::Error),
    /// Failed to add watch path
    WatchPath(notify::Error),
    /// Channel receive error
    ChannelError(String),
    /// Build failed (non-fatal, continues watching)
    BuildFailed(String),
    /// Source directory not found
    SourceNotFound(PathBuf),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchError::WatcherInit(e) => write!(f, "Failed to initialize file watcher: {}", e),
            WatchError::WatchPath(e) => write!(f, "Failed to watch path: {}", e),
            WatchError::ChannelError(msg) => write!(f, "Watch channel error: {}", msg),
            WatchError::BuildFailed(msg) => write!(f, "Build failed: {}", msg),
            WatchError::SourceNotFound(path) => {
                write!(f, "Source directory not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for WatchError {}

/// A detailed build error with file location information
#[derive(Debug, Clone)]
pub struct BuildError {
    /// Path to the file containing the error
    pub file: PathBuf,
    /// Line number (1-indexed, None if unknown)
    pub line: Option<usize>,
    /// Column number (1-indexed, None if unknown)
    pub column: Option<usize>,
    /// Error message
    pub message: String,
}

impl BuildError {
    /// Create a new build error with file and message
    pub fn new(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self { file: file.into(), line: None, column: None, message: message.into() }
    }

    /// Create a build error with line information
    pub fn with_line(file: impl Into<PathBuf>, line: usize, message: impl Into<String>) -> Self {
        Self { file: file.into(), line: Some(line), column: None, message: message.into() }
    }

    /// Create a build error with full location information
    pub fn with_location(
        file: impl Into<PathBuf>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self { file: file.into(), line: Some(line), column: Some(column), message: message.into() }
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in {}", self.file.display())?;
        if let Some(line) = self.line {
            write!(f, ":{}", line)?;
            if let Some(col) = self.column {
                write!(f, ":{}", col)?;
            }
        }
        write!(f, ": {}", self.message)
    }
}

/// Tracks files with errors across build iterations for recovery detection
#[derive(Debug, Default)]
pub struct ErrorTracker {
    /// Files that had errors in the previous build
    files_with_errors: HashSet<PathBuf>,
}

impl ErrorTracker {
    /// Create a new error tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Update tracker with new build result, returns list of fixed files
    pub fn update(&mut self, result: &BuildResult) -> Vec<PathBuf> {
        let current_error_files: HashSet<PathBuf> =
            result.build_errors.iter().map(|e| e.file.clone()).collect();

        // Find files that had errors before but don't now
        let fixed: Vec<PathBuf> =
            self.files_with_errors.difference(&current_error_files).cloned().collect();

        // Update the tracked error files
        self.files_with_errors = current_error_files;

        fixed
    }

    /// Check if there are any tracked errors
    pub fn has_errors(&self) -> bool {
        !self.files_with_errors.is_empty()
    }

    /// Get the number of files with errors
    pub fn error_count(&self) -> usize {
        self.files_with_errors.len()
    }
}

/// Options for watch mode
#[derive(Debug, Clone)]
pub struct WatchOptions {
    /// Source directory to watch
    pub src_dir: PathBuf,
    /// Output directory for builds
    pub out_dir: PathBuf,
    /// Watch configuration (debounce, clear screen)
    pub config: WatchConfig,
    /// Verbose output
    pub verbose: bool,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self {
            src_dir: PathBuf::from("src/pxl"),
            out_dir: PathBuf::from("build"),
            config: WatchConfig::default(),
            verbose: false,
        }
    }
}

/// Result of a single build attempt
#[derive(Debug)]
pub struct BuildResult {
    /// Number of files processed
    pub files_processed: usize,
    /// Number of sprites rendered
    pub sprites_rendered: usize,
    /// Simple error messages (legacy field, prefer build_errors)
    pub errors: Vec<String>,
    /// Detailed build errors with file/line information
    pub build_errors: Vec<BuildError>,
    /// Number of warnings
    pub warnings: Vec<String>,
    /// Build duration
    pub duration: Duration,
}

impl BuildResult {
    /// Create a new empty build result
    pub fn new() -> Self {
        Self {
            files_processed: 0,
            sprites_rendered: 0,
            errors: vec![],
            build_errors: vec![],
            warnings: vec![],
            duration: Duration::ZERO,
        }
    }

    /// Check if build succeeded (no errors)
    pub fn success(&self) -> bool {
        self.errors.is_empty() && self.build_errors.is_empty()
    }

    /// Add a detailed build error
    pub fn add_error(&mut self, error: BuildError) {
        self.build_errors.push(error);
    }

    /// Total number of errors (both legacy and detailed)
    pub fn error_count(&self) -> usize {
        self.errors.len() + self.build_errors.len()
    }
}

impl Default for BuildResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Clear the terminal screen
fn clear_screen() {
    // ANSI escape code to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
}

/// Format duration for display
fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1000 {
        format!("{}ms", millis)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

/// Get current timestamp for logging
fn timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs() % 86400; // seconds since midnight
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Perform a single build iteration.
///
/// This function discovers .pxl files, parses them, and renders sprites.
/// In watch mode, this is called on startup and after each file change.
pub fn do_build<F>(options: &WatchOptions, build_fn: F) -> BuildResult
where
    F: FnOnce(&Path, &Path) -> BuildResult,
{
    let start = Instant::now();
    let mut result = build_fn(&options.src_dir, &options.out_dir);
    result.duration = start.elapsed();
    result
}

/// Simple build function that discovers and counts files.
///
/// This is a placeholder that will be replaced by the full build pipeline
/// when BST-3 is complete. For now, it demonstrates the watch infrastructure.
pub fn simple_build(src_dir: &Path, _out_dir: &Path) -> BuildResult {
    use glob::glob;

    let mut result = BuildResult::new();

    // Find all .pxl and .jsonl files
    let pattern = format!("{}/**/*.pxl", src_dir.display());
    if let Ok(entries) = glob(&pattern) {
        for entry in entries.flatten() {
            result.files_processed += 1;
            if let Ok(content) = std::fs::read_to_string(&entry) {
                // Count sprites (very basic)
                result.sprites_rendered += content.matches("\"type\": \"sprite\"").count();
                result.sprites_rendered += content.matches("\"type\":\"sprite\"").count();
            }
        }
    }

    let pattern_jsonl = format!("{}/**/*.jsonl", src_dir.display());
    if let Ok(entries) = glob(&pattern_jsonl) {
        for entry in entries.flatten() {
            result.files_processed += 1;
            if let Ok(content) = std::fs::read_to_string(&entry) {
                result.sprites_rendered += content.matches("\"type\": \"sprite\"").count();
                result.sprites_rendered += content.matches("\"type\":\"sprite\"").count();
            }
        }
    }

    result
}

/// Watch for file changes and rebuild automatically.
///
/// This function blocks and runs until interrupted (Ctrl+C).
///
/// # Arguments
/// * `options` - Watch mode configuration
///
/// # Returns
/// * `Ok(())` if watch mode exits cleanly (shouldn't happen normally)
/// * `Err(WatchError)` if watch setup fails
///
/// # Example
/// ```ignore
/// let options = WatchOptions {
///     src_dir: PathBuf::from("src/pxl"),
///     out_dir: PathBuf::from("build"),
///     config: WatchConfig::default(),
///     verbose: false,
/// };
/// watch_and_rebuild(options)?;
/// ```
pub fn watch_and_rebuild(options: WatchOptions) -> Result<(), WatchError> {
    // Verify source directory exists
    if !options.src_dir.exists() {
        return Err(WatchError::SourceNotFound(options.src_dir.clone()));
    }

    // Create output directory if needed
    if !options.out_dir.exists() {
        std::fs::create_dir_all(&options.out_dir).ok();
    }

    // Create channel for debounced events
    let (tx, rx) = channel();

    // Create debounced watcher
    let debounce_duration = Duration::from_millis(options.config.debounce_ms as u64);
    let mut debouncer = new_debouncer(debounce_duration, tx).map_err(WatchError::WatcherInit)?;

    // Start watching the source directory
    debouncer
        .watcher()
        .watch(&options.src_dir, RecursiveMode::Recursive)
        .map_err(WatchError::WatchPath)?;

    // Error tracker for detecting fixed files
    let mut error_tracker = ErrorTracker::new();

    // Initial build
    if options.config.clear_screen {
        clear_screen();
    }
    println!("[{}] Building...", timestamp());
    let result = do_build(&options, simple_build);
    print_build_result(&result, &[]);
    error_tracker.update(&result);
    println!("[{}] Watching {} for changes...", timestamp(), options.src_dir.display());

    // Watch loop
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Filter for relevant file changes
                let relevant_changes: Vec<_> = events
                    .iter()
                    .filter(|e| {
                        matches!(e.kind, DebouncedEventKind::Any) && is_relevant_file(&e.path)
                    })
                    .collect();

                if !relevant_changes.is_empty() {
                    // Log changed files
                    for event in &relevant_changes {
                        if let Some(name) = event.path.file_name() {
                            println!("[{}] Changed: {}", timestamp(), name.to_string_lossy());
                        }
                    }

                    // Clear screen if configured
                    if options.config.clear_screen {
                        clear_screen();
                    }

                    // Rebuild
                    println!("[{}] Building...", timestamp());
                    let result = do_build(&options, simple_build);

                    // Track fixed files before updating error tracker
                    let fixed_files = error_tracker.update(&result);
                    print_build_result(&result, &fixed_files);

                    println!(
                        "[{}] Watching {} for changes...",
                        timestamp(),
                        options.src_dir.display()
                    );
                }
            }
            Ok(Err(error)) => {
                // Watch error (non-fatal) - log but continue watching
                eprintln!("[{}] Watch error: {:?}", timestamp(), error);
                eprintln!("[{}] Continuing to watch...", timestamp());
            }
            Err(e) => {
                return Err(WatchError::ChannelError(e.to_string()));
            }
        }
    }
}

/// Check if a file is relevant for rebuilding
fn is_relevant_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "pxl" | "jsonl" | "json")
    } else {
        false
    }
}

/// Print build result to console with fixed file notifications
fn print_build_result(result: &BuildResult, fixed_files: &[PathBuf]) {
    // Report fixed files first (before showing new errors)
    for fixed in fixed_files {
        if let Some(name) = fixed.file_name() {
            println!("[{}] Fixed: {}", timestamp(), name.to_string_lossy());
        }
    }

    if result.success() {
        println!(
            "[{}] Build complete ({}) - Files: {} | Sprites: {}",
            timestamp(),
            format_duration(result.duration),
            result.files_processed,
            result.sprites_rendered
        );
    } else {
        let error_count = result.error_count();
        println!(
            "[{}] Build failed ({}) - {} error{}",
            timestamp(),
            format_duration(result.duration),
            error_count,
            if error_count == 1 { "" } else { "s" }
        );

        // Print detailed build errors with file/line info
        for error in &result.build_errors {
            if let Some(name) = error.file.file_name() {
                eprint!("[{}] Error in {}:", timestamp(), name.to_string_lossy());
                if let Some(line) = error.line {
                    eprint!("\n          Line {}: ", line);
                } else {
                    eprint!(" ");
                }
                eprintln!("{}", error.message);
            } else {
                eprintln!("[{}] Error: {}", timestamp(), error);
            }
        }

        // Print legacy simple errors
        for error in &result.errors {
            eprintln!("[{}] Error: {}", timestamp(), error);
        }
    }

    // Print warnings
    for warning in &result.warnings {
        eprintln!("[{}] Warning: {}", timestamp(), warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_watch_options_default() {
        let options = WatchOptions::default();
        assert_eq!(options.src_dir, PathBuf::from("src/pxl"));
        assert_eq!(options.out_dir, PathBuf::from("build"));
        assert_eq!(options.config.debounce_ms, 100);
        assert!(options.config.clear_screen);
    }

    #[test]
    fn test_build_result_new() {
        let result = BuildResult::new();
        assert_eq!(result.files_processed, 0);
        assert_eq!(result.sprites_rendered, 0);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.success());
    }

    #[test]
    fn test_build_result_with_errors() {
        let mut result = BuildResult::new();
        result.errors.push("Test error".to_string());
        assert!(!result.success());
    }

    #[test]
    fn test_is_relevant_file() {
        assert!(is_relevant_file(Path::new("sprite.pxl")));
        assert!(is_relevant_file(Path::new("sprites.jsonl")));
        assert!(is_relevant_file(Path::new("data.json")));
        assert!(!is_relevant_file(Path::new("readme.md")));
        assert!(!is_relevant_file(Path::new("image.png")));
        assert!(!is_relevant_file(Path::new("noextension")));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(50)), "50ms");
        assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
        assert_eq!(format_duration(Duration::from_millis(1000)), "1.00s");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
    }

    #[test]
    fn test_simple_build_empty_dir() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        let result = simple_build(&src, temp.path());
        assert_eq!(result.files_processed, 0);
        assert_eq!(result.sprites_rendered, 0);
        assert!(result.success());
    }

    #[test]
    fn test_simple_build_with_files() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        // Create a test .pxl file with sprites
        let content = r#"{"type": "sprite", "name": "test1"}
{"type": "sprite", "name": "test2"}"#;
        std::fs::write(src.join("test.jsonl"), content).unwrap();

        let result = simple_build(&src, temp.path());
        assert_eq!(result.files_processed, 1);
        assert_eq!(result.sprites_rendered, 2);
        assert!(result.success());
    }

    #[test]
    fn test_watch_error_source_not_found() {
        let options =
            WatchOptions { src_dir: PathBuf::from("/nonexistent/path"), ..Default::default() };

        let result = watch_and_rebuild(options);
        assert!(matches!(result, Err(WatchError::SourceNotFound(_))));
    }

    #[test]
    fn test_do_build_with_custom_function() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        let options =
            WatchOptions { src_dir: src, out_dir: temp.path().to_path_buf(), ..Default::default() };

        let result = do_build(&options, |_src, _out| {
            let mut r = BuildResult::new();
            r.files_processed = 5;
            r.sprites_rendered = 10;
            r
        });

        assert_eq!(result.files_processed, 5);
        assert_eq!(result.sprites_rendered, 10);
        assert!(result.duration >= Duration::ZERO);
    }

    // Error recovery tests

    #[test]
    fn test_build_error_new() {
        let error = BuildError::new("test.pxl", "Invalid syntax");
        assert_eq!(error.file, PathBuf::from("test.pxl"));
        assert_eq!(error.line, None);
        assert_eq!(error.column, None);
        assert_eq!(error.message, "Invalid syntax");
    }

    #[test]
    fn test_build_error_with_line() {
        let error = BuildError::with_line("test.pxl", 5, "Invalid color");
        assert_eq!(error.file, PathBuf::from("test.pxl"));
        assert_eq!(error.line, Some(5));
        assert_eq!(error.column, None);
        assert_eq!(error.message, "Invalid color");
    }

    #[test]
    fn test_build_error_with_location() {
        let error = BuildError::with_location("test.pxl", 5, 10, "Unexpected token");
        assert_eq!(error.file, PathBuf::from("test.pxl"));
        assert_eq!(error.line, Some(5));
        assert_eq!(error.column, Some(10));
        assert_eq!(error.message, "Unexpected token");
    }

    #[test]
    fn test_build_error_display() {
        let error = BuildError::with_line("sprites/broken.pxl", 5, "Invalid color format \"#GGG\"");
        let display = format!("{}", error);
        assert!(display.contains("sprites/broken.pxl"));
        assert!(display.contains("5"));
        assert!(display.contains("Invalid color format"));
    }

    #[test]
    fn test_error_tracker_new() {
        let tracker = ErrorTracker::new();
        assert!(!tracker.has_errors());
        assert_eq!(tracker.error_count(), 0);
    }

    #[test]
    fn test_error_tracker_tracks_errors() {
        let mut tracker = ErrorTracker::new();

        // First build has errors
        let mut result = BuildResult::new();
        result.add_error(BuildError::new("file1.pxl", "Error 1"));
        result.add_error(BuildError::new("file2.pxl", "Error 2"));

        let fixed = tracker.update(&result);
        assert!(fixed.is_empty()); // No fixed files on first build
        assert!(tracker.has_errors());
        assert_eq!(tracker.error_count(), 2);
    }

    #[test]
    fn test_error_tracker_detects_fixed_files() {
        let mut tracker = ErrorTracker::new();

        // First build has errors in file1 and file2
        let mut result1 = BuildResult::new();
        result1.add_error(BuildError::new("file1.pxl", "Error 1"));
        result1.add_error(BuildError::new("file2.pxl", "Error 2"));
        tracker.update(&result1);

        // Second build: file1 is fixed, file2 still has error
        let mut result2 = BuildResult::new();
        result2.add_error(BuildError::new("file2.pxl", "Error 2"));
        let fixed = tracker.update(&result2);

        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed[0], PathBuf::from("file1.pxl"));
        assert!(tracker.has_errors());
        assert_eq!(tracker.error_count(), 1);
    }

    #[test]
    fn test_error_tracker_all_fixed() {
        let mut tracker = ErrorTracker::new();

        // First build has errors
        let mut result1 = BuildResult::new();
        result1.add_error(BuildError::new("file1.pxl", "Error 1"));
        tracker.update(&result1);

        // Second build: all fixed
        let result2 = BuildResult::new();
        let fixed = tracker.update(&result2);

        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed[0], PathBuf::from("file1.pxl"));
        assert!(!tracker.has_errors());
        assert_eq!(tracker.error_count(), 0);
    }

    #[test]
    fn test_build_result_with_build_errors() {
        let mut result = BuildResult::new();
        assert!(result.success());
        assert_eq!(result.error_count(), 0);

        result.add_error(BuildError::new("test.pxl", "Error"));
        assert!(!result.success());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_build_result_mixed_errors() {
        let mut result = BuildResult::new();

        // Add both legacy and detailed errors
        result.errors.push("Legacy error".to_string());
        result.add_error(BuildError::new("test.pxl", "Detailed error"));

        assert!(!result.success());
        assert_eq!(result.error_count(), 2);
    }
}
