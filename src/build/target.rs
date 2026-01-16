//! Build target definitions.
//!
//! A build target represents something that can be built by the pipeline,
//! such as a sprite atlas, animation spritesheet, or export format.

use std::path::PathBuf;

/// Type of build target.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// Individual sprite render
    Sprite,
    /// Texture atlas combining multiple sprites
    Atlas,
    /// Animation spritesheet
    Animation,
    /// Preview GIF for animation
    AnimationPreview,
    /// Game engine export (Godot, Unity, etc.)
    Export,
}

impl std::fmt::Display for TargetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetKind::Sprite => write!(f, "sprite"),
            TargetKind::Atlas => write!(f, "atlas"),
            TargetKind::Animation => write!(f, "animation"),
            TargetKind::AnimationPreview => write!(f, "preview"),
            TargetKind::Export => write!(f, "export"),
        }
    }
}

/// A build target representing work to be done.
#[derive(Debug, Clone)]
pub struct BuildTarget {
    /// Unique identifier for this target (e.g., "atlas:characters")
    pub id: String,
    /// What kind of target this is
    pub kind: TargetKind,
    /// Human-readable name
    pub name: String,
    /// Source files that contribute to this target
    pub sources: Vec<PathBuf>,
    /// Output path for this target
    pub output: PathBuf,
    /// Dependencies (other target IDs that must be built first)
    pub dependencies: Vec<String>,
}

impl BuildTarget {
    /// Create a new sprite target.
    pub fn sprite(name: String, source: PathBuf, output: PathBuf) -> Self {
        let id = format!("sprite:{}", name);
        Self {
            id,
            kind: TargetKind::Sprite,
            name,
            sources: vec![source],
            output,
            dependencies: vec![],
        }
    }

    /// Create a new atlas target.
    pub fn atlas(name: String, sources: Vec<PathBuf>, output: PathBuf) -> Self {
        let id = format!("atlas:{}", name);
        Self {
            id,
            kind: TargetKind::Atlas,
            name,
            sources,
            output,
            dependencies: vec![],
        }
    }

    /// Create a new animation target.
    pub fn animation(name: String, source: PathBuf, output: PathBuf) -> Self {
        let id = format!("animation:{}", name);
        Self {
            id,
            kind: TargetKind::Animation,
            name,
            sources: vec![source],
            output,
            dependencies: vec![],
        }
    }

    /// Create a new animation preview target.
    pub fn animation_preview(name: String, source: PathBuf, output: PathBuf) -> Self {
        let id = format!("preview:{}", name);
        let dep = format!("animation:{}", name);
        Self {
            id,
            kind: TargetKind::AnimationPreview,
            name,
            sources: vec![source],
            output,
            dependencies: vec![dep],
        }
    }

    /// Create a new export target.
    pub fn export(name: String, format: String, output: PathBuf) -> Self {
        let id = format!("export:{}:{}", format, name);
        Self {
            id,
            kind: TargetKind::Export,
            name,
            sources: vec![],
            output,
            dependencies: vec![],
        }
    }

    /// Add a dependency to this target.
    pub fn with_dependency(mut self, dep: String) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add multiple dependencies to this target.
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies.extend(deps);
        self
    }

    /// Check if this target matches a filter string.
    ///
    /// Supports patterns like:
    /// - Exact match: "atlas:characters"
    /// - Kind match: "atlas:*" or just "atlas"
    /// - Name match: "*:characters"
    pub fn matches_filter(&self, filter: &str) -> bool {
        // Exact match
        if self.id == filter {
            return true;
        }

        // Kind-only match (e.g., "atlas" matches all atlases)
        if self.kind.to_string() == filter {
            return true;
        }

        // Pattern match
        if let Some((kind_pat, name_pat)) = filter.split_once(':') {
            let kind_matches = kind_pat == "*" || kind_pat == self.kind.to_string();
            let name_matches = name_pat == "*" || name_pat == self.name;
            return kind_matches && name_matches;
        }

        false
    }
}

/// A collection of build targets with dependency information.
#[derive(Debug, Default)]
pub struct BuildPlan {
    /// All targets in the build
    targets: Vec<BuildTarget>,
}

impl BuildPlan {
    /// Create a new empty build plan.
    pub fn new() -> Self {
        Self { targets: vec![] }
    }

    /// Add a target to the plan.
    pub fn add_target(&mut self, target: BuildTarget) {
        self.targets.push(target);
    }

    /// Get all targets in the plan.
    pub fn targets(&self) -> &[BuildTarget] {
        &self.targets
    }

    /// Get the number of targets in the plan.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Check if the plan is empty.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Filter targets to only those matching the given patterns.
    pub fn filter(mut self, patterns: &[String]) -> Self {
        if patterns.is_empty() {
            return self;
        }

        self.targets.retain(|t| {
            patterns.iter().any(|p| t.matches_filter(p))
        });
        self
    }

    /// Get targets in build order (respecting dependencies).
    ///
    /// Returns targets sorted so that dependencies come before dependents.
    /// Returns an error if there are circular dependencies.
    pub fn build_order(&self) -> Result<Vec<&BuildTarget>, BuildOrderError> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for target in &self.targets {
            self.visit_target(target, &mut visited, &mut visiting, &mut result)?;
        }

