//! Transform registry for user-defined transforms.

use std::collections::HashMap;

use crate::models::TransformDef;
use crate::transforms::{self, Transform, TransformError};

use super::traits::Registry;

/// Registry for user-defined transforms.
///
/// Stores TransformDef objects that can be referenced by name in transform arrays.
/// Supports parameterized transforms, keyframe animations, and transform cycling.
#[derive(Debug, Clone, Default)]
pub struct TransformRegistry {
    transforms: HashMap<String, TransformDef>,
}

impl TransformRegistry {
    /// Create a new empty transform registry.
    pub fn new() -> Self {
        Self { transforms: HashMap::new() }
    }

    /// Register a user-defined transform.
    pub fn register(&mut self, transform: TransformDef) {
        self.transforms.insert(transform.name.clone(), transform);
    }

    /// Get a transform definition by name.
    pub fn get(&self, name: &str) -> Option<&TransformDef> {
        self.transforms.get(name)
    }

    /// Check if a transform with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.transforms.contains_key(name)
    }

    /// Get the number of registered transforms.
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Clear all transforms from the registry.
    pub fn clear(&mut self) {
        self.transforms.clear();
    }

    /// Iterate over all transforms in the registry.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &TransformDef)> {
        self.transforms.iter()
    }

    /// Expand a user-defined transform for a specific frame.
    ///
    /// If the transform is a simple ops-only transform, returns the ops directly.
    /// For keyframe animations, generates the appropriate transforms for the given frame.
    /// For cycling transforms, returns the transforms for the current cycle position.
    ///
    /// # Arguments
    /// * `name` - The name of the user-defined transform
    /// * `params` - Parameter values for parameterized transforms
    /// * `frame` - Current frame number (for keyframe animations)
    /// * `total_frames` - Total frames in the animation
    pub fn expand(
        &self,
        name: &str,
        params: &HashMap<String, f64>,
        frame: u32,
        total_frames: u32,
    ) -> Result<Vec<Transform>, TransformError> {
        let transform_def = self
            .transforms
            .get(name)
            .ok_or_else(|| TransformError::UnknownOperation(name.to_string()))?;

        transforms::generate_frame_transforms(transform_def, frame, total_frames, params)
    }

    /// Expand a simple (non-animated) user-defined transform.
    ///
    /// For simple ops-only transforms, returns the ops.
    /// For keyframe animations, returns transforms for frame 0.
    pub fn expand_simple(
        &self,
        name: &str,
        params: &HashMap<String, f64>,
    ) -> Result<Vec<Transform>, TransformError> {
        self.expand(name, params, 0, 1)
    }
}

impl Registry<TransformDef> for TransformRegistry {
    fn contains(&self, name: &str) -> bool {
        self.transforms.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&TransformDef> {
        self.transforms.get(name)
    }

    fn len(&self) -> usize {
        self.transforms.len()
    }

    fn clear(&mut self) {
        self.transforms.clear();
    }

    fn names(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.transforms.keys())
    }
}
