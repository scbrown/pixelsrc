//! Region-level `transform` strings.
//!
//! A region's `transform` is a space-separated sequence of operations applied
//! left to right to the region's rasterized pixel set:
//!
//! - `translate(x, y)` / `translate(x)` — integer offset in pixels
//! - `scale(n)` / `scale(x, y)` — positive integer, nearest-neighbour
//! - `rotate(<angle>deg)` / `rotate(<angle>rad)` — a multiple of 90deg
//!   (positive is clockwise); radians within 1e-9 of `k * pi / 2` count
//! - `flip-x` / `flip-y` — mirror across the pivot column / row
//!
//! Every operation acts about one fixed pivot: the top-left pixel of the
//! region's bounding box before the transform. Because the pivot does not move,
//! order matters: `rotate(90deg) translate(2,0)` and `translate(2,0) rotate(90deg)`
//! differ. Any other angle, a non-integer offset or scale, or an unknown
//! operation is an error: pixel art is never silently resampled.

use std::collections::HashSet;

/// One operation of a region `transform` string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionTransformOp {
    /// Integer offset.
    Translate(i32, i32),
    /// Positive integer nearest-neighbour scale.
    Scale(u32, u32),
    /// Clockwise rotation in quarter turns (0..=3).
    Rotate(u8),
    /// Mirror across the pivot column.
    FlipX,
    /// Mirror across the pivot row.
    FlipY,
}

const EPS: f64 = 1e-9;

fn parse_integer(func: &str, arg: &str) -> Result<i64, String> {
    let arg = arg.trim();
    let arg = arg.strip_suffix("px").unwrap_or(arg).trim();
    let value: f64 = arg.parse().map_err(|_| format!("{}(): '{}' is not a number", func, arg))?;
    if !value.is_finite() || (value - value.round()).abs() > EPS {
        return Err(format!(
            "{}(): '{}' is not an integer (pixel art is not resampled)",
            func, arg
        ));
    }
    Ok(value.round() as i64)
}

fn split_args(args: &str) -> Vec<&str> {
    args.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).collect()
}

fn parse_rotate(arg: &str) -> Result<u8, String> {
    let arg = arg.trim();
    let (number, quarter) = if let Some(n) = arg.strip_suffix("deg") {
        (n, 90.0)
    } else if let Some(n) = arg.strip_suffix("rad") {
        (n, std::f64::consts::FRAC_PI_2)
    } else {
        return Err(format!("rotate(): '{}' needs a 'deg' or 'rad' unit", arg));
    };
    let value: f64 =
        number.trim().parse().map_err(|_| format!("rotate(): '{}' is not an angle", arg))?;
    let turns = value / quarter;
    if !turns.is_finite() || (turns - turns.round()).abs() * quarter > EPS {
        return Err(format!(
            "rotate({}): only multiples of 90deg are allowed; arbitrary rotation is lossy in pixel art",
            arg
        ));
    }
    Ok((turns.round() as i64).rem_euclid(4) as u8)
}

/// Parse a region `transform` string into its operations, in order.
pub fn parse_region_transform(input: &str) -> Result<Vec<RegionTransformOp>, String> {
    let mut ops = Vec::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        let name_end = rest.find(|c: char| c == '(' || c.is_whitespace()).unwrap_or(rest.len());
        let name = &rest[..name_end];
        let after = rest[name_end..].trim_start();
        let (args, remaining) = if let Some(open) = after.strip_prefix('(') {
            let close = open.find(')').ok_or_else(|| format!("{}: missing ')'", name))?;
            (Some(&open[..close]), &open[close + 1..])
        } else {
            (None, after)
        };
        rest = remaining.trim_start();

        let args = args.map(split_args);
        let op = match (name.to_ascii_lowercase().as_str(), args.as_deref()) {
            ("translate", Some([x])) => RegionTransformOp::Translate(
                to_i32("translate", parse_integer("translate", x)?)?,
                0,
            ),
            ("translate", Some([x, y])) => RegionTransformOp::Translate(
                to_i32("translate", parse_integer("translate", x)?)?,
                to_i32("translate", parse_integer("translate", y)?)?,
            ),
            ("scale", Some([n])) => {
                let n = to_scale(parse_integer("scale", n)?)?;
                RegionTransformOp::Scale(n, n)
            }
            ("scale", Some([x, y])) => RegionTransformOp::Scale(
                to_scale(parse_integer("scale", x)?)?,
                to_scale(parse_integer("scale", y)?)?,
            ),
            ("rotate", Some([angle])) => RegionTransformOp::Rotate(parse_rotate(angle)?),
            ("flip-x", None | Some([])) => RegionTransformOp::FlipX,
            ("flip-y", None | Some([])) => RegionTransformOp::FlipY,
            ("translate" | "scale" | "rotate", _) => {
                return Err(format!("{}(): wrong number of arguments", name))
            }
            _ => return Err(format!("unknown transform operation '{}'", name)),
        };
        ops.push(op);
    }
    Ok(ops)
}

