//! Composition registry for layered sprite compositions.

use std::collections::HashMap;

use crate::models::Composition;

use super::traits::Registry;

/// Registry for named compositions.
///
/// Stores Composition objects that can be looked up by name.
/// Compositions define layered sprite arrangements for complex visuals.
#[derive(Debug, Clone, Default)]
pub struct CompositionRegistry {
    compositions: HashMap<String, Composition>,
}

impl CompositionRegistry {
    /// Create a new empty composition registry.
    pub fn new() -> Self {
        Self { compositions: HashMap::new() }
    }

    /// Register a composition in the registry.
    ///
    /// If a composition with the same name already exists, it is replaced.
    pub fn register(&mut self, composition: Composition) {
        self.compositions.insert(composition.name.clone(), composition);
    }

    /// Get a composition by name.
    pub fn get(&self, name: &str) -> Option<&Composition> {
        self.compositions.get(name)
    }

    /// Check if a composition with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.compositions.contains_key(name)
    }

    /// Get the number of compositions in the registry.
    pub fn len(&self) -> usize {
        self.compositions.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.compositions.is_empty()
    }

    /// Clear all compositions from the registry.
    pub fn clear(&mut self) {
        self.compositions.clear();
    }

    /// Get an iterator over all composition names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.compositions.keys()
    }

    /// Iterate over all compositions in the registry.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Composition)> {
        self.compositions.iter()
    }
}

impl Registry<Composition> for CompositionRegistry {
    fn contains(&self, name: &str) -> bool {
        self.compositions.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&Composition> {
        self.compositions.get(name)
    }

    fn len(&self) -> usize {
        self.compositions.len()
    }

    fn clear(&mut self) {
        self.compositions.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.compositions.keys())
    }
}
