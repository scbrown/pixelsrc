//! Validation command implementations (validate, agent-verify, analyze, fmt)

use std::path::PathBuf;
use std::process::ExitCode;

use crate::analyze::{collect_files, format_report_text, AnalysisReport};
use crate::fmt::format_pixelsrc;
use crate::lsp_agent_client::LspAgentClient;
use crate::validate::{Severity, Validator};

use super::{EXIT_ERROR, EXIT_INVALID_ARGS, EXIT_SUCCESS};

/// Execute the analyze command
pub fn run_analyze(
    files: &[PathBuf],
    dir: Option<&std::path::Path>,
    recursive: bool,
    format: &str,
    output: Option<&std::path::Path>,
) -> ExitCode {
    // Validate format
    if format != "text" && format != "json" {
        eprintln!("Error: --format must be 'text' or 'json'");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Collect files to analyze
    let file_list = match collect_files(files, dir, recursive) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::from(EXIT_ERROR);
        }
    };

    if file_list.is_empty() {
        eprintln!("Error: No files to analyze");
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    // Run analysis with progress indication
    let mut report = AnalysisReport::new();
    let total_files = file_list.len();
    let show_progress = total_files > 1 && output.is_some();

    for (i, path) in file_list.iter().enumerate() {
        if show_progress {
            eprint!("\rAnalyzing file {}/{}: {}", i + 1, total_files, path.display());
        }
        if let Err(e) = report.analyze_file(path) {
            report.files_failed += 1;
            report.failed_files.push((path.clone(), e));
        }
    }
    if show_progress {
        eprintln!(); // Clear progress line
    }

    // Format output
    let output_text = if format == "json" {
        // JSON output
        serde_json::json!({
            "files_analyzed": report.files_analyzed,
            "files_failed": report.files_failed,
            "total_sprites": report.total_sprites,
            "total_palettes": report.total_palettes,
            "total_compositions": report.total_compositions,
            "total_animations": report.total_animations,
            "total_variants": report.total_variants,
            "unique_tokens": report.token_counter.unique_count(),
            "total_token_occurrences": report.token_counter.total(),
            "top_tokens": report.token_counter.top_n(10).iter().map(|(t, c)| {
                serde_json::json!({
                    "token": t,
                    "count": c,
                    "percentage": report.token_counter.percentage(t)
                })
            }).collect::<Vec<_>>(),
            "co_occurrence": report.co_occurrence.top_n(10).iter().map(|((t1, t2), count)| {
                serde_json::json!({
                    "token1": t1,
                    "token2": t2,
                    "sprites": count
                })
            }).collect::<Vec<_>>(),
            "token_families": report.token_families().iter().take(10).map(|family| {
                serde_json::json!({
                    "prefix": family.prefix,
                    "tokens": family.tokens,
                    "total_count": family.total_count
                })
            }).collect::<Vec<_>>(),
            "avg_palette_size": report.avg_palette_size(),
        })
        .to_string()
    } else {
        format_report_text(&report)
    };

    // Write output
    if let Some(output_path) = output {
        if let Err(e) = std::fs::write(output_path, &output_text) {
            eprintln!("Error: Failed to write '{}': {}", output_path.display(), e);
            return ExitCode::from(EXIT_ERROR);
        }
        println!("Report written to: {}", output_path.display());
    } else {
        print!("{}", output_text);
    }

    ExitCode::from(EXIT_SUCCESS)
}

/// Execute the fmt command
pub fn run_fmt(files: &[PathBuf], check: bool, stdout_mode: bool) -> ExitCode {
    let mut needs_formatting = false;

    for file in files {
        // Read file content
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Cannot read '{}': {}", file.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        };

        // Format the content
        let formatted = match format_pixelsrc(&content) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: Cannot format '{}': {}", file.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        };

        if check {
            // Check mode: compare and report
            if content != formatted {
                eprintln!("{}: needs formatting", file.display());
                needs_formatting = true;
            }
        } else if stdout_mode {
            // Stdout mode: print formatted content
            print!("{}", formatted);
        } else {
            // In-place mode: write back to file
            if content != formatted {
                if let Err(e) = std::fs::write(file, &formatted) {
                    eprintln!("Error: Cannot write '{}': {}", file.display(), e);
                    return ExitCode::from(EXIT_ERROR);
                }
                eprintln!("{}: formatted", file.display());
            } else {
                eprintln!("{}: already formatted", file.display());
            }
        }
    }

    if check && needs_formatting {
        ExitCode::from(EXIT_ERROR)
    } else {
        ExitCode::from(EXIT_SUCCESS)
    }
}

