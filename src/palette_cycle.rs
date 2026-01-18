//! Palette cycling animation support
//!
//! Palette cycling is a classic pixel art animation technique where colors
//! rotate through a set of palette tokens, creating effects like shimmering
//! water, flickering fire, or pulsing energy without needing multiple sprite frames.

use crate::models::{Animation, Duration, PaletteCycle, Sprite};
use crate::renderer::render_sprite;
use image::RgbaImage;
use std::collections::HashMap;

/// Apply a palette cycle rotation step to a palette.
///
/// Given a cycle with tokens [A, B, C] and step 1:
/// - A gets B's original color
/// - B gets C's original color
/// - C gets A's original color
///
/// Returns a new palette with the rotated colors.
pub fn apply_cycle_step(
    original_palette: &HashMap<String, String>,
    cycle: &PaletteCycle,
    step: usize,
) -> HashMap<String, String> {
    let mut result = original_palette.clone();
    let tokens = &cycle.tokens;
    let len = tokens.len();

    if len == 0 {
        return result;
    }

    // Collect original colors for tokens in the cycle
    let original_colors: Vec<Option<String>> = tokens
        .iter()
        .map(|token| original_palette.get(token).cloned())
        .collect();

    // Apply rotation: token at index i gets color from index (i + step) % len
    for (i, token) in tokens.iter().enumerate() {
        let source_idx = (i + step) % len;
        if let Some(ref color) = original_colors[source_idx] {
            result.insert(token.clone(), color.clone());
        }
    }

    result
}

/// Apply multiple palette cycles simultaneously.
///
/// When an animation has multiple independent cycles (e.g., water and fire),
/// this applies all cycle rotations at the given step.
pub fn apply_cycles_step(
    original_palette: &HashMap<String, String>,
    cycles: &[PaletteCycle],
    step: usize,
) -> HashMap<String, String> {
    let mut result = original_palette.clone();

    for cycle in cycles {
        // Each cycle rotates independently at its own rate
        let cycle_len = cycle.cycle_length();
        if cycle_len > 0 {
            let cycle_step = step % cycle_len;
            result = apply_cycle_step(&result, cycle, cycle_step);
        }
    }

    result
}

/// Calculate the total number of frames needed to complete all cycle animations.
///
/// Returns the LCM of all cycle lengths to ensure all cycles complete at least once.
/// If no cycles, returns 1 (single frame).
pub fn calculate_total_frames(cycles: &[PaletteCycle]) -> usize {
    if cycles.is_empty() {
        return 1;
    }

    // Calculate LCM of all cycle lengths
    let mut total = 1usize;
    for cycle in cycles {
        let len = cycle.cycle_length();
        if len > 0 {
            total = lcm(total, len);
        }
    }

    total.max(1)
}

/// Calculate least common multiple of two numbers.
fn lcm(a: usize, b: usize) -> usize {
    if a == 0 || b == 0 {
        return 0;
    }
    (a * b) / gcd(a, b)
}

/// Calculate greatest common divisor using Euclidean algorithm.
fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Generate all frames for a palette-cycle animation.
///
/// For each frame, applies the palette cycle rotation and renders the sprite.
/// Returns a vector of rendered frames.
pub fn generate_cycle_frames(
    sprite: &Sprite,
    base_palette: &HashMap<String, String>,
    animation: &Animation,
) -> (Vec<RgbaImage>, Vec<String>) {
    let mut frames = Vec::new();
    let mut all_warnings = Vec::new();

    let cycles = animation.palette_cycles();
    let total_frames = calculate_total_frames(cycles);

    for step in 0..total_frames {
        // Apply all cycle rotations for this step
        let cycled_palette = apply_cycles_step(base_palette, cycles, step);

        // Render the sprite with the cycled palette
        let (image, warnings) = render_sprite(sprite, &cycled_palette);

        for w in warnings {
            // Only add warning once (they'll repeat for each frame)
            let msg = w.message.clone();
            if !all_warnings.contains(&msg) {
                all_warnings.push(msg);
            }
        }

        frames.push(image);
    }

    (frames, all_warnings)
}

