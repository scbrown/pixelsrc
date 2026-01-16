//! Animation validation - validate animation references

use crate::models::{Animation, FrameTag, Sprite};
use std::collections::HashMap;

/// A warning generated during animation validation
#[derive(Debug, Clone, PartialEq)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Validate frame tags against the animation's frame count.
///
/// Returns warnings for:
/// - Tag with start > end (invalid range)
/// - Tag with start or end beyond frame count bounds
fn validate_frame_tags(
    anim_name: &str,
    tags: &HashMap<String, FrameTag>,
    frame_count: usize,
) -> Vec<Warning> {
    let mut warnings = Vec::new();

    for (tag_name, tag) in tags {
        // Check start <= end
        if tag.start > tag.end {
            warnings.push(Warning::new(format!(
                "Animation '{}' tag '{}' has invalid range: start ({}) > end ({})",
                anim_name, tag_name, tag.start, tag.end
            )));
        }

        // Check bounds against frame count
        if tag.start as usize >= frame_count {
            warnings.push(Warning::new(format!(
                "Animation '{}' tag '{}' start ({}) is out of bounds (animation has {} frames)",
                anim_name, tag_name, tag.start, frame_count
            )));
        }
        if tag.end as usize >= frame_count {
            warnings.push(Warning::new(format!(
                "Animation '{}' tag '{}' end ({}) is out of bounds (animation has {} frames)",
                anim_name, tag_name, tag.end, frame_count
            )));
        }
    }

    warnings
}

