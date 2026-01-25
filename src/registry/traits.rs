//! Common trait for registries that store named items.

/// Common trait for registries that store named items.
///
/// This trait provides a unified interface for registries that map string names to values.
/// It defines common operations like checking existence, retrieving items, and counting entries.
///
/// # Type Parameters
///
/// * `V` - The type of value stored in the registry
///
/// # Example
///
/// ```
/// use pixelsrc::registry::{Registry, PaletteRegistry};
/// use pixelsrc::models::Palette;
/// use std::collections::HashMap;
///
/// let mut registry = PaletteRegistry::new();
/// let palette = Palette {
///     name: "mono".to_string(),
///     colors: HashMap::from([("{on}".to_string(), "#FFFFFF".to_string())]),
///     ..Default::default()
/// };
/// registry.register(palette);
///
/// assert!(registry.contains("mono"));
/// assert_eq!(registry.len(), 1);
/// ```
pub trait Registry<V> {
    /// Check if an item with the given name exists in the registry.
    fn contains(&self, name: &str) -> bool;

    /// Get an item by name.
    ///
    /// Returns `None` if no item with the given name exists.
    fn get(&self, name: &str) -> Option<&V>;

    /// Get the number of items in the registry.
    fn len(&self) -> usize;

    /// Check if the registry is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all items from the registry.
    fn clear(&mut self);

    /// Get an iterator over all names in the registry.
    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_>;
}