fn to_i32(func: &str, v: i64) -> Result<i32, String> {
    i32::try_from(v).map_err(|_| format!("{}(): {} is out of range", func, v))
}

fn to_scale(v: i64) -> Result<u32, String> {
    if !(1..=4096).contains(&v) {
        return Err(format!("scale(): factor {} must be a positive integer", v));
    }
    Ok(v as u32)
}

/// Apply parsed operations, left to right, about the pixel set's top-left pivot.
pub fn apply_region_transform(
    pixels: HashSet<(i32, i32)>,
    ops: &[RegionTransformOp],
) -> HashSet<(i32, i32)> {
    let (Some(px), Some(py)) = (pixels.iter().map(|p| p.0).min(), pixels.iter().map(|p| p.1).min())
    else {
        return pixels;
    };
    // Work in offsets from the fixed pivot.
    let mut offsets: Vec<(i64, i64)> =
        pixels.iter().map(|&(x, y)| (i64::from(x - px), i64::from(y - py))).collect();
    for op in ops {
        offsets = match *op {
            RegionTransformOp::Translate(tx, ty) => {
                offsets.into_iter().map(|(x, y)| (x + i64::from(tx), y + i64::from(ty))).collect()
            }
            RegionTransformOp::Scale(sx, sy) => {
                let (sx, sy) = (i64::from(sx), i64::from(sy));
                let mut out = Vec::with_capacity(offsets.len() * (sx * sy) as usize);
                for (x, y) in offsets {
                    for j in 0..sy {
                        for i in 0..sx {
                            out.push((x * sx + i, y * sy + j));
                        }
                    }
                }
                out
            }
            // Clockwise on a y-down grid: (x, y) -> (-y, x).
            RegionTransformOp::Rotate(q) => offsets
                .into_iter()
                .map(|(x, y)| match q {
                    1 => (-y, x),
                    2 => (-x, -y),
                    3 => (y, -x),
                    _ => (x, y),
                })
                .collect(),
            RegionTransformOp::FlipX => offsets.into_iter().map(|(x, y)| (-x, y)).collect(),
            RegionTransformOp::FlipY => offsets.into_iter().map(|(x, y)| (x, -y)).collect(),
        };
    }
    offsets
        .into_iter()
        .filter_map(|(x, y)| {
            Some((i32::try_from(x + i64::from(px)).ok()?, i32::try_from(y + i64::from(py)).ok()?))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use RegionTransformOp::*;

    #[test]
    fn parses_each_operation() {
        assert_eq!(
            parse_region_transform("translate(2, -3) scale(2) scale(1,3) rotate(90deg) flip-x"),
            Ok(vec![Translate(2, -3), Scale(2, 2), Scale(1, 3), Rotate(1), FlipX])
        );
        assert_eq!(parse_region_transform("rotate(-90deg)"), Ok(vec![Rotate(3)]));
        assert_eq!(parse_region_transform("rotate(450deg)"), Ok(vec![Rotate(1)]));
        assert_eq!(
            parse_region_transform(&format!("rotate({}rad)", std::f64::consts::PI)),
            Ok(vec![Rotate(2)])
        );
        assert_eq!(parse_region_transform("rotate(1.5707963267948966rad)"), Ok(vec![Rotate(1)]));
        assert_eq!(parse_region_transform("  "), Ok(vec![]));
    }

    #[test]
    fn rejects_lossy_or_unknown_operations() {
        for bad in [
            "rotate(45deg)",
            "rotate(1rad)",
            "rotate(1.5708rad)",
            "rotate(90)",
            "scale(1.5)",
            "scale(0)",
            "scale(-1)",
            "translate(1.5, 0)",
            "skew(10deg)",
            "translate(1, 2",
            "rotate(90deg, 1)",
        ] {
            assert!(parse_region_transform(bad).is_err(), "{bad} should be rejected");
        }
        let err = parse_region_transform("rotate(45deg)").unwrap_err();
        assert!(err.contains("multiples of 90deg"), "{err}");
    }

    #[test]
    fn rotation_is_about_the_top_left_pivot() {
        let line: HashSet<_> = [(3, 1), (3, 2), (3, 3)].into_iter().collect();
        let rotated = apply_region_transform(line, &[Rotate(1)]);
        let expected: HashSet<_> = [(3, 1), (2, 1), (1, 1)].into_iter().collect();
        assert_eq!(rotated, expected);
    }
}
