//! Build manifest for tracking build state and enabling incremental builds.
//!
//! The manifest records information about each built target including source
//! file hashes, output paths, timestamps, checksums, and build statistics.
//! This enables the build system to skip targets that are already up-to-date
//! and verify output integrity.
//!
//! # Manifest Format
//!
//! The manifest is stored as JSON in `.pxl-manifest.json` in the output directory:
//!
//! ```json
//! {
//!   "version": 2,
//!   "created_at": "2024-01-15T10:30:00Z",
//!   "updated_at": "2024-01-15T10:35:00Z",
//!   "targets": {
//!     "atlas:characters": {
//!       "sources": {
//!         "src/player.pxl": "abc123...",
//!         "src/enemy.pxl": "def456..."
//!       },
//!       "outputs": ["build/characters.png", "build/characters.json"],
//!       "built_at": "2024-01-15T10:35:00Z",
//!       "output_checksums": {
//!         "build/characters.png": "789abc...",
//!         "build/characters.json": "012def..."
//!       },
//!       "output_sizes": {
//!         "build/characters.png": 4096,
//!         "build/characters.json": 512
//!       },
//!       "duration_ms": 150
//!     }
//!   },
//!   "stats": {
//!     "total_targets": 5,
//!     "success_count": 5,
//!     "skipped_count": 0,
//!     "failed_count": 0,
//!     "total_duration_ms": 750,
//!     "total_output_size": 32768,
//!     "total_output_files": 10
//!   },
//!   "metadata": {
//!     "project_name": "my-game",
//!     "project_version": "1.0.0",
//!     "builder_version": "0.1.0",
//!     "build_mode": "strict",
//!     "scale": 2
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
const MANIFEST_VERSION: u32 = 2;

/// Default manifest filename.
pub const MANIFEST_FILENAME: &str = ".pxl-manifest.json";