/// Get the frame duration for a palette cycle animation.
///
/// Uses the cycle's duration if specified, otherwise falls back to animation duration.
pub fn get_cycle_duration(animation: &Animation) -> u32 {
    // If there are cycles with explicit durations, use the first one
    // Otherwise use animation duration
    for cycle in animation.palette_cycles() {
        if let Some(duration) = cycle.duration {
            return duration;
        }
    }
    animation.duration_ms()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaletteRef;

    fn make_palette(colors: &[(&str, &str)]) -> HashMap<String, String> {
        colors
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_apply_cycle_step_rotation() {
        let palette = make_palette(&[
            ("{w1}", "#001"),
            ("{w2}", "#002"),
            ("{w3}", "#003"),
        ]);

        let cycle = PaletteCycle {
            tokens: vec!["{w1}".to_string(), "{w2}".to_string(), "{w3}".to_string()],
            duration: None,
        };

        // Step 0: no rotation
        let result0 = apply_cycle_step(&palette, &cycle, 0);
        assert_eq!(result0.get("{w1}"), Some(&"#001".to_string()));
        assert_eq!(result0.get("{w2}"), Some(&"#002".to_string()));
        assert_eq!(result0.get("{w3}"), Some(&"#003".to_string()));

        // Step 1: rotate by 1
        let result1 = apply_cycle_step(&palette, &cycle, 1);
        assert_eq!(result1.get("{w1}"), Some(&"#002".to_string()));
        assert_eq!(result1.get("{w2}"), Some(&"#003".to_string()));
        assert_eq!(result1.get("{w3}"), Some(&"#001".to_string()));

        // Step 2: rotate by 2
        let result2 = apply_cycle_step(&palette, &cycle, 2);
        assert_eq!(result2.get("{w1}"), Some(&"#003".to_string()));
        assert_eq!(result2.get("{w2}"), Some(&"#001".to_string()));
        assert_eq!(result2.get("{w3}"), Some(&"#002".to_string()));

        // Step 3: wraps back to step 0
        let result3 = apply_cycle_step(&palette, &cycle, 3);
        assert_eq!(result3.get("{w1}"), Some(&"#001".to_string()));
        assert_eq!(result3.get("{w2}"), Some(&"#002".to_string()));
        assert_eq!(result3.get("{w3}"), Some(&"#003".to_string()));
    }

    #[test]
    fn test_apply_cycle_preserves_other_colors() {
        let palette = make_palette(&[
            ("{w1}", "#001"),
            ("{w2}", "#002"),
            ("{static}", "#999"),
        ]);

        let cycle = PaletteCycle {
            tokens: vec!["{w1}".to_string(), "{w2}".to_string()],
            duration: None,
        };

        let result = apply_cycle_step(&palette, &cycle, 1);

        // Cycled tokens changed
        assert_eq!(result.get("{w1}"), Some(&"#002".to_string()));
        assert_eq!(result.get("{w2}"), Some(&"#001".to_string()));

        // Non-cycled token preserved
        assert_eq!(result.get("{static}"), Some(&"#999".to_string()));
    }

    #[test]
    fn test_apply_cycle_empty_cycle() {
        let palette = make_palette(&[("{a}", "#111")]);

        let cycle = PaletteCycle {
            tokens: vec![],
            duration: None,
        };

        let result = apply_cycle_step(&palette, &cycle, 5);
        assert_eq!(result.get("{a}"), Some(&"#111".to_string()));
    }

    #[test]
    fn test_apply_cycles_multiple() {
        let palette = make_palette(&[
            ("{w1}", "#001"),
            ("{w2}", "#002"),
            ("{f1}", "#F01"),
            ("{f2}", "#F02"),
            ("{f3}", "#F03"),
        ]);

        let cycles = vec![
            PaletteCycle {
                tokens: vec!["{w1}".to_string(), "{w2}".to_string()],
                duration: Some(200),
            },
            PaletteCycle {
                tokens: vec!["{f1}".to_string(), "{f2}".to_string(), "{f3}".to_string()],
                duration: Some(100),
            },
        ];

        // At step 1: water rotates by 1 (mod 2), fire rotates by 1 (mod 3)
        let result = apply_cycles_step(&palette, &cycles, 1);

        // Water: w1 gets w2's color, w2 gets w1's color
        assert_eq!(result.get("{w1}"), Some(&"#002".to_string()));
        assert_eq!(result.get("{w2}"), Some(&"#001".to_string()));

        // Fire: f1 gets f2's, f2 gets f3's, f3 gets f1's
        assert_eq!(result.get("{f1}"), Some(&"#F02".to_string()));
        assert_eq!(result.get("{f2}"), Some(&"#F03".to_string()));
        assert_eq!(result.get("{f3}"), Some(&"#F01".to_string()));
    }

    #[test]
    fn test_calculate_total_frames() {
        // Single cycle of length 3
        let cycles1 = vec![PaletteCycle {
            tokens: vec!["{a}".to_string(), "{b}".to_string(), "{c}".to_string()],
            duration: None,
        }];
        assert_eq!(calculate_total_frames(&cycles1), 3);

        // Two cycles: length 2 and length 3 -> LCM = 6
        let cycles2 = vec![
            PaletteCycle {
                tokens: vec!["{a}".to_string(), "{b}".to_string()],
                duration: None,
            },
            PaletteCycle {
                tokens: vec!["{c}".to_string(), "{d}".to_string(), "{e}".to_string()],
                duration: None,
            },
        ];
        assert_eq!(calculate_total_frames(&cycles2), 6);

        // No cycles -> 1 frame
        let cycles_empty: Vec<PaletteCycle> = vec![];
        assert_eq!(calculate_total_frames(&cycles_empty), 1);
    }

    #[test]
    fn test_lcm_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 3), 1);
        assert_eq!(lcm(3, 4), 12);
        assert_eq!(lcm(6, 8), 24);
    }

    #[test]
    fn test_generate_cycle_frames_basic() {
        // Create a simple 2x1 sprite with two water tokens
        let sprite = Sprite {
            name: "water".to_string(),
            size: Some([2, 1]),
            palette: PaletteRef::Inline(HashMap::new()),
            grid: vec!["{w1}{w2}".to_string()],
            metadata: None,
            ..Default::default()
        };

        let palette = make_palette(&[
            ("{w1}", "#0000FF"),
            ("{w2}", "#00FFFF"),
        ]);

        let anim = Animation {
            name: "water_cycle".to_string(),
            frames: vec!["water".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: Some(true),
            palette_cycle: Some(vec![PaletteCycle {
                tokens: vec!["{w1}".to_string(), "{w2}".to_string()],
                duration: Some(150),
            }]),
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };

        let (frames, warnings) = generate_cycle_frames(&sprite, &palette, &anim);

        // Should generate 2 frames (cycle length = 2)
        assert_eq!(frames.len(), 2);
        assert!(warnings.is_empty());

        // Frame 0: w1=#0000FF, w2=#00FFFF
        let frame0 = &frames[0];
        assert_eq!(frame0.get_pixel(0, 0).0, [0, 0, 255, 255]); // Blue
        assert_eq!(frame0.get_pixel(1, 0).0, [0, 255, 255, 255]); // Cyan

        // Frame 1: w1=#00FFFF (from w2), w2=#0000FF (from w1)
        let frame1 = &frames[1];
        assert_eq!(frame1.get_pixel(0, 0).0, [0, 255, 255, 255]); // Cyan
        assert_eq!(frame1.get_pixel(1, 0).0, [0, 0, 255, 255]); // Blue
    }

    #[test]
    fn test_get_cycle_duration() {
        // Animation with cycle that has explicit duration
        let anim1 = Animation {
            name: "test".to_string(),
            frames: vec!["f".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: None,
            palette_cycle: Some(vec![PaletteCycle {
                tokens: vec!["{a}".to_string()],
                duration: Some(150),
            }]),
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        assert_eq!(get_cycle_duration(&anim1), 150);

        // Animation with cycle without explicit duration
        let anim2 = Animation {
            name: "test".to_string(),
            frames: vec!["f".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: None,
            palette_cycle: Some(vec![PaletteCycle {
                tokens: vec!["{a}".to_string()],
                duration: None,
            }]),
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        assert_eq!(get_cycle_duration(&anim2), 100);

        // Animation without palette_cycle
        let anim3 = Animation {
            name: "test".to_string(),
            frames: vec!["f".to_string()],
            duration: Some(Duration::Milliseconds(100)),
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
            ..Default::default()
        };
        assert_eq!(get_cycle_duration(&anim3), 100);
    }
}
