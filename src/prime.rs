//! Primer content for AI context injection
//!
//! The `pxl prime` command prints format documentation to help AI systems
//! generate better pixelsrc content.

use std::fmt;
use std::str::FromStr;

/// Full primer content, embedded at compile time
pub const PRIMER_FULL: &str = include_str!("../docs/primer.md");

/// Brief primer content, embedded at compile time
pub const PRIMER_BRIEF: &str = include_str!("../docs/primer_brief.md");

/// Available primer sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimerSection {
    /// Format quick reference (object types, syntax)
    Format,
    /// Complete example with explanation
    Examples,
    /// Best practices and tips
    Tips,
    /// Full primer (default)
    Full,
}

impl fmt::Display for PrimerSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimerSection::Format => write!(f, "format"),
            PrimerSection::Examples => write!(f, "examples"),
            PrimerSection::Tips => write!(f, "tips"),
            PrimerSection::Full => write!(f, "full"),
        }
    }
}

impl FromStr for PrimerSection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "format" => Ok(PrimerSection::Format),
            "examples" => Ok(PrimerSection::Examples),
            "tips" => Ok(PrimerSection::Tips),
            "full" => Ok(PrimerSection::Full),
            _ => Err(format!("Unknown section '{}'. Available: format, examples, tips, full", s)),
        }
    }
}

/// Get primer content based on section and brief mode
pub fn get_primer(section: PrimerSection, brief: bool) -> &'static str {
    if brief {
        // Brief mode always returns the brief primer (no section splitting)
        return PRIMER_BRIEF;
    }

    match section {
        PrimerSection::Full => PRIMER_FULL,
        PrimerSection::Format => extract_section(PRIMER_FULL, "Format Quick Reference"),
        PrimerSection::Examples => extract_section(PRIMER_FULL, "Complete Example"),
        PrimerSection::Tips => extract_section(PRIMER_FULL, "Best Practices"),
    }
}

/// Extract a section from the primer by heading
fn extract_section(content: &'static str, heading: &str) -> &'static str {
    // Find the section by looking for ## heading
    let search = format!("## {}", heading);

    if let Some(start_idx) = content.find(&search) {
        // Find where this section ends (next ## heading or end of file)
        let section_content = &content[start_idx..];

        // Find the next ## heading (skip the current one)
        let after_heading = &section_content[3..]; // Skip "## "
        if let Some(next_section) = after_heading.find("\n## ") {
            // Return from start of heading to next heading
            &content[start_idx..start_idx + 3 + next_section]
        } else {
            // No more sections, return to end
            section_content
        }
    } else {
        // Section not found, return full content
        content
    }
}

/// List available sections
pub fn list_sections() -> &'static [&'static str] {
    &["format", "examples", "tips", "full"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primer_section_from_str() {
        assert_eq!(PrimerSection::from_str("format").unwrap(), PrimerSection::Format);
        assert_eq!(PrimerSection::from_str("FORMAT").unwrap(), PrimerSection::Format);
        assert_eq!(PrimerSection::from_str("examples").unwrap(), PrimerSection::Examples);
        assert_eq!(PrimerSection::from_str("tips").unwrap(), PrimerSection::Tips);
        assert_eq!(PrimerSection::from_str("full").unwrap(), PrimerSection::Full);
        assert!(PrimerSection::from_str("invalid").is_err());
    }

    #[test]
    fn test_primer_section_display() {
        assert_eq!(format!("{}", PrimerSection::Format), "format");
        assert_eq!(format!("{}", PrimerSection::Examples), "examples");
        assert_eq!(format!("{}", PrimerSection::Tips), "tips");
        assert_eq!(format!("{}", PrimerSection::Full), "full");
    }

    #[test]
    fn test_get_primer_full() {
        let content = get_primer(PrimerSection::Full, false);
        assert!(content.contains("# Pixelsrc Primer"));
        assert!(content.contains("## Format Quick Reference"));
    }

    #[test]
    fn test_get_primer_brief() {
        let content = get_primer(PrimerSection::Full, true);
        assert!(content.contains("# Pixelsrc Quick Reference"));
        // Brief version is shorter
        assert!(content.len() < 3000);
    }

    #[test]
    fn test_get_primer_format_section() {
        let content = get_primer(PrimerSection::Format, false);
        assert!(content.contains("## Format Quick Reference"));
        assert!(content.contains("Object Types"));
    }

    #[test]
    fn test_get_primer_examples_section() {
        let content = get_primer(PrimerSection::Examples, false);
        assert!(content.contains("## Complete Example"));
        assert!(content.contains("coin"));
    }

    #[test]
    fn test_get_primer_tips_section() {
        let content = get_primer(PrimerSection::Tips, false);
        assert!(content.contains("## Best Practices"));
        assert!(content.contains("DO"));
    }
}
