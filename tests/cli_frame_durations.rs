//! GIF delays encode per-frame overrides and retain the uniform fallback.
use image::AnimationDecoder;
use std::{fs::File, io::BufReader, process::Command};

const INPUT: &str = r##"{type:"palette",name:"p",colors:{x:"#ff0000"}}
{type:"sprite",name:"a",size:[1,1],palette:"p",regions:{x:{rect:[0,0,1,1]}}}
{type:"animation",name:"timing",frames:["a","a","a"],duration:100 EXTRA}
"##;
fn render(extra: &str) -> (std::process::Output, tempfile::TempDir) {
    render_text(&INPUT.replace(" EXTRA", extra))
}
fn render_text(text: &str) -> (std::process::Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("timing.pxl");
    std::fs::write(&input, text).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(["render", "--gif", "--animation", "timing"])
        .arg(input)
        .arg("-o")
        .arg(dir.path().join("timing.gif"))
        .output()
        .unwrap();
    (result, dir)
}
fn delays(dir: &tempfile::TempDir) -> Vec<u32> {
    image::codecs::gif::GifDecoder::new(BufReader::new(
        File::open(dir.path().join("timing.gif")).unwrap(),
    ))
    .unwrap()
    .into_frames()
    .collect_frames()
    .unwrap()
    .iter()
    .map(|f| {
        let (n, d) = f.delay().numer_denom_ms();
        n / d
    })
    .collect()
}
#[test]
fn gif_encodes_uneven_delays_with_uniform_fallback() {
    let (result, dir) = render(",frame_durations:[40,null,'250ms']");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(delays(&dir), vec![40, 100, 250]);
    println!("decoded GIF delays: {:?} ms", delays(&dir));
}
#[test]
fn existing_animation_retains_uniform_timing() {
    let (result, dir) = render("");
    assert!(result.status.success());
    assert_eq!(delays(&dir), vec![100, 100, 100]);
}
#[test]
fn invalid_per_frame_timings_have_diagnostics() {
    for durations in ["[0,100,100]", "[655360,100,100]", "['invalid',100,100]", "[100]"] {
        let (result, _) = render(&format!(",frame_durations:{durations}"));
        assert!(!result.status.success(), "accepted {durations}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("frame_durations"),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn skipped_missing_frame_keeps_remaining_delays_aligned() {
    let text = INPUT
        .replace("\"a\",\"a\",\"a\"", "\"a\",\"missing\",\"a\"")
        .replace(" EXTRA", ",frame_durations:[40,100,250]");
    let (result, dir) = render_text(&text);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(delays(&dir), vec![40, 250]);
}

#[test]
fn composition_frames_receive_per_frame_delays() {
    let text=INPUT.replace("{type:\"animation\"", "{type:\"composition\",name:\"scene\",size:[1,1],cell_size:[1,1],sprites:{A:\"a\"},layers:[{map:[\"A\"]}]}\n{type:\"animation\"")
        .replace("frames:[\"a\",\"a\",\"a\"]", "frames:[\"scene\",\"a\",\"scene\"]")
        .replace(" EXTRA", ",frame_durations:[40,100,250]");
    let (result, dir) = render_text(&text);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(delays(&dir), vec![40, 100, 250]);
}
