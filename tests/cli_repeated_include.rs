//! A palette include may be resolved again after its previous resolution completes.
use image::AnimationDecoder;
use std::{fs::File, io::BufReader, process::Command};

#[test]
fn repeated_gif_frame_reuses_palette_include_without_false_cycle() {
    for strict in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("base.pxl"),
            r##"{type:"palette",name:"p",colors:{_:"transparent",x:"#ff0000"}}"##,
        )
        .unwrap();
        let input = dir.path().join("anim.pxl");
        std::fs::write(&input,r#"{type:"sprite",name:"a",size:[4,4],palette:"@include:base.pxl",regions:{x:{points:[[0,0]]}}}
{type:"animation",name:"an",frames:["a","a"],duration:100}"#).unwrap();
        let path = dir.path().join("anim.gif");
        let mut command = Command::new(env!("CARGO_BIN_EXE_pxl"));
        command.args(["render", "--gif", "--animation", "an"]).arg(input).arg("-o").arg(&path);
        if strict {
            command.arg("--strict");
        }
        let result = command.output().unwrap();
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(result.status.success(), "{stderr}");
        assert!(
            !stderr.contains("Circular include") && !stderr.contains("Unknown token"),
            "{stderr}"
        );
        let frames = image::codecs::gif::GifDecoder::new(BufReader::new(File::open(path).unwrap()))
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        assert_eq!(frames.len(), 2);
        for frame in &frames {
            assert_eq!(frame.buffer().get_pixel(0, 0).0, [255, 0, 0, 255]);
        }
        assert_eq!(frames[0].buffer(), frames[1].buffer());
        println!("strict={strict}: both decoded GIF frames are red, no include warnings");
    }
}
