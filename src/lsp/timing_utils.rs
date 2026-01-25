//! Timing function utilities for LSP hover.

use crate::motion::{ease, parse_timing_function, Interpolation, StepPosition};
use serde_json::Value;

use super::types::TimingFunctionInfo;

/// Render an ASCII visualization of an easing curve
///
/// Creates a simple ASCII graph showing the easing function's shape.
pub fn render_easing_curve(interpolation: &Interpolation, width: usize, height: usize) -> String {
    let mut grid = vec![vec![' '; width]; height];

    // Sample the easing function
    let samples: Vec<f64> = (0..=width)
        .map(|i| {
            let t = i as f64 / width as f64;
            ease(t, interpolation)
        })
        .collect();

    // Find min/max for scaling (handle overshoot)
    let min_val = samples.iter().cloned().fold(f64::INFINITY, f64::min).min(0.0);
    let max_val = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1.0);
    let range = max_val - min_val;

    // Plot the curve
    for (x, &value) in samples.iter().enumerate().take(width) {
        // Scale value to grid height
        let normalized = if range > 0.0 { (value - min_val) / range } else { 0.5 };
        let y = ((1.0 - normalized) * (height - 1) as f64).round() as usize;
        let y = y.min(height - 1);
        if x < width {
            grid[y][x] = '█';
        }
    }

    // Build the output with axis labels
    let mut output = String::new();

    // Top label (1.0 or max)
    let top_label = if max_val > 1.0 { format!("{:.1}", max_val) } else { "1.0".to_string() };
    output.push_str(&format!("{:>4}│", top_label));
    output.push_str(&grid[0].iter().collect::<String>());
    output.push('\n');

    // Middle rows
    for row in grid.iter().skip(1).take(height - 2) {
        output.push_str("    │");
        output.push_str(&row.iter().collect::<String>());
        output.push('\n');
    }

    // Bottom row with 0.0 label
    let bottom_label = if min_val < 0.0 { format!("{:.1}", min_val) } else { "0.0".to_string() };
    output.push_str(&format!("{:>4}│", bottom_label));
    output.push_str(&grid[height - 1].iter().collect::<String>());
    output.push('\n');

    // X-axis
    output.push_str("    └");
    output.push_str(&"─".repeat(width));
    output.push('\n');
    output.push_str("     0");
    output.push_str(&" ".repeat(width - 3));
    output.push_str("→ 1");

    output
}

/// Get a human-readable description of an interpolation type
pub fn describe_interpolation(interpolation: &Interpolation) -> &'static str {
    match interpolation {
        Interpolation::Linear => "Constant speed (no easing)",
        Interpolation::EaseIn => "Slow start, fast end (acceleration)",
        Interpolation::EaseOut => "Fast start, slow end (deceleration)",
        Interpolation::EaseInOut => "Smooth S-curve (slow start and end)",
        Interpolation::Bounce => "Overshoot and settle back",
        Interpolation::Elastic => "Spring-like oscillation",
        Interpolation::Bezier { .. } => "Custom cubic bezier curve",
        Interpolation::Steps { .. } => "Discrete step function",
    }
}

/// Get the CSS-canonical form of an interpolation
pub fn interpolation_to_css(interpolation: &Interpolation) -> String {
    match interpolation {
        Interpolation::Linear => "linear".to_string(),
        Interpolation::EaseIn => "ease-in".to_string(),
        Interpolation::EaseOut => "ease-out".to_string(),
        Interpolation::EaseInOut => "ease-in-out".to_string(),
        Interpolation::Bounce => "bounce".to_string(),
        Interpolation::Elastic => "elastic".to_string(),
        Interpolation::Bezier { p1, p2 } => {
            format!("cubic-bezier({}, {}, {}, {})", p1.0, p1.1, p2.0, p2.1)
        }
        Interpolation::Steps { count, position } => match position {
            StepPosition::JumpEnd => {
                if *count == 1 {
                    "step-end".to_string()
                } else {
                    format!("steps({})", count)
                }
            }
            StepPosition::JumpStart => {
                if *count == 1 {
                    "step-start".to_string()
                } else {
                    format!("steps({}, jump-start)", count)
                }
            }
            _ => format!("steps({}, {})", count, position),
        },
    }
}

/// Parse timing function context from a JSON line at cursor position
///
/// Returns TimingFunctionInfo if the cursor is within a timing_function value.
pub fn parse_timing_function_context(line: &str, char_pos: u32) -> Option<TimingFunctionInfo> {
    // Parse the JSON line
    let obj: Value = serde_json::from_str(line).ok()?;
    let obj = obj.as_object()?;

    // Check if this is an animation type
    let obj_type = obj.get("type")?.as_str()?;
    if obj_type != "animation" {
        return None;
    }

    // Look for timing_function field
    let timing_str = obj.get("timing_function")?.as_str()?;

    // Find the timing_function key position in the raw JSON
    let key_pos = line.find("\"timing_function\"")?;

    // Find the colon after the key
    let after_key = &line[key_pos..];
    let colon_offset = after_key.find(':')?;

    // Find the opening quote of the value
    let after_colon = &after_key[colon_offset..];
    let quote_offset = after_colon.find('"')?;
    let value_start = key_pos + colon_offset + quote_offset + 1;

    // Find the closing quote
    let value_end = value_start + timing_str.len();

    // Check if cursor is within the value
    let char_pos = char_pos as usize;
    if char_pos < value_start || char_pos > value_end {
        return None;
    }

    // Parse the timing function
    let interpolation = parse_timing_function(timing_str).ok()?;

    Some(TimingFunctionInfo { function_str: timing_str.to_string(), interpolation })
}