/// Error during manifest operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ManifestError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Version mismatch
    #[error("Manifest version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
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
    /// Aggregate build statistics from the last build
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<BuildStats>,
    /// Build metadata (project info, tool version)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BuildMetadata>,
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
            stats: None,
            metadata: None,
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
        self.record_build_with_duration(target_id, sources, outputs, None)
    }

    /// Record a successful build for a target with duration.
    pub fn record_build_with_duration(
        &mut self,
        target_id: &str,
        sources: &[PathBuf],
        outputs: &[PathBuf],
        duration_ms: Option<u64>,
    ) -> Result<(), ManifestError> {
        let mut source_hashes = HashMap::new();
        for source in sources {
            if source.exists() {
                let hash = hash_file(source)?;
                source_hashes.insert(source.to_string_lossy().to_string(), hash);
            }
        }

        // Compute output checksums and sizes
        let mut output_checksums = HashMap::new();
        let mut output_sizes = HashMap::new();
        for output in outputs {
            if output.exists() {
                let output_str = output.to_string_lossy().to_string();
                let hash = hash_file(output)?;
                output_checksums.insert(output_str.clone(), hash);

                if let Ok(metadata) = fs::metadata(output) {
                    output_sizes.insert(output_str, metadata.len());
                }
            }
        }

        let target_manifest = TargetManifest {
            sources: source_hashes,
            outputs: outputs.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            built_at: format_timestamp(SystemTime::now()),
            output_checksums,
            output_sizes,
            duration_ms,
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

    /// Set build metadata.
    pub fn set_metadata(&mut self, metadata: BuildMetadata) {
        self.metadata = Some(metadata);
    }

    /// Get build metadata.
    pub fn metadata(&self) -> Option<&BuildMetadata> {
        self.metadata.as_ref()
    }

    /// Set build stats.
    pub fn set_stats(&mut self, stats: BuildStats) {
        self.stats = Some(stats);
    }

    /// Get build stats.
    pub fn stats(&self) -> Option<&BuildStats> {
        self.stats.as_ref()
    }

    /// Compute aggregate stats from the current targets.
    ///
    /// This recomputes stats from existing target entries.
    pub fn compute_stats(&mut self) {
        let mut stats = BuildStats { total_targets: self.targets.len(), ..Default::default() };

        for target in self.targets.values() {
            // Count as success (we only record successful builds)
            stats.success_count += 1;

            // Sum output sizes
            for size in target.output_sizes.values() {
                stats.total_output_size += size;
            }
            stats.total_output_files += target.outputs.len();

            // Sum durations
            if let Some(duration) = target.duration_ms {
                stats.total_duration_ms += duration;
            }
        }

        self.stats = Some(stats);
    }

    /// Compute the total output size across all targets.
    pub fn total_output_size(&self) -> u64 {
        self.targets.values().flat_map(|t| t.output_sizes.values()).sum()
    }

    /// Verify output file checksums.
    ///
    /// Returns a list of target IDs whose output files have changed.
    pub fn verify_outputs(&self) -> Result<Vec<String>, ManifestError> {
        let mut changed = Vec::new();

        for (target_id, target) in &self.targets {
            for (output_path, expected_hash) in &target.output_checksums {
                let path = Path::new(output_path);
                if !path.exists() {
                    changed.push(target_id.clone());
                    break;
                }

                let current_hash = hash_file(path)?;
                if &current_hash != expected_hash {
                    changed.push(target_id.clone());
                    break;
                }
            }
        }

        Ok(changed)
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
    /// Map of output file paths to their checksums (for verification)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub output_checksums: HashMap<String, String>,
    /// Map of output file paths to their sizes in bytes
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub output_sizes: HashMap<String, u64>,
    /// Build duration in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Aggregate build statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildStats {
    /// Total number of targets
    pub total_targets: usize,
    /// Number of successful builds
    pub success_count: usize,
    /// Number of skipped targets (already up to date)
    pub skipped_count: usize,
    /// Number of failed targets
    pub failed_count: usize,
    /// Total build duration in milliseconds
    pub total_duration_ms: u64,
    /// Total size of all output files in bytes
    pub total_output_size: u64,
    /// Total number of output files
    pub total_output_files: usize,
}

/// Build metadata with project and tool information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildMetadata {
    /// Project name from config
    pub project_name: String,
    /// Project version from config
    pub project_version: String,
    /// Builder tool version
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builder_version: Option<String>,
    /// Build mode (strict, verbose)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_mode: Option<String>,
    /// Scale factor used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
}

impl BuildMetadata {
    /// Create new build metadata with project info.
    pub fn new(project_name: &str, project_version: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            project_version: project_version.to_string(),
            builder_version: None,
            build_mode: None,
            scale: None,
        }
    }

    /// Set the builder version.
    pub fn with_builder_version(mut self, version: &str) -> Self {
        self.builder_version = Some(version.to_string());
        self
    }

    /// Set the build mode.
    pub fn with_build_mode(mut self, mode: &str) -> Self {
        self.build_mode = Some(mode.to_string());
        self
    }

    /// Set the scale factor.
    pub fn with_scale(mut self, scale: u32) -> Self {
        self.scale = Some(scale);
        self
    }
}

impl BuildStats {
    /// Create stats from build counts and duration.
    pub fn new(
        success_count: usize,
        skipped_count: usize,
        failed_count: usize,
        total_duration_ms: u64,
    ) -> Self {
        Self {
            total_targets: success_count + skipped_count + failed_count,
            success_count,
            skipped_count,
            failed_count,
            total_duration_ms,
            total_output_size: 0,
            total_output_files: 0,
        }
    }

    /// Check if the build was completely successful.
    pub fn is_success(&self) -> bool {
        self.failed_count == 0
    }

    /// Get the total duration as a formatted string.
    pub fn duration_string(&self) -> String {
        let ms = self.total_duration_ms;
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            format!("{:.2}s", ms as f64 / 1000.0)
        } else {
            let mins = ms / 60_000;
            let secs = (ms % 60_000) / 1000;
            format!("{}m {}s", mins, secs)
        }
    }

    /// Format the output size as a human-readable string.
    pub fn output_size_string(&self) -> String {
        format_size(self.total_output_size)
    }
}

