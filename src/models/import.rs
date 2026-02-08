//! Import declaration model for cross-file references.

use serde::{Deserialize, Serialize};

/// An import declaration that pulls items from other files into local scope.
///
/// # Variants
///
/// **Unfiltered import** — imports all items from a file:
/// ```json
/// {"type": "import", "from": "characters/hero/base"}
/// ```
///
/// **Selective import** — imports specific items by type:
/// ```json
/// {"type": "import", "from": "palettes/shared", "palettes": ["gameboy", "nes"]}
/// ```
///
/// **Aliased import** — creates a namespace prefix:
/// ```json
/// {"type": "import", "from": "characters/hero/base", "as": "hero"}
/// ```
///
/// **Directory import** — imports all files in a directory:
/// ```json
/// {"type": "import", "from": "characters/hero/"}
/// ```
///
/// **Relative import** — works without pxl.toml:
/// ```json
/// {"type": "import", "from": "../shared/colors"}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Import {
    /// Source path to import from (required).
    ///
    /// Can be:
    /// - Root-relative: `"characters/hero/base"` (requires pxl.toml)
    /// - Relative: `"./palettes/brand"` or `"../shared/colors"`
    /// - Directory: `"characters/hero/"` (trailing slash)
    /// - External: `"lospec-palettes/retro"` (dependency name prefix)
    pub from: String,

    /// Optional alias for the import, creating a namespace prefix.
    ///
    /// When set, imported items are accessed as `alias:item_name`.
    #[serde(rename = "as")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    /// Specific sprite names to import.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprites: Option<Vec<String>>,

    /// Specific palette names to import.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palettes: Option<Vec<String>>,

    /// Specific transform names to import.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,

    /// Specific animation names to import.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<Vec<String>>,
}

impl Import {
    /// Returns true if this is a directory import (path ends with `/`).
    pub fn is_directory_import(&self) -> bool {
        self.from.ends_with('/')
    }

    /// Returns true if this is a relative import (starts with `./` or `../`).
    pub fn is_relative(&self) -> bool {
        self.from.starts_with("./") || self.from.starts_with("../")
    }

    /// Returns true if this is a selective import (has any type filters).
    pub fn is_selective(&self) -> bool {
        self.sprites.is_some()
            || self.palettes.is_some()
            || self.transforms.is_some()
            || self.animations.is_some()
    }

    /// Returns true if this is an unfiltered import (no type filters, no alias).
    pub fn is_unfiltered(&self) -> bool {
        !self.is_selective() && self.alias.is_none()
    }

    /// Returns true if this import has an alias.
    pub fn is_aliased(&self) -> bool {
        self.alias.is_some()
    }

    /// Validate the import declaration.
    ///
    /// Returns a list of validation error messages. An empty list means valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.from.is_empty() {
            errors.push("import 'from' field cannot be empty".to_string());
        }

        // Validate alias doesn't contain reserved characters
        if let Some(alias) = &self.alias {
            if alias.contains(':') || alias.contains('/') {
                errors.push(format!("import alias '{}' cannot contain ':' or '/'", alias));
            }
            if alias.is_empty() {
                errors.push("import 'as' alias cannot be empty".to_string());
            }
        }

        // Validate imported names don't contain reserved characters
        let check_names = |names: &Option<Vec<String>>, kind: &str, errors: &mut Vec<String>| {
            if let Some(names) = names {
                for name in names {
                    if name.contains(':') || name.contains('/') {
                        errors.push(format!(
                            "imported {} name '{}' cannot contain ':' or '/'",
                            kind, name
                        ));
                    }
                }
            }
        };

        check_names(&self.sprites, "sprite", &mut errors);
        check_names(&self.palettes, "palette", &mut errors);
        check_names(&self.transforms, "transform", &mut errors);
        check_names(&self.animations, "animation", &mut errors);

        errors
    }
}
