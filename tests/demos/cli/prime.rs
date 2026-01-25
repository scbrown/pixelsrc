//! Prime CLI Demos
//!
//! Demo tests for the `pxl prime` command that outputs AI-context primers.

use pixelsrc::prime::{get_primer, list_sections, PrimerSection};

/// @demo cli/prime#full
/// @title Full Primer Output
/// @description Output the complete AI primer for context loading.
#[test]
fn test_prime_full() {
    let content = get_primer(PrimerSection::Full, false);

    // Full primer contains all major sections
    assert!(content.contains("# Pixelsrc Primer"), "Should have primer title");
    assert!(content.contains("## Format Quick Reference"), "Should have format section");
    assert!(content.contains("## Complete Example"), "Should have examples section");
    assert!(content.contains("## Best Practices"), "Should have tips section");
}

/// @demo cli/prime#brief
/// @title Brief Primer Output
/// @description Output a compact primer for limited context windows.
#[test]
fn test_prime_brief() {
    let content = get_primer(PrimerSection::Full, true);

    // Brief version is more compact
    assert!(content.contains("# Pixelsrc Quick Reference"), "Should have brief title");
    assert!(content.len() < 3500, "Brief version should be under 3500 chars");
}

/// @demo cli/prime#format_section
/// @title Format Section Only
/// @description Output just the format reference section.
#[test]
fn test_prime_format_section() {
    let content = get_primer(PrimerSection::Format, false);

    // Format section has type information
    assert!(content.contains("## Format Quick Reference"), "Should have format heading");
    assert!(content.contains("Object Types"), "Should describe object types");
}

/// @demo cli/prime#examples_section
/// @title Examples Section Only
/// @description Output just the examples section with working code.
#[test]
fn test_prime_examples_section() {
    let content = get_primer(PrimerSection::Examples, false);

    // Examples section has complete code samples
    assert!(content.contains("## Complete Example"), "Should have examples heading");
    assert!(content.contains("coin"), "Should include coin example");
}

/// @demo cli/prime#tips_section
/// @title Tips Section Only
/// @description Output just the best practices section.
#[test]
fn test_prime_tips_section() {
    let content = get_primer(PrimerSection::Tips, false);

    // Tips section has DOs and DON'Ts
    assert!(content.contains("## Best Practices"), "Should have tips heading");
    assert!(content.contains("DO"), "Should have DO recommendations");
}

/// @demo cli/prime#list_sections
/// @title List Available Sections
/// @description Show all available primer sections.
#[test]
fn test_prime_list_sections() {
    let sections = list_sections();

    // All expected sections are available
    assert!(sections.contains(&"format"), "Should have format section");
    assert!(sections.contains(&"examples"), "Should have examples section");
    assert!(sections.contains(&"tips"), "Should have tips section");
    assert!(sections.contains(&"full"), "Should have full section");
    assert_eq!(sections.len(), 4, "Should have exactly 4 sections");
}