/// Execute the validate command
pub fn run_validate(files: &[PathBuf], stdin: bool, strict: bool, json: bool) -> ExitCode {
    use std::io::{self, BufRead};

    let mut validator = Validator::new();

    if stdin {
        // Read from stdin
        let stdin_handle = io::stdin();
        for (line_idx, line_result) in stdin_handle.lock().lines().enumerate() {
            let line_number = line_idx + 1;
            match line_result {
                Ok(line) => validator.validate_line(line_number, &line),
                Err(e) => {
                    eprintln!("Error reading stdin at line {}: {}", line_number, e);
                    return ExitCode::from(EXIT_ERROR);
                }
            }
        }
    } else {
        // Validate files
        if files.is_empty() {
            eprintln!("Error: No files to validate");
            return ExitCode::from(EXIT_INVALID_ARGS);
        }

        for path in files {
            if !json {
                println!("Validating {}...", path.display());
            }
            if let Err(e) = validator.validate_file(path) {
                eprintln!("Error: Cannot read '{}': {}", path.display(), e);
                return ExitCode::from(EXIT_ERROR);
            }
        }
    }

    let issues = validator.into_issues();
    let error_count = issues.iter().filter(|i| matches!(i.severity, Severity::Error)).count();
    let warning_count = issues.iter().filter(|i| matches!(i.severity, Severity::Warning)).count();

    // Determine validity based on strict mode
    let has_failures = error_count > 0 || (strict && warning_count > 0);

    if json {
        // JSON output
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Error))
            .map(|i| {
                let mut obj = serde_json::json!({
                    "line": i.line,
                    "type": i.issue_type.to_string(),
                    "message": i.message,
                });
                if let Some(ref ctx) = i.context {
                    obj["context"] = serde_json::json!(ctx);
                }
                if let Some(ref sug) = i.suggestion {
                    obj["suggestion"] = serde_json::json!(sug);
                }
                obj
            })
            .collect();

        let warnings: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Warning))
            .map(|i| {
                let mut obj = serde_json::json!({
                    "line": i.line,
                    "type": i.issue_type.to_string(),
                    "message": i.message,
                });
                if let Some(ref ctx) = i.context {
                    obj["context"] = serde_json::json!(ctx);
                }
                if let Some(ref sug) = i.suggestion {
                    obj["suggestion"] = serde_json::json!(sug);
                }
                obj
            })
            .collect();

        let output = serde_json::json!({
            "valid": !has_failures,
            "errors": errors,
            "warnings": warnings,
        });

        println!("{}", serde_json::to_string_pretty(&output).expect("JSON value serialization"));
    } else {
        // Text output
        if issues.is_empty() {
            println!();
            println!("No issues found.");
        } else {
            println!();
            for issue in &issues {
                let severity_str = match issue.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARNING",
                };

                let mut msg = format!("Line {}: {} - {}", issue.line, severity_str, issue.message);

                if let Some(ref ctx) = issue.context {
                    msg.push_str(&format!(" ({})", ctx));
                }
                if let Some(ref sug) = issue.suggestion {
                    msg.push_str(&format!(" ({})", sug));
                }

                eprintln!("{}", msg);
            }

            println!();
            match (error_count, warning_count) {
                (0, w) => println!("Found {} warning{}.", w, if w == 1 { "" } else { "s" }),
                (e, 0) => println!("Found {} error{}.", e, if e == 1 { "" } else { "s" }),
                (e, w) => println!(
                    "Found {} error{}, {} warning{}.",
                    e,
                    if e == 1 { "" } else { "s" },
                    w,
                    if w == 1 { "" } else { "s" }
                ),
            }

            if !strict && warning_count > 0 && error_count == 0 {
                println!("Hint: Run with --strict to treat warnings as errors.");
            }
        }
    }

    if has_failures {
        ExitCode::from(EXIT_ERROR)
    } else {
        ExitCode::from(EXIT_SUCCESS)
    }
}

