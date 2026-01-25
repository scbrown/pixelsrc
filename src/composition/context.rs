//! Render context for composition rendering with caching support

use image::RgbaImage;
use std::collections::HashMap;

use super::error::CompositionError;

/// Context for rendering operations with caching support.
///
/// The RenderContext stores rendered compositions to avoid redundant rendering
/// when the same composition is referenced multiple times (e.g., in nested
/// compositions or tiled layouts).
///
/// # Example
///
/// ```ignore
/// use pixelsrc::composition::RenderContext;
/// use image::RgbaImage;
///
/// let mut ctx = RenderContext::new();
///
/// // First render - compute and cache
/// if ctx.get_cached("scene").is_none() {
///     let rendered = render_composition(/* ... */);
///     ctx.cache("scene".to_string(), rendered);
/// }
///
/// // Second reference - get from cache
/// let cached = ctx.get_cached("scene").unwrap();
/// ```
#[derive(Debug, Default, Clone)]
pub struct RenderContext {
    /// Cache of rendered compositions by name
    composition_cache: HashMap<String, RgbaImage>,
    /// Stack of composition names currently being rendered (for cycle detection)
    render_stack: Vec<String>,
}

impl RenderContext {
    /// Create a new empty render context.
    pub fn new() -> Self {
        Self { composition_cache: HashMap::new(), render_stack: Vec::new() }
    }

    /// Get a cached rendered composition by name.
    ///
    /// Returns `None` if the composition has not been cached yet.
    pub fn get_cached(&self, name: &str) -> Option<&RgbaImage> {
        self.composition_cache.get(name)
    }

    /// Cache a rendered composition.
    ///
    /// If a composition with the same name was already cached, it will be replaced.
    pub fn cache(&mut self, name: String, image: RgbaImage) {
        self.composition_cache.insert(name, image);
    }

    /// Check if a composition is already cached.
    pub fn is_cached(&self, name: &str) -> bool {
        self.composition_cache.contains_key(name)
    }

    /// Clear all cached compositions.
    pub fn clear(&mut self) {
        self.composition_cache.clear();
    }

    /// Get the number of cached compositions.
    pub fn len(&self) -> usize {
        self.composition_cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.composition_cache.is_empty() && self.render_stack.is_empty()
    }

    /// Push a composition onto the render stack.
    ///
    /// Returns `Ok(())` if successful, or `Err(CompositionError::CycleDetected)`
    /// if the composition is already on the stack (indicating a cycle).
    pub fn push(&mut self, name: impl Into<String>) -> Result<(), CompositionError> {
        let name = name.into();
        if self.render_stack.contains(&name) {
            let mut cycle_path: Vec<String> =
                self.render_stack.iter().skip_while(|n| *n != &name).cloned().collect();
            cycle_path.push(name);
            return Err(CompositionError::CycleDetected { cycle_path });
        }
        self.render_stack.push(name);
        Ok(())
    }

    /// Pop a composition from the render stack.
    pub fn pop(&mut self) -> Option<String> {
        self.render_stack.pop()
    }

    /// Check if a composition is currently being rendered.
    pub fn contains(&self, name: &str) -> bool {
        self.render_stack.iter().any(|n| n == name)
    }

    /// Get the current depth of the render stack.
    pub fn depth(&self) -> usize {
        self.render_stack.len()
    }

    /// Get the current render path as a slice.
    pub fn path(&self) -> &[String] {
        &self.render_stack
    }
}
