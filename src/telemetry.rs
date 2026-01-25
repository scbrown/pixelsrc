//! Local error telemetry for pixelsrc
//!
//! Collects errors in JSONL format for analysis and debugging.
//! Privacy-safe: no personal data, only error patterns.

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

/// An error entry for the telemetry log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// ISO 8601 timestamp when the error occurred
    pub timestamp: String,
    /// The command that was running (e.g., "render", "validate", "import")
    pub command: String,
    /// The file being processed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Type of error (e.g., "parse_error", "validation_error", "io_error")
    pub error_type: String,
    /// Error context/message
    pub context: String,
    /// Suggested fix (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl ErrorEntry {
    /// Create a new error entry with the current timestamp
    pub fn new(
        command: impl Into<String>,
        error_type: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: chrono_now(),
            command: command.into(),
            file: None,
            error_type: error_type.into(),
            context: context.into(),
            suggestion: None,
        }
    }

    /// Set the file that was being processed
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set a suggested fix
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Get current timestamp in ISO 8601 format
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // Convert to date/time components (simplified UTC)
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let secs = time_secs % 60;

    // Calculate year/month/day from days since epoch (1970-01-01)
    let mut remaining_days = days as i64;
    let mut year = 1970i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }
    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, mins, secs)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Error collector that writes to a JSONL file
pub struct ErrorCollector {
    /// Path to the error log file
    path: std::path::PathBuf,
    /// Whether collection is enabled
    enabled: bool,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new(path: impl AsRef<Path>, enabled: bool) -> Self {
        Self { path: path.as_ref().to_path_buf(), enabled }
    }

    /// Check if error collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Log an error entry (appends to JSONL file)
    pub fn log(&self, entry: &ErrorEntry) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let file = OpenOptions::new().create(true).append(true).open(&self.path)?;

        let mut writer = BufWriter::new(file);
        let json = serde_json::to_string(entry).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        writeln!(writer, "{}", json)?;
        writer.flush()?;

        Ok(())
    }

    /// Log an error with a simple interface
    pub fn log_error(
        &self,
        command: &str,
        error_type: &str,
        context: &str,
        file: Option<&str>,
        suggestion: Option<&str>,
    ) -> std::io::Result<()> {
        let mut entry = ErrorEntry::new(command, error_type, context);
        if let Some(f) = file {
            entry = entry.with_file(f);
        }
        if let Some(s) = suggestion {
            entry = entry.with_suggestion(s);
        }
        self.log(&entry)
    }
}

// Global error collector (thread-local to avoid synchronization)
thread_local! {
    static COLLECTOR: std::cell::RefCell<Option<ErrorCollector>> = const { std::cell::RefCell::new(None) };
}

/// Initialize the global error collector
pub fn init_collector(path: impl AsRef<Path>, enabled: bool) {
    COLLECTOR.with(|c| {
        *c.borrow_mut() = Some(ErrorCollector::new(path, enabled));
    });
}

/// Log an error using the global collector
pub fn log_error(entry: &ErrorEntry) {
    COLLECTOR.with(|c| {
        if let Some(ref collector) = *c.borrow() {
            let _ = collector.log(entry);
        }
    });
}

/// Check if the global collector is enabled
pub fn is_collection_enabled() -> bool {
    COLLECTOR.with(|c| c.borrow().as_ref().map(|c| c.is_enabled()).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_error_entry_creation() {
        let entry = ErrorEntry::new("render", "parse_error", "Invalid JSON on line 5");
        assert_eq!(entry.command, "render");
        assert_eq!(entry.error_type, "parse_error");
        assert_eq!(entry.context, "Invalid JSON on line 5");
        assert!(entry.file.is_none());
        assert!(entry.suggestion.is_none());
    }

    #[test]
    fn test_error_entry_with_file() {
        let entry = ErrorEntry::new("validate", "undefined_token", "Token {x} not in palette")
            .with_file("sprites/hero.pxl");
        assert_eq!(entry.file, Some("sprites/hero.pxl".to_string()));
    }

    #[test]
    fn test_error_entry_with_suggestion() {
        let entry = ErrorEntry::new("validate", "undefined_token", "Token {x} not in palette")
            .with_suggestion("Add {x} to the palette colors");
        assert_eq!(entry.suggestion, Some("Add {x} to the palette colors".to_string()));
    }

    #[test]
    fn test_error_collector_disabled() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("errors.jsonl");

        let collector = ErrorCollector::new(&path, false);
        let entry = ErrorEntry::new("test", "test_error", "test context");

        // Should succeed but not write anything
        collector.log(&entry).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_error_collector_enabled() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("errors.jsonl");

        let collector = ErrorCollector::new(&path, true);
        let entry = ErrorEntry::new("render", "parse_error", "Invalid JSON")
            .with_file("test.pxl")
            .with_suggestion("Check line 5");

        collector.log(&entry).unwrap();

        // Verify file was created and contains valid JSONL
        let contents = fs::read_to_string(&path).unwrap();
        let parsed: ErrorEntry = serde_json::from_str(contents.trim()).unwrap();
        assert_eq!(parsed.command, "render");
        assert_eq!(parsed.error_type, "parse_error");
        assert_eq!(parsed.context, "Invalid JSON");
        assert_eq!(parsed.file, Some("test.pxl".to_string()));
        assert_eq!(parsed.suggestion, Some("Check line 5".to_string()));
    }

    #[test]
    fn test_error_collector_append_mode() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("errors.jsonl");

        let collector = ErrorCollector::new(&path, true);

        // Log two entries
        collector.log(&ErrorEntry::new("render", "error1", "context1")).unwrap();
        collector.log(&ErrorEntry::new("validate", "error2", "context2")).unwrap();

        // Verify both entries are in the file
        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);

        let entry1: ErrorEntry = serde_json::from_str(lines[0]).unwrap();
        let entry2: ErrorEntry = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(entry1.command, "render");
        assert_eq!(entry2.command, "validate");
    }

    #[test]
    fn test_chrono_now_format() {
        let timestamp = chrono_now();
        // Should be ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
        assert!(timestamp.len() == 20);
        assert!(timestamp.contains('T'));
        assert!(timestamp.ends_with('Z'));
    }
}
