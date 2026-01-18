//! Build manifest for tracking build state and enabling incremental builds.
//!
//! The manifest records information about each built target including source
//! file hashes, output paths, and timestamps. This enables the build system
//! to skip targets that are already up-to-date.
//!
//! # Manifest Format
//!
//! The manifest is stored as JSON in `.pxl-manifest.json` in the output directory:
//!
//! ```json
//! {
//!   "version": 1,
//!   "created_at": "2024-01-15T10:30:00Z",
//!   "updated_at": "2024-01-15T10:35:00Z",
//!   "targets": {
//!     "atlas:characters": {
//!       "sources": {
//!         "src/player.pxl": "abc123...",
//!         "src/enemy.pxl": "def456..."
//!       },
//!       "outputs": ["build/characters.png", "build/characters.json"],
//!       "built_at": "2024-01-15T10:35:00Z"
//!     }
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Current manifest format version.
const MANIFEST_VERSION: u32 = 1;

/// Default manifest filename.
pub const MANIFEST_FILENAME: &str = ".pxl-manifest.json";

/// Error during manifest operations.
#[derive(Debug)]
pub enum ManifestError {
    /// IO error
    Io(std::io::Error),
    /// JSON parsing error
    Json(serde_json::Error),
    /// Version mismatch
    VersionMismatch { expected: u32, found: u32 },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "IO error: {}", e),
            ManifestError::Json(e) => write!(f, "JSON error: {}", e),
            ManifestError::VersionMismatch { expected, found } => {
                write!(f, "Manifest version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for ManifestError {}

impl From<std::io::Error> for ManifestError {
    fn from(e: std::io::Error) -> Self {
        ManifestError::Io(e)
    }
}

impl From<serde_json::Error> for ManifestError {
    fn from(e: serde_json::Error) -> Self {
        ManifestError::Json(e)
    }
}

/// Build manifest tracking all built targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    /// Manifest format version
    pub version: u32,
    /// When the manifest was first created
    pub created_at: String,
    /// When the manifest was last updated
    pub updated_at: String,
    /// Information about each built target
    pub targets: HashMap<String, TargetManifest>,
}

impl BuildManifest {
    /// Create a new empty manifest.
    pub fn new() -> Self {
        let now = format_timestamp(SystemTime::now());
        Self {
            version: MANIFEST_VERSION,
            created_at: now.clone(),
            updated_at: now,
            targets: HashMap::new(),
        }
    }

    /// Load a manifest from a file.
    ///
    /// Returns `Ok(None)` if the file doesn't exist.
    pub fn load(path: &Path) -> Result<Option<Self>, ManifestError> {
        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let manifest: BuildManifest = serde_json::from_reader(reader)?;

        // Check version compatibility
        if manifest.version != MANIFEST_VERSION {
            return Err(ManifestError::VersionMismatch {
                expected: MANIFEST_VERSION,
                found: manifest.version,
            });
        }

        Ok(Some(manifest))
    }

    /// Load a manifest from the default location in the output directory.
    pub fn load_from_dir(out_dir: &Path) -> Result<Option<Self>, ManifestError> {
        Self::load(&out_dir.join(MANIFEST_FILENAME))
    }

    /// Save the manifest to a file.
    pub fn save(&mut self, path: &Path) -> Result<(), ManifestError> {
        // Update timestamp
        self.updated_at = format_timestamp(SystemTime::now());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    /// Save the manifest to the default location in the output directory.
    pub fn save_to_dir(&mut self, out_dir: &Path) -> Result<(), ManifestError> {
        self.save(&out_dir.join(MANIFEST_FILENAME))
    }

    /// Record a successful build for a target.
    pub fn record_build(
        &mut self,
        target_id: &str,
        sources: &[PathBuf],
        outputs: &[PathBuf],
    ) -> Result<(), ManifestError> {
        let mut source_hashes = HashMap::new();
        for source in sources {
            if source.exists() {
                let hash = hash_file(source)?;
                source_hashes.insert(source.to_string_lossy().to_string(), hash);
            }
        }

        let target_manifest = TargetManifest {
            sources: source_hashes,
            outputs: outputs.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            built_at: format_timestamp(SystemTime::now()),
        };

        self.targets.insert(target_id.to_string(), target_manifest);
        Ok(())
    }

    /// Check if a target needs to be rebuilt.
    ///
    /// A target needs rebuilding if:
    /// - It has never been built
    /// - Any source file has changed (different hash)
    /// - Any source file is missing from the previous build
    /// - Any output file is missing
    pub fn needs_rebuild(
        &self,
        target_id: &str,
        sources: &[PathBuf],
    ) -> Result<bool, ManifestError> {
        let target = match self.targets.get(target_id) {
            Some(t) => t,
            None => return Ok(true), // Never built
        };

        // Check if any output is missing
        for output in &target.outputs {
            if !Path::new(output).exists() {
                return Ok(true);
            }
        }

        // Check if source count changed
        if sources.len() != target.sources.len() {
            return Ok(true);
        }

        // Check if any source has changed
        for source in sources {
            let source_str = source.to_string_lossy().to_string();
            match target.sources.get(&source_str) {
                None => return Ok(true), // New source file
                Some(old_hash) => {
                    if !source.exists() {
                        return Ok(true); // Source deleted
                    }
                    let current_hash = hash_file(source)?;
                    if &current_hash != old_hash {
                        return Ok(true); // Source changed
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get the manifest entry for a target.
    pub fn get_target(&self, target_id: &str) -> Option<&TargetManifest> {
        self.targets.get(target_id)
    }

    /// Remove a target from the manifest.
    pub fn remove_target(&mut self, target_id: &str) -> Option<TargetManifest> {
        self.targets.remove(target_id)
    }

    /// Get all target IDs in the manifest.
    pub fn target_ids(&self) -> impl Iterator<Item = &String> {
        self.targets.keys()
    }

    /// Get the number of targets in the manifest.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Check if the manifest is empty.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Clear all targets from the manifest.
    pub fn clear(&mut self) {
        self.targets.clear();
    }
}

impl Default for BuildManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Manifest entry for a single build target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetManifest {
    /// Map of source file paths to their content hashes
    pub sources: HashMap<String, String>,
    /// Output file paths produced by this target
    pub outputs: Vec<String>,
    /// When this target was last built
    pub built_at: String,
}

impl TargetManifest {
    /// Get the build timestamp.
    pub fn built_at(&self) -> &str {
        &self.built_at
    }

    /// Get the list of output files.
    pub fn outputs(&self) -> &[String] {
        &self.outputs
    }

    /// Get the source file hashes.
    pub fn sources(&self) -> &HashMap<String, String> {
        &self.sources
    }
}

/// Compute a hash of a file's contents.
///
/// Uses a simple but fast hash for build tracking purposes.
fn hash_file(path: &Path) -> Result<String, ManifestError> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    // Simple hash: FNV-1a
    let hash = fnv1a_hash(&contents);
    Ok(format!("{:016x}", hash))
}

/// FNV-1a hash algorithm.
fn fnv1a_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Format a SystemTime as an ISO 8601 timestamp string.
fn format_timestamp(time: SystemTime) -> String {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // Simple UTC timestamp without timezone library
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Calculate year/month/day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days as i64);

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, minutes, seconds)
}

/// Convert days since Unix epoch to year/month/day.
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    // Simplified algorithm - accurate for dates from 1970 onwards
    let mut remaining_days = days;
    let mut year = 1970i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let days_in_months: [i64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for days_in_month in days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days as u32 + 1;
    (year, month, day)
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_manifest_new() {
        let manifest = BuildManifest::new();
        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert!(manifest.targets.is_empty());
        assert!(!manifest.created_at.is_empty());
        assert!(!manifest.updated_at.is_empty());
    }

    #[test]
    fn test_manifest_record_build() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = temp.path().join("build/test.png");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source.clone()], &[output.clone()]).unwrap();

        assert_eq!(manifest.len(), 1);
        let target = manifest.get_target("sprite:test").unwrap();
        assert_eq!(target.sources.len(), 1);
        assert_eq!(target.outputs.len(), 1);
    }

    #[test]
    fn test_manifest_needs_rebuild_never_built() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");

        let manifest = BuildManifest::new();
        assert!(manifest.needs_rebuild("sprite:test", &[source]).unwrap());
    }

    #[test]
    fn test_manifest_needs_rebuild_up_to_date() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source.clone()], &[output]).unwrap();

        assert!(!manifest.needs_rebuild("sprite:test", &[source]).unwrap());
    }

    #[test]
    fn test_manifest_needs_rebuild_source_changed() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "original content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source.clone()], &[output]).unwrap();

        // Modify source file
        create_test_file(temp.path(), "src/test.pxl", "modified content");

        assert!(manifest.needs_rebuild("sprite:test", &[source]).unwrap());
    }

    #[test]
    fn test_manifest_needs_rebuild_output_missing() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = temp.path().join("build/test.png");

        // Create output temporarily
        create_test_file(temp.path(), "build/test.png", "output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source.clone()], &[output.clone()]).unwrap();

        // Delete output
        fs::remove_file(&output).unwrap();

        assert!(manifest.needs_rebuild("sprite:test", &[source]).unwrap());
    }

    #[test]
    fn test_manifest_needs_rebuild_new_source() {
        let temp = TempDir::new().unwrap();
        let source1 = create_test_file(temp.path(), "src/test1.pxl", "content 1");
        let source2 = create_test_file(temp.path(), "src/test2.pxl", "content 2");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("atlas:test", &[source1.clone()], &[output]).unwrap();

        // Now build with additional source
        assert!(manifest.needs_rebuild("atlas:test", &[source1, source2]).unwrap());
    }

    #[test]
    fn test_manifest_save_load() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = temp.path().join("build/test.png");
        let manifest_path = temp.path().join(MANIFEST_FILENAME);

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source], &[output]).unwrap();
        manifest.save(&manifest_path).unwrap();

        let loaded = BuildManifest::load(&manifest_path).unwrap().unwrap();
        assert_eq!(loaded.version, MANIFEST_VERSION);
        assert_eq!(loaded.len(), 1);
        assert!(loaded.get_target("sprite:test").is_some());
    }

    #[test]
    fn test_manifest_load_nonexistent() {
        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join(MANIFEST_FILENAME);

        let result = BuildManifest::load(&manifest_path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_manifest_remove_target() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = temp.path().join("build/test.png");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source], &[output]).unwrap();

        assert_eq!(manifest.len(), 1);
        manifest.remove_target("sprite:test");
        assert_eq!(manifest.len(), 0);
    }

    #[test]
    fn test_manifest_clear() {
        let temp = TempDir::new().unwrap();
        let source1 = create_test_file(temp.path(), "src/test1.pxl", "content 1");
        let source2 = create_test_file(temp.path(), "src/test2.pxl", "content 2");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test1", &[source1], &[]).unwrap();
        manifest.record_build("sprite:test2", &[source2], &[]).unwrap();

        assert_eq!(manifest.len(), 2);
        manifest.clear();
        assert!(manifest.is_empty());
    }

    #[test]
    fn test_fnv1a_hash() {
        let hash1 = fnv1a_hash(b"hello");
        let hash2 = fnv1a_hash(b"hello");
        let hash3 = fnv1a_hash(b"world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_file() {
        let temp = TempDir::new().unwrap();
        let path = create_test_file(temp.path(), "test.txt", "test content");

        let hash1 = hash_file(&path).unwrap();
        let hash2 = hash_file(&path).unwrap();
        assert_eq!(hash1, hash2);

        // Modify file
        create_test_file(temp.path(), "test.txt", "different content");
        let hash3 = hash_file(&path).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_format_timestamp() {
        let time = SystemTime::UNIX_EPOCH;
        let ts = format_timestamp(time);
        assert_eq!(ts, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_days_to_ymd() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        assert_eq!(days_to_ymd(365), (1971, 1, 1));
        assert_eq!(days_to_ymd(366), (1971, 1, 2));

        // 2000 is a leap year
        // Days from 1970-01-01 to 2000-01-01
        let days_to_2000 = 30 * 365 + 7; // 30 years + 7 leap years (72,76,80,84,88,92,96)
        assert_eq!(days_to_ymd(days_to_2000), (2000, 1, 1));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(1970));
        assert!(is_leap_year(1972));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
    }

    #[test]
    fn test_target_manifest_accessors() {
        let target = TargetManifest {
            sources: HashMap::from([("src/test.pxl".to_string(), "abc123".to_string())]),
            outputs: vec!["build/test.png".to_string()],
            built_at: "2024-01-15T10:30:00Z".to_string(),
        };

        assert_eq!(target.built_at(), "2024-01-15T10:30:00Z");
        assert_eq!(target.outputs(), &["build/test.png"]);
        assert_eq!(target.sources().len(), 1);
    }

    #[test]
    fn test_manifest_target_ids() {
        let temp = TempDir::new().unwrap();
        let source1 = create_test_file(temp.path(), "src/test1.pxl", "content 1");
        let source2 = create_test_file(temp.path(), "src/test2.pxl", "content 2");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:a", &[source1], &[]).unwrap();
        manifest.record_build("sprite:b", &[source2], &[]).unwrap();

        let ids: Vec<&String> = manifest.target_ids().collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&&"sprite:a".to_string()));
        assert!(ids.contains(&&"sprite:b".to_string()));
    }

    #[test]
    fn test_manifest_default() {
        let manifest = BuildManifest::default();
        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert!(manifest.is_empty());
    }
}