/// Execute the agent-verify command
pub fn run_agent_verify(
    stdin: bool,
    content: Option<&str>,
    strict: bool,
    grid_info: bool,
    suggest_tokens: bool,
    resolve_colors_flag: bool,
    analyze_timing_flag: bool,
) -> ExitCode {
    use std::io::{self, Read};

    // Get content from --content arg or stdin
    let content_string: String = if let Some(c) = content {
        c.to_string()
    } else if stdin || content.is_none() {
        // Read from stdin by default
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            let error_json = serde_json::json!({
                "error": format!("Failed to read from stdin: {}", e)
            });
            println!("{}", serde_json::to_string_pretty(&error_json).expect("JSON value serialization"));
            return ExitCode::from(EXIT_ERROR);
        }
        buffer
    } else {
        let error_json = serde_json::json!({
            "error": "No content provided. Use --content or provide input via stdin."
        });
        println!("{}", serde_json::to_string_pretty(&error_json).expect("JSON value serialization"));
        return ExitCode::from(EXIT_INVALID_ARGS);
    };

    // Create client with appropriate strictness
    let client = if strict { LspAgentClient::strict() } else { LspAgentClient::new() };

    // Build the result object
    let mut result = serde_json::Map::new();

    // Always include verification result
    let verification = client.verify_content(&content_string);
    result.insert("valid".to_string(), serde_json::json!(verification.valid));
    result.insert("error_count".to_string(), serde_json::json!(verification.error_count));
    result.insert("warning_count".to_string(), serde_json::json!(verification.warning_count));

    // Convert errors to JSON
    let errors: Vec<serde_json::Value> = verification
        .errors
        .iter()
        .map(|d| {
            let mut obj = serde_json::json!({
                "line": d.line,
                "type": d.issue_type,
                "message": d.message,
            });
            if let Some(ref ctx) = d.context {
                obj["context"] = serde_json::json!(ctx);
            }
            if let Some(ref sug) = d.suggestion {
                obj["suggestion"] = serde_json::json!(sug);
            }
            obj
        })
        .collect();
    result.insert("errors".to_string(), serde_json::json!(errors));

    // Convert warnings to JSON
    let warnings: Vec<serde_json::Value> = verification
        .warnings
        .iter()
        .map(|d| {
            let mut obj = serde_json::json!({
                "line": d.line,
                "type": d.issue_type,
                "message": d.message,
            });
            if let Some(ref ctx) = d.context {
                obj["context"] = serde_json::json!(ctx);
            }
            if let Some(ref sug) = d.suggestion {
                obj["suggestion"] = serde_json::json!(sug);
            }
            obj
        })
        .collect();
    result.insert("warnings".to_string(), serde_json::json!(warnings));

    // Optional: sprite dimension info
    if grid_info {
        // Extract size info from sprites (supports both legacy grid and regions format)
        let grid_info_vec: Vec<serde_json::Value> = content_string
            .lines()
            .filter_map(|line| {
                let obj: serde_json::Value = serde_json::from_str(line).ok()?;
                let obj = obj.as_object()?;
                if obj.get("type")?.as_str()? != "sprite" {
                    return None;
                }
                let name = obj.get("name")?.as_str()?;

                // Get size from size field (primary source for both grid and regions format)
                let size = obj.get("size").and_then(|s| s.as_array())?;
                let width = size.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let height = size.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                // Count regions if using regions format
                let region_count =
                    obj.get("regions").and_then(|r| r.as_object()).map(|r| r.len()).unwrap_or(0);

                Some(serde_json::json!({
                    "name": name,
                    "size": [width, height],
                    "region_count": region_count,
                }))
            })
            .collect();

        result.insert("grid_info".to_string(), serde_json::json!(grid_info_vec));
    }

    // Optional: token suggestions
    if suggest_tokens {
        let completions = client.get_completions(&content_string, 1, 0);
        let tokens: Vec<serde_json::Value> = completions
            .items
            .iter()
            .map(|c| {
                let mut obj = serde_json::json!({
                    "token": c.label,
                });
                if let Some(ref detail) = c.detail {
                    obj["color"] = serde_json::json!(detail);
                }
                obj
            })
            .collect();
        result.insert("available_tokens".to_string(), serde_json::json!(tokens));
    }

    // Optional: resolve colors
    if resolve_colors_flag {
        let color_result = client.resolve_colors(&content_string);
        let resolved: Vec<serde_json::Value> = color_result
            .colors
            .iter()
            .map(|c| {
                serde_json::json!({
                    "token": c.token,
                    "original": c.original,
                    "resolved": c.resolved,
                    "palette": c.palette,
                })
            })
            .collect();
        result.insert("resolved_colors".to_string(), serde_json::json!(resolved));

        if !color_result.errors.is_empty() {
            result.insert(
                "color_resolution_errors".to_string(),
                serde_json::json!(color_result.errors),
            );
        }
    }

    // Optional: analyze timing
    if analyze_timing_flag {
        let timing_result = client.analyze_timing(&content_string);
        let analysis: Vec<serde_json::Value> = timing_result
            .animations
            .iter()
            .map(|t| {
                let mut obj = serde_json::json!({
                    "animation": t.animation,
                    "timing_function": t.timing_function,
                    "description": t.description,
                    "curve_type": t.curve_type,
                });
                if let Some(ref curve) = t.ascii_curve {
                    obj["ascii_curve"] = serde_json::json!(curve);
                }
                obj
            })
            .collect();
        result.insert("timing_analysis".to_string(), serde_json::json!(analysis));
    }

    // Output JSON result
    println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(result)).expect("JSON value serialization"));

    if verification.valid {
        ExitCode::from(EXIT_SUCCESS)
    } else {
        ExitCode::from(EXIT_ERROR)
    }
}
