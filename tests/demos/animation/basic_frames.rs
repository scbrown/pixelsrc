//! Basic Frames Demo Tests

use crate::demos::{assert_frame_count, assert_validates, capture_render_info};

/// @demo format/animation#basic
/// @title Basic Frame Sequence
/// @description Simple animation with multiple frames and FPS timing.
#[test]
fn test_basic_frames() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");
    assert_validates(jsonl, true);

    // Verify animation has 3 frames
    assert_frame_count(jsonl, "blink", 3);

    // Verify each frame sprite can be rendered
    for frame_name in ["frame1", "frame2", "frame3"] {
        let info = capture_render_info(jsonl, frame_name);
        assert_eq!(info.width, 3, "Frame '{}' should be 3 pixels wide", frame_name);
        assert_eq!(info.height, 3, "Frame '{}' should be 3 pixels tall", frame_name);
    }
}

/// @demo format/animation#basic_dimensions
/// @title Frame Dimensions
/// @description All frames in an animation should have consistent dimensions.
#[test]
fn test_basic_frames_dimensions() {
    let jsonl = include_str!("../../../examples/demos/animation/basic_frames.jsonl");

    // Capture all frame dimensions
    let frames: Vec<_> = ["frame1", "frame2", "frame3"]
        .iter()
        .map(|name| capture_render_info(jsonl, name))
        .collect();

    // Verify all frames have the same dimensions
    let first = &frames[0];
    for (i, frame) in frames.iter().enumerate() {
        assert_eq!(
            frame.width, first.width,
            "Frame {} width should match frame 0",
            i
        );
        assert_eq!(
            frame.height, first.height,
            "Frame {} height should match frame 0",
            i
        );
    }
}
