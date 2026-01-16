//! Build context containing configuration and state for a build.

use crate::config::PxlConfig;
use std::path::{Path, PathBuf};

/// Build context containing configuration and paths for a build operation.
///
/// The context provides access to all information needed to execute a build,
/// including the configuration, project root, and output directories.
#[derive(Debug, Clone)]
pub struct BuildContext {
    /// The loaded configuration
    config: PxlConfig,
    /// Project root directory (where pxl.toml is located)
    project_root: PathBuf,
    /// Whether to run in strict mode (warnings are errors)
    strict: bool,
    /// Whether to run in verbose mode
    verbose: bool,
    /// Optional filter to build specific targets only
    target_filter: Option<Vec<String>>,
}

impl BuildContext {
    /// Create a new build context.
    ///
    /// # Arguments
    /// - `config` - The loaded configuration
    /// - `project_root` - The project root directory
    pub fn new(config: PxlConfig, project_root: PathBuf) -> Self {
        let strict = config.validate.strict;
        Self {
            config,
            project_root,
            strict,
            verbose: false,
            target_filter: None,
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &PxlConfig {
        &self.config
    }

    /// Get the project root directory.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the source directory (resolved to absolute path).
    pub fn src_dir(&self) -> PathBuf {
        self.resolve_path(&self.config.project.src)
    }

    /// Get the output directory (resolved to absolute path).
    pub fn out_dir(&self) -> PathBuf {
        self.resolve_path(&self.config.project.out)
    }

    /// Whether strict mode is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Whether verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Set strict mode.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set verbose mode.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set target filter to build only specific targets.
    pub fn with_filter(mut self, targets: Vec<String>) -> Self {
        self.target_filter = Some(targets);
        self
    }

    /// Get the target filter.
    pub fn target_filter(&self) -> Option<&[String]> {
        self.target_filter.as_deref()
    }

    /// Resolve a path relative to the project root.
    ///
    /// If the path is absolute, returns it unchanged.
    /// If relative, joins it with the project root.
    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.project_root.join(path)
        }
    }

    /// Get the default scale factor from config.
    pub fn default_scale(&self) -> u32 {
        self.config.defaults.scale
    }

    /// Get the default padding from config.
    pub fn default_padding(&self) -> u32 {
        self.config.defaults.padding
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_config;

    #[test]
    fn test_build_context_new() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root.clone());

        assert_eq!(ctx.project_root(), &root);
        assert!(!ctx.is_strict());
        assert!(!ctx.is_verbose());
    }

    #[test]
    fn test_build_context_with_strict() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root).with_strict(true);

        assert!(ctx.is_strict());
    }

    #[test]
    fn test_build_context_with_verbose() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root).with_verbose(true);

        assert!(ctx.is_verbose());
    }

    #[test]
    fn test_build_context_with_filter() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root)
            .with_filter(vec!["atlas:main".to_string()]);

        assert_eq!(ctx.target_filter(), Some(&["atlas:main".to_string()][..]));
    }

    #[test]
    fn test_build_context_resolve_path_absolute() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root);

        let absolute = Path::new("/other/path");
        assert_eq!(ctx.resolve_path(absolute), PathBuf::from("/other/path"));
    }

    #[test]
    fn test_build_context_resolve_path_relative() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root);

        let relative = Path::new("src/sprites");
        assert_eq!(ctx.resolve_path(relative), PathBuf::from("/project/src/sprites"));
    }

    #[test]
    fn test_build_context_src_dir() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root);

        assert_eq!(ctx.src_dir(), PathBuf::from("/project/src/pxl"));
    }

    #[test]
    fn test_build_context_out_dir() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root);

        assert_eq!(ctx.out_dir(), PathBuf::from("/project/build"));
    }

    #[test]
    fn test_build_context_defaults() {
        let config = default_config();
        let root = PathBuf::from("/project");
        let ctx = BuildContext::new(config, root);

        assert_eq!(ctx.default_scale(), 1);
        assert_eq!(ctx.default_padding(), 1);
    }
}
