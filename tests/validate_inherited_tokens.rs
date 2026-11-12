//! Named palette usage follows the geometry used by rendering.
use pixelsrc::validate::{IssueType, Validator};

const PALETTES: &str = r##"{ type: "palette", name: "p", colors: { _: "transparent", a: "#ff0000", b: "#0000ff" } }
{ type: "palette", name: "q", colors: { _: "transparent", a: "#00ff00", b: "#ffffff" } }"##;

fn unused(extra: &str) -> Vec<String> {
    let mut validator = Validator::new();
    let input = format!("{PALETTES}\n{extra}");
    for (i, line) in input.lines().enumerate() {
        validator.validate_line(i + 1, line);
    }
    validator.report_unused_palette_tokens();
    validator
        .issues()
        .iter()
        .filter(|i| {
            matches!(i.issue_type, IssueType::UnusedToken)
                && i.context.as_deref() == Some("palette \"q\"")
        })
        .map(|i| i.message.clone())
        .collect()
}

#[test]
fn source_credits_inherited_tokens_but_still_reports_unused_token() {
    let reports = unused(
        r##"{ type: "sprite", name: "dot", size: [4,4], palette: "p", regions: { a: { rect: [0,0,4,4] } } }
{ type: "sprite", name: "dot_q", palette: "q", source: "dot" }"##,
    );
    assert_eq!(reports.len(), 1);
    assert!(reports[0].contains("region: b —"), "{reports:?}");
}

#[test]
fn source_chain_with_forward_reference_credits_every_reached_token() {
    assert!(unused(r##"{ type: "sprite", name: "dot_q", palette: "q", source: "middle" }
{ type: "sprite", name: "middle", palette: "p", source: "dot" }
{ type: "sprite", name: "dot", size: [4,4], palette: "p", regions: { a: { rect: [0,0,2,4] }, b: { rect: [2,0,2,4] } } }"##).is_empty());
}

#[test]
fn extends_uses_effective_regions_after_removal() {
    let reports = unused(
        r##"{ type: "sprite", name: "dot", size: [4,4], palette: "p", regions: { a: { rect: [0,0,2,4] }, b: { rect: [2,0,2,4] } } }
{ type: "sprite", name: "dot_q", palette: "q", extends: "dot", remove: ["b"] }"##,
    );
    assert_eq!(reports.len(), 1);
    assert!(reports[0].contains("region: b —"), "{reports:?}");
}

#[test]
fn variant_base_can_inherit_its_regions_from_a_source() {
    assert!(unused(r##"{ type: "sprite", name: "dot", size: [4,4], palette: "p", regions: { a: { rect: [0,0,2,4] }, b: { rect: [2,0,2,4] } } }
{ type: "sprite", name: "dot_q", palette: "q", source: "dot" }
{ type: "variant", name: "night", base: "dot_q", palette: { a: "#111111" } }"##).is_empty());
}

#[test]
fn palette_swap_repro_validates_in_strict_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_pxl"))
        .args(["validate", "tests/fixtures/valid/source_palette_swap.pxl", "--strict"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("tokens never used"));
}
