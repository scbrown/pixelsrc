//! Explain, diff, and suggest command implementations

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitCode;

use crate::config::loader::find_config_from;
use crate::diff::{diff_files, format_diff};
use crate::explain::{explain_object, format_explanation, resolve_palette_colors, Explanation};
use crate::models::TtpObject;
use crate::parser::parse_stream;
use crate::suggest::{format_suggestion, suggest, Suggester, SuggestionFix, SuggestionType};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the explain command
pub fn run_explain(input: &PathBuf, name_filter: Option<&str>, json: bool) -> ExitCode {
    // Open input file
    let file = match File::open(input) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Cannot open input file '{}': {}", input.display(), e);
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    // Parse JSONL stream
    let reader = BufReader::new(file);
    let parse_result = parse_stream(reader);

    if parse_result.objects.is_empty() {
        eprintln!("Error: No objects found in input file");
        return ExitCode::from(EXIT_ERROR);
    }

    // Build palette lookup for color resolution
    // Map palette name -> (token -> color)
    let mut known_palettes: HashMap<String, HashMap<String, String>> = HashMap::new();
    for obj in &parse_result.objects {
        if let TtpObject::Palette(palette) = obj {
            known_palettes.insert(palette.name.clone(), palette.colors.clone());
        }
    }

    // Explain each object
    let mut explanations: Vec<Explanation> = Vec::new();

    for obj in &parse_result.objects {
        let obj_name = match obj {
            TtpObject::Palette(p) => &p.name,
            TtpObject::Sprite(s) => &s.name,
            TtpObject::Composition(c) => &c.name,
            TtpObject::Animation(a) => &a.name,
            TtpObject::Variant(v) => &v.name,
            TtpObject::Particle(p) => &p.name,
            TtpObject::Transform(t) => &t.name,
            TtpObject::StateRules(sr) => &sr.name,
            TtpObject::Import(i) => &i.from,
        };

        // Apply name filter if specified
        if let Some(filter) = name_filter {
            if obj_name != filter {
                continue;
            }
        }

        // Resolve palette colors for sprites if needed
        let resolved_colors: Option<HashMap<String, String>> =
            if let TtpObject::Sprite(sprite) = obj {
                resolve_palette_colors(&sprite.palette, &known_palettes)
            } else {
                None
            };

        let exp = explain_object(obj, resolved_colors.as_ref());
        explanations.push(exp);
    }

    if explanations.is_empty() {
        if let Some(filter) = name_filter {
            eprintln!("Error: No object named '{}' found in input", filter);
            let all_names: Vec<String> = parse_result
                .objects
                .iter()
                .map(|obj| match obj {
                    TtpObject::Palette(p) => p.name.clone(),
                    TtpObject::Sprite(s) => s.name.clone(),
                    TtpObject::Composition(c) => c.name.clone(),
                    TtpObject::Animation(a) => a.name.clone(),
                    TtpObject::Variant(v) => v.name.clone(),
                    TtpObject::Particle(p) => p.name.clone(),
                    TtpObject::Transform(t) => t.name.clone(),
                    TtpObject::StateRules(sr) => sr.name.clone(),
                    TtpObject::Import(i) => i.from.clone(),
                })
                .collect();
            let name_refs: Vec<&str> = all_names.iter().map(|s| s.as_str()).collect();
            if let Some(suggestion) = format_suggestion(&suggest(filter, &name_refs, 3)) {
                eprintln!("{}", suggestion);
            }
        }
        return ExitCode::from(EXIT_ERROR);
    }

    // Output
    if json {
        // JSON output
        let json_explanations: Vec<serde_json::Value> = explanations
            .iter()
            .map(|exp| match exp {
                Explanation::Sprite(s) => serde_json::json!({
                    "type": "sprite",
                    "name": s.name,
                    "width": s.width,
                    "height": s.height,
                    "total_cells": s.total_cells,
                    "palette_ref": s.palette_ref,
                    "tokens": s.tokens.iter().map(|t| serde_json::json!({
                        "token": t.token,
                        "count": t.count,
                        "percentage": t.percentage,
                        "color": t.color,
                        "color_name": t.color_name,
                    })).collect::<Vec<_>>(),
                    "transparent_count": s.transparent_count,
                    "transparency_ratio": s.transparency_ratio,
                    "consistent_rows": s.consistent_rows,
                    "issues": s.issues,
                }),
                Explanation::Palette(p) => serde_json::json!({
                    "type": "palette",
                    "name": p.name,
                    "color_count": p.color_count,
                    "colors": p.colors.iter().map(|(token, hex, name)| serde_json::json!({
                        "token": token,
                        "color": hex,
                        "color_name": name,
                    })).collect::<Vec<_>>(),
                    "is_builtin": p.is_builtin,
                }),
                Explanation::Animation(a) => serde_json::json!({
                    "type": "animation",
                    "name": a.name,
                    "frames": a.frames,
                    "frame_count": a.frame_count,
                    "duration_ms": a.duration_ms,
                    "loops": a.loops,
                }),
                Explanation::Composition(c) => serde_json::json!({
                    "type": "composition",
                    "name": c.name,
                    "base": c.base,
                    "size": c.size,
                    "cell_size": c.cell_size,
                    "sprite_count": c.sprite_count,
                    "layer_count": c.layer_count,
                }),
                Explanation::Variant(v) => serde_json::json!({
                    "type": "variant",
                    "name": v.name,
                    "base": v.base,
                    "override_count": v.override_count,
                    "overrides": v.overrides.iter().map(|(token, color)| serde_json::json!({
                        "token": token,
                        "color": color,
                    })).collect::<Vec<_>>(),
                }),
                Explanation::Particle(p) => serde_json::json!({
                    "type": "particle",
                    "name": p.name,
                    "sprite": p.sprite,
                    "rate": p.rate,
                    "lifetime": p.lifetime,
                    "has_gravity": p.has_gravity,
                    "has_fade": p.has_fade,
                }),
                Explanation::Transform(t) => serde_json::json!({
                    "type": "transform",
                    "name": t.name,
                    "is_parameterized": t.is_parameterized,
                    "params": t.params,
                    "generates_animation": t.generates_animation,
                    "frame_count": t.frame_count,
                    "transform_type": t.transform_type,
                }),
                Explanation::StateRules(sr) => serde_json::json!({
                    "type": "state-rules",
                    "name": sr.name,
                    "rule_count": sr.rule_count,
                    "selectors": sr.selectors,
                }),
                Explanation::Import(i) => serde_json::json!({
                    "type": "import",
                    "from": i.from,
                    "is_directory": i.is_directory,
                    "is_relative": i.is_relative,
                    "alias": i.alias,
                    "imported_types": i.imported_types,
                }),
            })
            .collect();

        let output = if json_explanations.len() == 1 {
            serde_json::to_string_pretty(&json_explanations[0]).expect("JSON value serialization")
        } else {
            serde_json::to_string_pretty(&json_explanations).expect("JSON value serialization")
        };
        println!("{}", output);
    } else {
        // Text output
        for (i, exp) in explanations.iter().enumerate() {
            if i > 0 {
                println!("\n{}", "=".repeat(40));
                println!();
            }
            print!("{}", format_explanation(exp));
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the diff command
pub fn run_diff(file_a: &PathBuf, file_b: &PathBuf, sprite: Option<&str>, json: bool) -> ExitCode {
    // Get display names for the files
    let file_a_display = file_a.display().to_string();
    let file_b_display = file_b.display().to_string();

    // Compare the files
    let diffs = match diff_files(file_a, file_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    // Filter by sprite name if specified
    let filtered_diffs: Vec<_> = if let Some(name) = sprite {
        diffs.into_iter().filter(|(n, _)| n == name).collect()
    } else {
        diffs
    };

    if filtered_diffs.is_empty() {
        if sprite.is_some() {
            eprintln!(
                "Error: Sprite '{}' not found in either file",
                sprite.expect("sprite is Some in this branch")
            );
            return ExitCode::from(EXIT_ERROR);
        }
        println!("No sprites found to compare.");
        return ExitCode::from(EXIT_SUCCESS);
    }

    if json {
        // JSON output
        let output: Vec<_> = filtered_diffs
            .iter()
            .map(|(name, diff)| {
                let mut obj = serde_json::json!({
                    "sprite": name,
                    "summary": diff.summary,
                });

                if let Some(ref dim) = diff.dimension_change {
                    obj["dimension_change"] = serde_json::json!({
                        "old": [dim.old.0, dim.old.1],
                        "new": [dim.new.0, dim.new.1],
                    });
                }

                if !diff.palette_changes.is_empty() {
                    let palette_changes: Vec<_> = diff
                        .palette_changes
                        .iter()
                        .map(|c| match c {
                            crate::diff::PaletteChange::Added { token, color } => {
                                serde_json::json!({
                                    "type": "added",
                                    "token": token,
                                    "color": color,
                                })
                            }
                            crate::diff::PaletteChange::Removed { token } => {
                                serde_json::json!({
                                    "type": "removed",
                                    "token": token,
                                })
                            }
                            crate::diff::PaletteChange::Changed { token, old_color, new_color } => {
                                serde_json::json!({
                                    "type": "changed",
                                    "token": token,
                                    "old_color": old_color,
                                    "new_color": new_color,
                                })
                            }
                        })
                        .collect();
                    obj["palette_changes"] = serde_json::json!(palette_changes);
                }

                obj
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output).expect("JSON value serialization"));
    } else {
        // Text output
        for (i, (name, diff)) in filtered_diffs.iter().enumerate() {
            if i > 0 {
                println!();
                println!("---");
                println!();
            }
            println!("{}", format_diff(name, diff, &file_a_display, &file_b_display));
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the suggest command
pub fn run_suggest(files: &[PathBuf], stdin: bool, json: bool, only: Option<&str>) -> ExitCode {
    use std::io::{self, BufReader};

    // Parse the --only filter
    let type_filter: Option<SuggestionType> = match only {
        Some("token") => Some(SuggestionType::MissingToken),
        Some("include") => Some(SuggestionType::IncludeToImport),
        Some("import") => Some(SuggestionType::AddExplicitImport),
        Some(other) => {
            eprintln!(
                "Error: Unknown suggestion type '{}'. Use 'token', 'include', or 'import'.",
                other
            );
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        None => None,
    };

    let mut suggester = Suggester::new();

    if stdin {
        // Read from stdin
        let stdin_handle = io::stdin();
        if let Err(e) = suggester.analyze_reader(stdin_handle.lock()) {
            eprintln!("Error reading stdin: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    } else {
        // Analyze files
        if files.is_empty() {
            eprintln!("Error: No files to analyze");
            return ExitCode::from(EXIT_INVALID_ARGS);
        }

        for path in files {
            if !json {
                println!("Analyzing {}...", path.display());
            }
            let file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: Cannot open '{}': {}", path.display(), e);
                    return ExitCode::from(EXIT_ERROR);
                }
            };
            if let Err(e) = suggester.analyze_reader(BufReader::new(file)) {
                eprintln!("Error reading '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        }
    }

    // Try project-aware suggestions (explicit import recommendations)
    if !stdin {
        if let Some(first_file) = files.first() {
            let search_dir = first_file
                .canonicalize()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

            if let Some(config_path) = find_config_from(search_dir) {
                if let Some(project_root) = config_path.parent() {
                    if let Ok(config) = crate::config::loader::load_config(Some(&config_path)) {
                        let src_root = project_root.join(&config.project.src);
                        if src_root.exists() {
                            let mut registry = crate::build::project_registry::ProjectRegistry::new(
                                config.project.name.clone(),
                                src_root,
                            );
                            if registry.load_all(false).is_ok() {
                                for path in files {
                                    if let Ok(content) = std::fs::read_to_string(path) {
                                        suggester
                                            .suggest_explicit_imports(&registry, path, &content);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let report = suggester.into_report();

    // Apply type filter if specified
    let suggestions: Vec<_> = if let Some(filter_type) = type_filter {
        report.filter_by_type(filter_type).into_iter().cloned().collect()
    } else {
        report.suggestions.clone()
    };

    if json {
        // JSON output
        let output = serde_json::json!({
            "sprites_analyzed": report.sprites_analyzed,
            "suggestion_count": suggestions.len(),
            "suggestions": suggestions,
        });
        println!("{}", serde_json::to_string_pretty(&output).expect("JSON value serialization"));
    } else {
        // Text output
        if suggestions.is_empty() {
            println!();
            println!("No suggestions found.");
            println!("Analyzed {} sprite(s).", report.sprites_analyzed);
        } else {
            println!();
            println!(
                "Found {} suggestion(s) in {} sprite(s):",
                suggestions.len(),
                report.sprites_analyzed
            );
            println!();

            for suggestion in &suggestions {
                println!(
                    "Line {}: [{}] {}",
                    suggestion.line, suggestion.suggestion_type, suggestion.sprite
                );
                println!("  {}", suggestion.message);

                // Show fix details
                match &suggestion.fix {
                    SuggestionFix::ReplaceToken { from, to } => {
                        println!("  Fix: Replace {} with {}", from, to);
                    }
                    SuggestionFix::AddToPalette { token, suggested_color } => {
                        println!("  Fix: Add \"{}\": \"{}\" to palette", token, suggested_color);
                    }
                    SuggestionFix::UseImport { include_ref, import_json } => {
                        println!("  Replace: {}", include_ref);
                        println!("  With:    {}", import_json);
                    }
                    SuggestionFix::AddImport { import_json } => {
                        println!("  Add: {}", import_json);
                    }
                }
                println!();
            }

            // Summary by type
            let token_count = suggestions
                .iter()
                .filter(|s| s.suggestion_type == SuggestionType::MissingToken)
                .count();
            let include_count = suggestions
                .iter()
                .filter(|s| s.suggestion_type == SuggestionType::IncludeToImport)
                .count();
            let import_count = suggestions
                .iter()
                .filter(|s| s.suggestion_type == SuggestionType::AddExplicitImport)
                .count();

            let mut summary_parts = Vec::new();
            if token_count > 0 {
                summary_parts.push(format!("{} missing token(s)", token_count));
            }
            if include_count > 0 {
                summary_parts.push(format!("{} @include: migration(s)", include_count));
            }
            if import_count > 0 {
                summary_parts.push(format!("{} explicit import(s)", import_count));
            }
            if !summary_parts.is_empty() {
                println!("Summary: {}", summary_parts.join(", "));
            }
        }
    }

    ExitCode::from(EXIT_SUCCESS)
}
