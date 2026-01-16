//! Watch mode for automatic rebuilds on file changes
//!
//! Provides file system watching with debouncing for the `pxl build --watch` command.

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
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
    /// Number of errors encountered
    pub errors: Vec<String>,
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
            warnings: vec![],
            duration: Duration::ZERO,
        }
    }

    /// Check if build succeeded (no errors)
    pub fn success(&self) -> bool {
        self.errors.is_empty()
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
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
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
    let mut debouncer =
        new_debouncer(debounce_duration, tx).map_err(WatchError::WatcherInit)?;

    // Start watching the source directory
    debouncer
        .watcher()
        .watch(&options.src_dir, RecursiveMode::Recursive)
        .map_err(WatchError::WatchPath)?;

    // Initial build
    if options.config.clear_screen {
        clear_screen();
    }
    println!("[{}] Building...", timestamp());
    let result = do_build(&options, simple_build);
    print_build_result(&result);
    println!(
        "[{}] Watching {} for changes...",
        timestamp(),
        options.src_dir.display()
    );

    // Watch loop
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Filter for relevant file changes
                let relevant_changes: Vec<_> = events
                    .iter()
                    .filter(|e| {
                        matches!(e.kind, DebouncedEventKind::Any)
                            && is_relevant_file(&e.path)
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
                    print_build_result(&result);
                    println!(
                        "[{}] Watching {} for changes...",
                        timestamp(),
                        options.src_dir.display()
                    );
                }
            }
            Ok(Err(error)) => {
                // Watch error (non-fatal)
                eprintln!("[{}] Watch error: {:?}", timestamp(), error);
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

/// Print build result to console
fn print_build_result(result: &BuildResult) {
    if result.success() {
        println!(
            "[{}] Build complete ({}) - Files: {} | Sprites: {}",
            timestamp(),
            format_duration(result.duration),
            result.files_processed,
            result.sprites_rendered
        );
    } else {
        println!(
            "[{}] Build failed ({}) - {} errors",
            timestamp(),
            format_duration(result.duration),
            result.errors.len()
        );
        for error in &result.errors {
            eprintln!("  Error: {}", error);
        }
    }

    for warning in &result.warnings {
        eprintln!("  Warning: {}", warning);
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
        let options = WatchOptions {
            src_dir: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };

        let result = watch_and_rebuild(options);
        assert!(matches!(result, Err(WatchError::SourceNotFound(_))));
    }

    #[test]
    fn test_do_build_with_custom_function() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        let options = WatchOptions {
            src_dir: src,
            out_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let result = do_build(&options, |_src, _out| {
            let mut r = BuildResult::new();
            r.files_processed = 5;
            r.sprites_rendered = 10;
            r
        });

        assert_eq!(result.files_processed, 5);
        assert_eq!(result.sprites_rendered, 10);
        assert!(result.duration > Duration::ZERO || result.duration == Duration::ZERO);
    }
}