/// Format a byte size as a human-readable string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    }
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

    /// Get the output file checksums.
    pub fn output_checksums(&self) -> &HashMap<String, String> {
        &self.output_checksums
    }

    /// Get the output file sizes.
    pub fn output_sizes(&self) -> &HashMap<String, u64> {
        &self.output_sizes
    }

    /// Get the build duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }

    /// Get the total output size for this target.
    pub fn total_output_size(&self) -> u64 {
        self.output_sizes.values().sum()
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
        manifest
            .record_build(
                "sprite:test",
                std::slice::from_ref(&source),
                std::slice::from_ref(&output),
            )
            .unwrap();

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
        manifest.record_build("sprite:test", std::slice::from_ref(&source), &[output]).unwrap();

        assert!(!manifest.needs_rebuild("sprite:test", &[source]).unwrap());
    }

    #[test]
    fn test_manifest_needs_rebuild_source_changed() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "original content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", std::slice::from_ref(&source), &[output]).unwrap();

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
        manifest
            .record_build(
                "sprite:test",
                std::slice::from_ref(&source),
                std::slice::from_ref(&output),
            )
            .unwrap();

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
        manifest.record_build("atlas:test", std::slice::from_ref(&source1), &[output]).unwrap();

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
            output_checksums: HashMap::from([("build/test.png".to_string(), "def456".to_string())]),
            output_sizes: HashMap::from([("build/test.png".to_string(), 1024)]),
            duration_ms: Some(150),
        };

        assert_eq!(target.built_at(), "2024-01-15T10:30:00Z");
        assert_eq!(target.outputs(), &["build/test.png"]);
        assert_eq!(target.sources().len(), 1);
        assert_eq!(target.output_checksums().len(), 1);
        assert_eq!(target.output_sizes().len(), 1);
        assert_eq!(target.duration_ms(), Some(150));
        assert_eq!(target.total_output_size(), 1024);
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

    #[test]
    fn test_record_build_with_duration() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = create_test_file(temp.path(), "build/test.png", "output data");

        let mut manifest = BuildManifest::new();
        manifest
            .record_build_with_duration("sprite:test", &[source], &[output], Some(250))
            .unwrap();

        let target = manifest.get_target("sprite:test").unwrap();
        assert_eq!(target.duration_ms(), Some(250));
        assert!(!target.output_checksums().is_empty());
        assert!(!target.output_sizes().is_empty());
        assert_eq!(target.total_output_size(), 11); // "output data" = 11 bytes
    }

    #[test]
    fn test_build_metadata() {
        let metadata = BuildMetadata::new("my-game", "1.0.0")
            .with_builder_version("0.1.0")
            .with_build_mode("strict")
            .with_scale(2);

        assert_eq!(metadata.project_name, "my-game");
        assert_eq!(metadata.project_version, "1.0.0");
        assert_eq!(metadata.builder_version, Some("0.1.0".to_string()));
        assert_eq!(metadata.build_mode, Some("strict".to_string()));
        assert_eq!(metadata.scale, Some(2));
    }

    #[test]
    fn test_build_stats() {
        let stats = BuildStats::new(5, 2, 1, 1500);

        assert_eq!(stats.total_targets, 8);
        assert_eq!(stats.success_count, 5);
        assert_eq!(stats.skipped_count, 2);
        assert_eq!(stats.failed_count, 1);
        assert_eq!(stats.total_duration_ms, 1500);
        assert!(!stats.is_success());
    }

    #[test]
    fn test_build_stats_success() {
        let stats = BuildStats::new(5, 2, 0, 1000);
        assert!(stats.is_success());
    }

    #[test]
    fn test_build_stats_duration_string() {
        let stats = BuildStats::new(1, 0, 0, 500);
        assert_eq!(stats.duration_string(), "500ms");

        let stats = BuildStats::new(1, 0, 0, 2500);
        assert_eq!(stats.duration_string(), "2.50s");

        let stats = BuildStats::new(1, 0, 0, 125000);
        assert_eq!(stats.duration_string(), "2m 5s");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_manifest_set_metadata() {
        let mut manifest = BuildManifest::new();
        assert!(manifest.metadata().is_none());

        let metadata = BuildMetadata::new("test", "1.0.0");
        manifest.set_metadata(metadata);

        assert!(manifest.metadata().is_some());
        assert_eq!(manifest.metadata().unwrap().project_name, "test");
    }

    #[test]
    fn test_manifest_set_stats() {
        let mut manifest = BuildManifest::new();
        assert!(manifest.stats().is_none());

        let stats = BuildStats::new(3, 1, 0, 500);
        manifest.set_stats(stats);

        assert!(manifest.stats().is_some());
        assert_eq!(manifest.stats().unwrap().total_targets, 4);
    }

    #[test]
    fn test_manifest_compute_stats() {
        let temp = TempDir::new().unwrap();
        let source1 = create_test_file(temp.path(), "src/test1.pxl", "content 1");
        let source2 = create_test_file(temp.path(), "src/test2.pxl", "content 2");
        let output1 = create_test_file(temp.path(), "build/test1.png", "output 1"); // 8 bytes
        let output2 = create_test_file(temp.path(), "build/test2.png", "output 22"); // 9 bytes

        let mut manifest = BuildManifest::new();
        manifest
            .record_build_with_duration("sprite:test1", &[source1], &[output1], Some(100))
            .unwrap();
        manifest
            .record_build_with_duration("sprite:test2", &[source2], &[output2], Some(200))
            .unwrap();

        manifest.compute_stats();

        let stats = manifest.stats().unwrap();
        assert_eq!(stats.total_targets, 2);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.total_duration_ms, 300);
        assert_eq!(stats.total_output_files, 2);
        assert_eq!(stats.total_output_size, 17); // 8 + 9
    }

    #[test]
    fn test_manifest_total_output_size() {
        let temp = TempDir::new().unwrap();
        let source1 = create_test_file(temp.path(), "src/test1.pxl", "content 1");
        let source2 = create_test_file(temp.path(), "src/test2.pxl", "content 2");
        let output1 = create_test_file(temp.path(), "build/test1.png", "12345"); // 5 bytes
        let output2 = create_test_file(temp.path(), "build/test2.png", "1234567890"); // 10 bytes

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test1", &[source1], &[output1]).unwrap();
        manifest.record_build("sprite:test2", &[source2], &[output2]).unwrap();

        assert_eq!(manifest.total_output_size(), 15);
    }

    #[test]
    fn test_manifest_verify_outputs() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "content");
        let output = create_test_file(temp.path(), "build/test.png", "original output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source], std::slice::from_ref(&output)).unwrap();

        // No changes - should return empty
        let changed = manifest.verify_outputs().unwrap();
        assert!(changed.is_empty());

        // Modify output file
        create_test_file(temp.path(), "build/test.png", "modified output");
        let changed = manifest.verify_outputs().unwrap();
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&"sprite:test".to_string()));
    }

    #[test]
    fn test_manifest_verify_outputs_missing() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "content");
        let output = create_test_file(temp.path(), "build/test.png", "output");

        let mut manifest = BuildManifest::new();
        manifest.record_build("sprite:test", &[source], std::slice::from_ref(&output)).unwrap();

        // Delete output
        fs::remove_file(&output).unwrap();

        let changed = manifest.verify_outputs().unwrap();
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&"sprite:test".to_string()));
    }

    #[test]
    fn test_manifest_save_load_with_stats_metadata() {
        let temp = TempDir::new().unwrap();
        let source = create_test_file(temp.path(), "src/test.pxl", "test content");
        let output = create_test_file(temp.path(), "build/test.png", "output");
        let manifest_path = temp.path().join(MANIFEST_FILENAME);

        let mut manifest = BuildManifest::new();
        manifest
            .record_build_with_duration("sprite:test", &[source], &[output], Some(100))
            .unwrap();
        manifest.set_metadata(BuildMetadata::new("test-project", "2.0.0").with_scale(4));
        manifest.compute_stats();
        manifest.save(&manifest_path).unwrap();

        let loaded = BuildManifest::load(&manifest_path).unwrap().unwrap();
        assert!(loaded.metadata().is_some());
        assert_eq!(loaded.metadata().unwrap().project_name, "test-project");
        assert_eq!(loaded.metadata().unwrap().scale, Some(4));
        assert!(loaded.stats().is_some());
        assert_eq!(loaded.stats().unwrap().total_targets, 1);
    }
}