        Ok(result)
    }

    fn visit_target<'a>(
        &'a self,
        target: &'a BuildTarget,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
        result: &mut Vec<&'a BuildTarget>,
    ) -> Result<(), BuildOrderError> {
        if visited.contains(&target.id) {
            return Ok(());
        }

        if visiting.contains(&target.id) {
            return Err(BuildOrderError::CyclicDependency(target.id.clone()));
        }

        visiting.insert(target.id.clone());

        // Visit dependencies first
        for dep_id in &target.dependencies {
            if let Some(dep) = self.targets.iter().find(|t| &t.id == dep_id) {
                self.visit_target(dep, visited, visiting, result)?;
            }
        }

        visiting.remove(&target.id);
        visited.insert(target.id.clone());
        result.push(target);

        Ok(())
    }
}

/// Error during build order calculation.
#[derive(Debug)]
pub enum BuildOrderError {
    /// Circular dependency detected
    CyclicDependency(String),
}

impl std::fmt::Display for BuildOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildOrderError::CyclicDependency(id) => {
                write!(f, "Circular dependency detected involving target '{}'", id)
            }
        }
    }
}

impl std::error::Error for BuildOrderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_kind_display() {
        assert_eq!(TargetKind::Sprite.to_string(), "sprite");
        assert_eq!(TargetKind::Atlas.to_string(), "atlas");
        assert_eq!(TargetKind::Animation.to_string(), "animation");
        assert_eq!(TargetKind::AnimationPreview.to_string(), "preview");
        assert_eq!(TargetKind::Export.to_string(), "export");
    }

    #[test]
    fn test_build_target_sprite() {
        let target = BuildTarget::sprite(
            "player".to_string(),
            PathBuf::from("src/player.pxl"),
            PathBuf::from("build/player.png"),
        );

        assert_eq!(target.id, "sprite:player");
        assert_eq!(target.kind, TargetKind::Sprite);
        assert_eq!(target.name, "player");
        assert_eq!(target.sources.len(), 1);
    }

    #[test]
    fn test_build_target_atlas() {
        let target = BuildTarget::atlas(
            "characters".to_string(),
            vec![
                PathBuf::from("src/player.pxl"),
                PathBuf::from("src/enemy.pxl"),
            ],
            PathBuf::from("build/characters.png"),
        );

        assert_eq!(target.id, "atlas:characters");
        assert_eq!(target.kind, TargetKind::Atlas);
        assert_eq!(target.sources.len(), 2);
    }

    #[test]
    fn test_build_target_matches_filter_exact() {
        let target = BuildTarget::atlas(
            "characters".to_string(),
            vec![],
            PathBuf::from("build/characters.png"),
        );

        assert!(target.matches_filter("atlas:characters"));
        assert!(!target.matches_filter("atlas:enemies"));
    }

    #[test]
    fn test_build_target_matches_filter_kind() {
        let target = BuildTarget::atlas(
            "characters".to_string(),
            vec![],
            PathBuf::from("build/characters.png"),
        );

        assert!(target.matches_filter("atlas"));
        assert!(!target.matches_filter("sprite"));
    }

    #[test]
    fn test_build_target_matches_filter_wildcard() {
        let target = BuildTarget::atlas(
            "characters".to_string(),
            vec![],
            PathBuf::from("build/characters.png"),
        );

        assert!(target.matches_filter("atlas:*"));
        assert!(target.matches_filter("*:characters"));
        assert!(!target.matches_filter("*:enemies"));
    }

    #[test]
    fn test_build_plan_add_target() {
        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::sprite(
            "player".to_string(),
            PathBuf::from("src/player.pxl"),
            PathBuf::from("build/player.png"),
        ));

        assert_eq!(plan.len(), 1);
        assert!(!plan.is_empty());
    }

    #[test]
    fn test_build_plan_filter() {
        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::atlas(
            "characters".to_string(),
            vec![],
            PathBuf::from("build/characters.png"),
        ));
        plan.add_target(BuildTarget::atlas(
            "environment".to_string(),
            vec![],
            PathBuf::from("build/environment.png"),
        ));
        plan.add_target(BuildTarget::sprite(
            "player".to_string(),
            PathBuf::from("src/player.pxl"),
            PathBuf::from("build/player.png"),
        ));

        let filtered = plan.filter(&["atlas".to_string()]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_build_plan_build_order_simple() {
        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::sprite(
            "a".to_string(),
            PathBuf::from("a.pxl"),
            PathBuf::from("a.png"),
        ));
        plan.add_target(BuildTarget::sprite(
            "b".to_string(),
            PathBuf::from("b.pxl"),
            PathBuf::from("b.png"),
        ));

        let order = plan.build_order().unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_build_plan_build_order_with_deps() {
        let mut plan = BuildPlan::new();
        plan.add_target(BuildTarget::animation(
            "walk".to_string(),
            PathBuf::from("walk.pxl"),
            PathBuf::from("walk.png"),
        ));
        plan.add_target(BuildTarget::animation_preview(
            "walk".to_string(),
            PathBuf::from("walk.pxl"),
            PathBuf::from("walk.gif"),
        ));

        let order = plan.build_order().unwrap();
        assert_eq!(order.len(), 2);
        // Animation should come before preview
        assert_eq!(order[0].id, "animation:walk");
        assert_eq!(order[1].id, "preview:walk");
    }
}
