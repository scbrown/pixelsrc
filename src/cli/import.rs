//! Import command implementation

use std::path::PathBuf;
use std::process::ExitCode;

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the import command
pub fn run_import(
    input: &PathBuf,
    output: Option<&std::path::Path>,
    max_colors: usize,
    sprite_name: Option<&str>,
    analyze: bool,
    confidence: f64,
    hints: bool,
    shapes: bool,
) -> ExitCode {
    // Validate max_colors
    if !(2..=256).contains(&max_colors) {
        eprintln!("Error: --max-colors must be between 2 and 256");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Validate confidence threshold
    if !(0.0..=1.0).contains(&confidence) {
        eprintln!("Error: --confidence must be between 0.0 and 1.0");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Derive sprite name from filename if not provided
    let name = sprite_name
        .map(String::from)
        .unwrap_or_else(|| input.file_stem().unwrap_or_default().to_string_lossy().to_string());

    // Import the PNG with analysis options
    let options = crate::import::ImportOptions {
        analyze,
        confidence_threshold: confidence,
        hints,
        extract_shapes: shapes,
        half_sprite: false, // TODO: Add CLI flag when needed
        dither_handling: crate::import::DitherHandling::Keep, // TODO: Add CLI flag when needed
        detect_upscale: analyze, // Enable upscale detection when analysis is on
        detect_outlines: analyze, // Enable outline detection when analysis is on
    };

    let result = match crate::import::import_png_with_options(input, &name, max_colors, &options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Generate output path
    let output_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        // Use .pxl extension if analysis is enabled, otherwise .jsonl
        let ext = if analyze { "pxl" } else { "jsonl" };
        input.parent().unwrap_or(std::path::Path::new(".")).join(format!("{}.{}", stem, ext))
    });

    // Write output (JSONL for legacy, structured for analysis)
    let output_content = if analyze { result.to_structured_jsonl() } else { result.to_jsonl() };

    if let Err(e) = std::fs::write(&output_path, &output_content) {
        eprintln!("Error: Failed to write '{}': {}", output_path.display(), e);
        return ExitCode::from(EXIT_ERROR);
    }

    // Print summary
    println!(
        "Imported: {} ({}x{}, {} colors)",
        output_path.display(),
        result.width,
        result.height,
        result.palette.len()
    );

    // Print analysis results if enabled
    if analyze {
        if let Some(ref analysis) = result.analysis {
            if !analysis.roles.is_empty() {
                println!("  Roles inferred: {}", analysis.roles.len());
            }
            if !analysis.relationships.is_empty() {
                println!("  Relationships: {}", analysis.relationships.len());
            }
            if let Some(ref symmetry) = analysis.symmetry {
                println!("  Symmetry: {:?}", symmetry);
            }
        }
    }

    // Print hints if requested
    if hints {
        if let Some(ref analysis) = result.analysis {
            if !analysis.naming_hints.is_empty() {
                println!("  Token naming hints:");
                for hint in &analysis.naming_hints {
                    println!("    {}: {}", hint.token, hint.suggested_name);
                }
            }
        }
    }

    // Print shape extraction results
    if shapes {
        if let Some(ref structured) = result.structured_regions {
            let mut rects = 0;
            let mut polys = 0;
            let mut points = 0;
            for region in structured.values() {
                match region {
                    crate::import::StructuredRegion::Rect(_) => rects += 1,
                    crate::import::StructuredRegion::Polygon(_) => polys += 1,
                    crate::import::StructuredRegion::Points(_) => points += 1,
                    crate::import::StructuredRegion::Union(sub) => {
                        for s in sub {
                            match s {
                                crate::import::StructuredRegion::Rect(_) => rects += 1,
                                crate::import::StructuredRegion::Polygon(_) => polys += 1,
                                crate::import::StructuredRegion::Points(_) => points += 1,
                                _ => {}
                            }
                        }
                    }
                }
            }
            println!(
                "  Shapes extracted: {} rects, {} polygons, {} point arrays",
                rects, polys, points
            );
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}