/// Validate an animation against a set of sprites.
///
/// Returns warnings for:
/// - Animation with no frames (empty frames array)
/// - Animation frames that reference unknown sprites
/// - Frame tags with invalid ranges (start > end)
/// - Frame tags with out-of-bounds indices
///
/// # Examples
///
/// ```ignore
/// use pixelsrc::animation::validate_animation;
/// use pixelsrc::models::{Animation, Sprite, PaletteRef};
///
/// let anim = Animation {
///     name: "walk".to_string(),
///     frames: vec!["frame1".to_string(), "frame2".to_string()],
///     duration: None,
///     r#loop: None,
/// };
/// let sprites = vec![/* sprites with names "frame1", "frame2" */];
/// let warnings = validate_animation(&anim, &sprites);
/// assert!(warnings.is_empty());
/// ```
pub fn validate_animation(anim: &Animation, sprites: &[Sprite]) -> Vec<Warning> {
    let mut warnings = Vec::new();

    // Warn if animation has no frames
    if anim.frames.is_empty() {
        warnings.push(Warning::new(format!(
            "Animation '{}' has no frames",
            anim.name
        )));
        return warnings;
    }

    // Build set of sprite names for fast lookup
    let sprite_names: std::collections::HashSet<&str> =
        sprites.iter().map(|s| s.name.as_str()).collect();

    // Check each frame references an existing sprite
    for frame in &anim.frames {
        if !sprite_names.contains(frame.as_str()) {
            warnings.push(Warning::new(format!(
                "Animation '{}' references unknown sprite '{}'",
                anim.name, frame
            )));
        }
    }

    // Validate frame tags if present
    if let Some(tags) = &anim.tags {
        warnings.extend(validate_frame_tags(&anim.name, tags, anim.frames.len()));
    }

    // Validate frame_metadata length matches frames
    if let Some(frame_meta) = &anim.frame_metadata {
        if frame_meta.len() != anim.frames.len() {
            warnings.push(Warning::new(format!(
                "Animation '{}' has {} frames but {} frame_metadata entries",
                anim.name,
                anim.frames.len(),
                frame_meta.len()
            )));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaletteRef;
    use std::collections::HashMap;

    fn make_sprite(name: &str) -> Sprite {
        Sprite {
            name: name.to_string(),
            size: None,
            palette: PaletteRef::Inline(HashMap::from([(
                "{_}".to_string(),
                "#00000000".to_string(),
            )])),
            grid: vec!["{_}".to_string()],
            metadata: None,
        }
    }

    #[test]
    fn test_valid_animation_no_warnings() {
        // Animation with existing sprites should produce no warnings
        let anim = Animation {
            name: "walk".to_string(),
            frames: vec![
                "frame1".to_string(),
                "frame2".to_string(),
                "frame3".to_string(),
            ],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        let sprites = vec![
            make_sprite("frame1"),
            make_sprite("frame2"),
            make_sprite("frame3"),
        ];

        let warnings = validate_animation(&anim, &sprites);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_animation_missing_sprite_warning() {
        // Animation referencing non-existent sprite should warn
        let anim = Animation {
            name: "blink".to_string(),
            frames: vec!["on".to_string(), "off".to_string()],
            duration: Some(500),
            r#loop: Some(true),
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        // Only "on" sprite exists
        let sprites = vec![make_sprite("on")];

        let warnings = validate_animation(&anim, &sprites);

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("blink"));
        assert!(warnings[0].message.contains("off"));
        assert!(warnings[0].message.contains("unknown sprite"));
    }

    #[test]
    fn test_animation_empty_frames_warning() {
        // Animation with empty frames array should warn
        let anim = Animation {
            name: "empty_anim".to_string(),
            frames: vec![],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        let sprites = vec![make_sprite("some_sprite")];

        let warnings = validate_animation(&anim, &sprites);

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("empty_anim"));
        assert!(warnings[0].message.contains("no frames"));
    }

    #[test]
    fn test_animation_multiple_missing_sprites() {
        // Animation with multiple missing sprites should warn for each
        let anim = Animation {
            name: "multi_missing".to_string(),
            frames: vec![
                "exists".to_string(),
                "missing1".to_string(),
                "missing2".to_string(),
            ],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        let sprites = vec![make_sprite("exists")];

        let warnings = validate_animation(&anim, &sprites);

        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.message.contains("missing1")));
        assert!(warnings.iter().any(|w| w.message.contains("missing2")));
    }

    #[test]
    fn test_animation_all_frames_missing() {
        // Animation where all frames reference missing sprites
        let anim = Animation {
            name: "all_missing".to_string(),
            frames: vec!["ghost1".to_string(), "ghost2".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        // No matching sprites
        let sprites = vec![make_sprite("unrelated")];

        let warnings = validate_animation(&anim, &sprites);

        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_animation_empty_sprites_list() {
        // Animation validated against empty sprite list
        let anim = Animation {
            name: "no_sprites".to_string(),
            frames: vec!["frame1".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        let sprites: Vec<Sprite> = vec![];

        let warnings = validate_animation(&anim, &sprites);

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("frame1"));
    }

    #[test]
    fn test_animation_single_frame_valid() {
        // Single-frame animation with existing sprite
        let anim = Animation {
            name: "static".to_string(),
            frames: vec!["pose".to_string()],
            duration: None,
            r#loop: Some(false),
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        let sprites = vec![make_sprite("pose")];

        let warnings = validate_animation(&anim, &sprites);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_animation_duplicate_frames_valid() {
        // Animation with duplicate frame references (valid pattern for hold/repeat)
        let anim = Animation {
            name: "hold".to_string(),
            frames: vec![
                "frame1".to_string(),
                "frame1".to_string(),
                "frame2".to_string(),
                "frame1".to_string(),
            ],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: None,
            attachments: None,
        };

        let sprites = vec![make_sprite("frame1"), make_sprite("frame2")];

        let warnings = validate_animation(&anim, &sprites);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_warning_creation() {
        let warning = Warning::new("test message");
        assert_eq!(warning.message, "test message");
    }

    #[test]
    fn test_animation_frame_metadata_length_mismatch() {
        // Animation with frame_metadata length not matching frames count
        use crate::models::FrameMetadata;

        let anim = Animation {
            name: "attack".to_string(),
            frames: vec!["f1".to_string(), "f2".to_string(), "f3".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: Some(vec![
                FrameMetadata::default(), // Only 2 entries for 3 frames
                FrameMetadata::default(),
            ]),
            attachments: None,
        };

        let sprites = vec![make_sprite("f1"), make_sprite("f2"), make_sprite("f3")];
        let warnings = validate_animation(&anim, &sprites);

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("3 frames"));
        assert!(warnings[0].message.contains("2 frame_metadata"));
    }

    #[test]
    fn test_animation_frame_metadata_length_matches() {
        // Animation with frame_metadata length matching frames count
        use crate::models::FrameMetadata;

        let anim = Animation {
            name: "attack".to_string(),
            frames: vec!["f1".to_string(), "f2".to_string()],
            duration: None,
            r#loop: None,
            palette_cycle: None,
            tags: None,
            frame_metadata: Some(vec![FrameMetadata::default(), FrameMetadata::default()]),
            attachments: None,
        };

        let sprites = vec![make_sprite("f1"), make_sprite("f2")];
        let warnings = validate_animation(&anim, &sprites);

        assert!(warnings.is_empty());
    }
}
