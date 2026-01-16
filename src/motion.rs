//! Motion path interpolation for keyframe animations
//!
//! This module provides bezier curve interpolation and easing functions
//! for smooth motion in animations. It supports:
//!
//! - Linear interpolation
//! - Standard easing functions (ease-in, ease-out, ease-in-out)
//! - Cubic bezier interpolation with control points
//! - Automatic arc path fitting
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::motion::{Interpolation, interpolate, Point2D};
//!
//! // Linear interpolation between two points
//! let start = Point2D { x: 0.0, y: 0.0 };
//! let end = Point2D { x: 100.0, y: 50.0 };
//! let mid = interpolate_point(&start, &end, 0.5, Interpolation::Linear);
//! assert_eq!(mid.x, 50.0);
//! assert_eq!(mid.y, 25.0);
//! ```

use std::f64::consts::PI;

/// A 2D point for motion path calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Linear interpolation between two points
    pub fn lerp(&self, other: &Point2D, t: f64) -> Point2D {
        Point2D {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Distance to another point
    pub fn distance(&self, other: &Point2D) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Control point for bezier curves
#[derive(Debug, Clone, PartialEq)]
pub struct ControlPoint {
    /// The keyframe position
    pub position: Point2D,
    /// Bezier control point (handle) - offset from position
    pub control: Option<Point2D>,
}

impl ControlPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: Point2D::new(x, y),
            control: None,
        }
    }

    pub fn with_control(x: f64, y: f64, cx: f64, cy: f64) -> Self {
        Self {
            position: Point2D::new(x, y),
            control: Some(Point2D::new(cx, cy)),
        }
    }
}

/// Interpolation method for keyframe animation
#[derive(Debug, Clone, PartialEq)]
pub enum Interpolation {
    /// Constant speed between keyframes
    Linear,
    /// Slow start, fast end (acceleration)
    EaseIn,
    /// Fast start, slow end (deceleration)
    EaseOut,
    /// Smooth S-curve (slow start and end)
    EaseInOut,
    /// Overshoot and settle back
    Bounce,
    /// Spring-like oscillation
    Elastic,
    /// Custom cubic bezier curve
    Bezier {
        /// Control point 1 (x, y) - typically (0.0-1.0, 0.0-1.0)
        p1: (f64, f64),
        /// Control point 2 (x, y) - typically (0.0-1.0, 0.0-1.0)
        p2: (f64, f64),
    },
}

impl Default for Interpolation {
    fn default() -> Self {
        Interpolation::Linear
    }
}

/// Motion path type for position interpolation
#[derive(Debug, Clone, PartialEq)]
pub enum MotionPath {
    /// Straight line between keyframes
    Linear,
    /// Automatic arc curve fitting (parabolic for throw/jump)
    Arc,
    /// Explicit bezier with control points
    Bezier(Vec<ControlPoint>),
}

impl Default for MotionPath {
    fn default() -> Self {
        MotionPath::Linear
    }
}

/// Apply easing to a normalized time value (0.0 to 1.0)
///
/// # Arguments
/// * `t` - Normalized time (0.0 = start, 1.0 = end)
/// * `interpolation` - The easing function to apply
///
/// # Returns
/// Eased value (typically 0.0 to 1.0, but may overshoot for bounce/elastic)
pub fn ease(t: f64, interpolation: &Interpolation) -> f64 {
    let t = t.clamp(0.0, 1.0);

    match interpolation {
        Interpolation::Linear => t,

        Interpolation::EaseIn => {
            // Quadratic ease-in: t^2
            t * t
        }

        Interpolation::EaseOut => {
            // Quadratic ease-out: 1 - (1-t)^2
            1.0 - (1.0 - t) * (1.0 - t)
        }

        Interpolation::EaseInOut => {
            // Quadratic ease-in-out
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }

        Interpolation::Bounce => {
            // Bounce easing - overshoots then settles
            let t = 1.0 - t;
            if t < 1.0 / 2.75 {
                1.0 - 7.5625 * t * t
            } else if t < 2.0 / 2.75 {
                let t = t - 1.5 / 2.75;
                1.0 - (7.5625 * t * t + 0.75)
            } else if t < 2.5 / 2.75 {
                let t = t - 2.25 / 2.75;
                1.0 - (7.5625 * t * t + 0.9375)
            } else {
                let t = t - 2.625 / 2.75;
                1.0 - (7.5625 * t * t + 0.984375)
            }
        }

        Interpolation::Elastic => {
            // Elastic easing - spring-like oscillation
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let p = 0.3;
                let s = p / 4.0;
                let t = t - 1.0;
                -(2.0_f64.powf(10.0 * t) * ((t - s) * (2.0 * PI) / p).sin())
            }
        }

        Interpolation::Bezier { p1, p2 } => {
            // Cubic bezier easing
            // Using iterative approach to find t for given x, then compute y
            cubic_bezier_ease(t, p1.0, p1.1, p2.0, p2.1)
        }
    }
}

/// Cubic bezier easing calculation
///
/// Given t (0-1), compute the eased value using a cubic bezier curve.
/// The curve is defined by: P0=(0,0), P1=(x1,y1), P2=(x2,y2), P3=(1,1)
fn cubic_bezier_ease(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // For easing, we need to find the bezier parameter that gives us x=t
    // Then use that parameter to compute y

    // Newton-Raphson iteration to find bezier parameter for x=t
    let mut guess = t;
    for _ in 0..8 {
        let x = cubic_bezier_1d(guess, x1, x2);
        let dx = cubic_bezier_derivative(guess, x1, x2);
        if dx.abs() < 1e-10 {
            break;
        }
        guess -= (x - t) / dx;
        guess = guess.clamp(0.0, 1.0);
    }

    cubic_bezier_1d(guess, y1, y2)
}

/// Evaluate 1D cubic bezier at parameter t
/// Points: P0=0, P1=p1, P2=p2, P3=1
fn cubic_bezier_1d(t: f64, p1: f64, p2: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // B(t) = (1-t)^3 * P0 + 3*(1-t)^2*t * P1 + 3*(1-t)*t^2 * P2 + t^3 * P3
    // With P0=0 and P3=1, the (1-t)^3 * P0 term is 0:
    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

/// Derivative of 1D cubic bezier
fn cubic_bezier_derivative(t: f64, p1: f64, p2: f64) -> f64 {
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // B'(t) = 3*(1-t)^2*(P1-P0) + 6*(1-t)*t*(P2-P1) + 3*t^2*(P3-P2)
    // With P0=0 and P3=1:
    3.0 * mt2 * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t2 * (1.0 - p2)
}

/// Interpolate between two points with easing
pub fn interpolate_point(start: &Point2D, end: &Point2D, t: f64, interpolation: &Interpolation) -> Point2D {
    let eased_t = ease(t, interpolation);
    start.lerp(end, eased_t)
}

/// Interpolate along a motion path
///
/// # Arguments
/// * `keyframes` - List of control points defining the path
/// * `t` - Normalized time (0.0 = first keyframe, 1.0 = last keyframe)
/// * `path` - Motion path type
/// * `interpolation` - Easing function for timing
///
/// # Returns
/// Interpolated position along the path
pub fn interpolate_path(
    keyframes: &[ControlPoint],
    t: f64,
    path: &MotionPath,
    interpolation: &Interpolation,
) -> Point2D {
    if keyframes.is_empty() {
        return Point2D::new(0.0, 0.0);
    }
    if keyframes.len() == 1 {
        return keyframes[0].position;
    }

    let t = t.clamp(0.0, 1.0);

    // Find which segment we're in
    let num_segments = keyframes.len() - 1;
    let segment_t = t * num_segments as f64;
    let segment_idx = (segment_t.floor() as usize).min(num_segments - 1);
    let local_t = segment_t - segment_idx as f64;

    let p0 = &keyframes[segment_idx];
    let p1 = &keyframes[segment_idx + 1];

    match path {
        MotionPath::Linear => {
            let eased_t = ease(local_t, interpolation);
            p0.position.lerp(&p1.position, eased_t)
        }

        MotionPath::Arc => {
            // Automatic arc fitting - creates a parabolic arc
            // Good for throw/jump motions
            let eased_t = ease(local_t, interpolation);

            // Calculate arc height based on horizontal distance
            let dx = p1.position.x - p0.position.x;
            let dy = p1.position.y - p0.position.y;

            // Arc peaks at midpoint, height proportional to distance
            let arc_height = dx.abs() * 0.3; // 30% of horizontal distance

            // Parabolic arc: y offset = 4h * t * (1-t) where h is height
            let arc_offset = 4.0 * arc_height * eased_t * (1.0 - eased_t);

            Point2D {
                x: p0.position.x + dx * eased_t,
                y: p0.position.y + dy * eased_t - arc_offset, // Negative because y-up
            }
        }

        MotionPath::Bezier(_) => {
            // Use control points for cubic bezier curve
            let eased_t = ease(local_t, interpolation);

            // Get control points (use position if no explicit control)
            let c0 = p0.control.unwrap_or(p0.position);
            let c1 = p1.control.unwrap_or(p1.position);

            cubic_bezier_point(&p0.position, &c0, &c1, &p1.position, eased_t)
        }
    }
}

/// Evaluate cubic bezier curve at parameter t
fn cubic_bezier_point(p0: &Point2D, p1: &Point2D, p2: &Point2D, p3: &Point2D, t: f64) -> Point2D {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Point2D {
        x: mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        y: mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    }
}

/// Interpolate a single value between keyframes
///
/// # Arguments
/// * `start` - Start value
/// * `end` - End value
/// * `t` - Normalized time (0.0 to 1.0)
/// * `interpolation` - Easing function
///
/// # Returns
/// Interpolated value
pub fn interpolate_value(start: f64, end: f64, t: f64, interpolation: &Interpolation) -> f64 {
    let eased_t = ease(t, interpolation);
    start + (end - start) * eased_t
}

/// Generate intermediate frames for a keyframe animation
///
/// # Arguments
/// * `keyframes` - Map of frame number to position
/// * `total_frames` - Total number of frames to generate
/// * `path` - Motion path type
/// * `interpolation` - Easing function
///
/// # Returns
/// Vector of positions, one per frame
pub fn generate_motion_frames(
    keyframes: &[(u32, Point2D)],
    total_frames: u32,
    path: &MotionPath,
    interpolation: &Interpolation,
) -> Vec<Point2D> {
    if keyframes.is_empty() || total_frames == 0 {
        return vec![];
    }

    // Build frame-to-keyframe mapping
    let mut frames = Vec::with_capacity(total_frames as usize);

    for frame in 0..total_frames {
        // Find surrounding keyframes
        let mut prev_kf: Option<(u32, &Point2D)> = None;
        let mut next_kf: Option<(u32, &Point2D)> = None;

        for (kf_frame, pos) in keyframes {
            if *kf_frame <= frame {
                prev_kf = Some((*kf_frame, pos));
            }
            if *kf_frame >= frame && next_kf.is_none() {
                next_kf = Some((*kf_frame, pos));
            }
        }

        let pos = match (prev_kf, next_kf) {
            (Some((pf, pp)), Some((nf, np))) if pf != nf => {
                // Interpolate between keyframes
                let t = (frame - pf) as f64 / (nf - pf) as f64;

                match path {
                    MotionPath::Linear => interpolate_point(pp, np, t, interpolation),
                    MotionPath::Arc => {
                        let eased_t = ease(t, interpolation);
                        let dx = np.x - pp.x;
                        let dy = np.y - pp.y;
                        let arc_height = dx.abs() * 0.3;
                        let arc_offset = 4.0 * arc_height * eased_t * (1.0 - eased_t);
                        Point2D {
                            x: pp.x + dx * eased_t,
                            y: pp.y + dy * eased_t - arc_offset,
                        }
                    }
                    MotionPath::Bezier(_) => {
                        // For bezier, use the full path interpolation
                        // This is simplified - real impl would use segment control points
                        interpolate_point(pp, np, t, interpolation)
                    }
                }
            }
            (Some((_, pos)), _) => *pos,
            (_, Some((_, pos))) => *pos,
            _ => Point2D::new(0.0, 0.0),
        };

        frames.push(pos);
    }

    frames
}

/// Parse interpolation mode from string
pub fn parse_interpolation(s: &str) -> Option<Interpolation> {
    match s.to_lowercase().as_str() {
        "linear" => Some(Interpolation::Linear),
        "ease-in" | "easein" => Some(Interpolation::EaseIn),
        "ease-out" | "easeout" => Some(Interpolation::EaseOut),
        "ease-in-out" | "easeinout" | "ease" => Some(Interpolation::EaseInOut),
        "bounce" => Some(Interpolation::Bounce),
        "elastic" => Some(Interpolation::Elastic),
        _ => None,
    }
}

/// Parse motion path from string
pub fn parse_motion_path(s: &str) -> Option<MotionPath> {
    match s.to_lowercase().as_str() {
        "linear" => Some(MotionPath::Linear),
        "arc" => Some(MotionPath::Arc),
        "bezier" => Some(MotionPath::Bezier(vec![])),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Point2D Tests
    // ========================================================================

    #[test]
    fn test_point_lerp() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(100.0, 50.0);

        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 25.0).abs() < 0.001);

        let start = a.lerp(&b, 0.0);
        assert!((start.x - 0.0).abs() < 0.001);
        assert!((start.y - 0.0).abs() < 0.001);

        let end = a.lerp(&b, 1.0);
        assert!((end.x - 100.0).abs() < 0.001);
        assert!((end.y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_point_distance() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 0.001);
    }

    // ========================================================================
    // Easing Tests
    // ========================================================================

    #[test]
    fn test_ease_linear() {
        assert!((ease(0.0, &Interpolation::Linear) - 0.0).abs() < 0.001);
        assert!((ease(0.5, &Interpolation::Linear) - 0.5).abs() < 0.001);
        assert!((ease(1.0, &Interpolation::Linear) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_in() {
        // Ease-in is slow at start, so at t=0.5, result should be < 0.5
        let mid = ease(0.5, &Interpolation::EaseIn);
        assert!(mid < 0.5);
        assert!((ease(0.0, &Interpolation::EaseIn) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &Interpolation::EaseIn) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_out() {
        // Ease-out is fast at start, so at t=0.5, result should be > 0.5
        let mid = ease(0.5, &Interpolation::EaseOut);
        assert!(mid > 0.5);
        assert!((ease(0.0, &Interpolation::EaseOut) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &Interpolation::EaseOut) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_in_out() {
        // Ease-in-out should be close to 0.5 at t=0.5 (but exactly 0.5 for quadratic)
        let mid = ease(0.5, &Interpolation::EaseInOut);
        assert!((mid - 0.5).abs() < 0.001);
        assert!((ease(0.0, &Interpolation::EaseInOut) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &Interpolation::EaseInOut) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_bezier() {
        // CSS ease equivalent: cubic-bezier(0.25, 0.1, 0.25, 1.0)
        let bezier = Interpolation::Bezier {
            p1: (0.25, 0.1),
            p2: (0.25, 1.0),
        };
        assert!((ease(0.0, &bezier) - 0.0).abs() < 0.01);
        assert!((ease(1.0, &bezier) - 1.0).abs() < 0.01);

        // Mid value should be computed via bezier
        let mid = ease(0.5, &bezier);
        assert!(mid > 0.0 && mid < 1.0);
    }

    // ========================================================================
    // Motion Path Tests
    // ========================================================================

    #[test]
    fn test_interpolate_path_linear() {
        let keyframes = vec![
            ControlPoint::new(0.0, 0.0),
            ControlPoint::new(100.0, 50.0),
        ];

        let mid = interpolate_path(&keyframes, 0.5, &MotionPath::Linear, &Interpolation::Linear);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_path_arc() {
        let keyframes = vec![
            ControlPoint::new(0.0, 0.0),
            ControlPoint::new(100.0, 0.0),
        ];

        // Arc should peak at midpoint with upward curve (negative y)
        let mid = interpolate_path(&keyframes, 0.5, &MotionPath::Arc, &Interpolation::Linear);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!(mid.y < 0.0); // Should be above the baseline (negative y is up)
    }

    #[test]
    fn test_interpolate_path_multiple_keyframes() {
        let keyframes = vec![
            ControlPoint::new(0.0, 0.0),
            ControlPoint::new(50.0, 25.0),
            ControlPoint::new(100.0, 0.0),
        ];

        let start = interpolate_path(&keyframes, 0.0, &MotionPath::Linear, &Interpolation::Linear);
        assert!((start.x - 0.0).abs() < 0.001);

        let mid = interpolate_path(&keyframes, 0.5, &MotionPath::Linear, &Interpolation::Linear);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 25.0).abs() < 0.001);

        let end = interpolate_path(&keyframes, 1.0, &MotionPath::Linear, &Interpolation::Linear);
        assert!((end.x - 100.0).abs() < 0.001);
    }

    // ========================================================================
    // Value Interpolation Tests
    // ========================================================================

    #[test]
    fn test_interpolate_value() {
        let result = interpolate_value(0.0, 100.0, 0.5, &Interpolation::Linear);
        assert!((result - 50.0).abs() < 0.001);

        let result = interpolate_value(10.0, 20.0, 0.0, &Interpolation::Linear);
        assert!((result - 10.0).abs() < 0.001);

        let result = interpolate_value(10.0, 20.0, 1.0, &Interpolation::Linear);
        assert!((result - 20.0).abs() < 0.001);
    }

    // ========================================================================
    // Frame Generation Tests
    // ========================================================================

    #[test]
    fn test_generate_motion_frames() {
        let keyframes = vec![
            (0, Point2D::new(0.0, 0.0)),
            (10, Point2D::new(100.0, 50.0)),
        ];

        let frames = generate_motion_frames(&keyframes, 11, &MotionPath::Linear, &Interpolation::Linear);
        assert_eq!(frames.len(), 11);

        // First frame should be at start
        assert!((frames[0].x - 0.0).abs() < 0.001);
        assert!((frames[0].y - 0.0).abs() < 0.001);

        // Last frame should be at end
        assert!((frames[10].x - 100.0).abs() < 0.001);
        assert!((frames[10].y - 50.0).abs() < 0.001);

        // Middle frame should be halfway
        assert!((frames[5].x - 50.0).abs() < 0.001);
        assert!((frames[5].y - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_motion_frames_arc() {
        let keyframes = vec![
            (0, Point2D::new(0.0, 0.0)),
            (10, Point2D::new(100.0, 0.0)),
        ];

        let frames = generate_motion_frames(&keyframes, 11, &MotionPath::Arc, &Interpolation::Linear);
        assert_eq!(frames.len(), 11);

        // Middle frame should be above baseline (negative y)
        assert!(frames[5].y < 0.0);
    }

    // ========================================================================
    // Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_interpolation() {
        assert_eq!(parse_interpolation("linear"), Some(Interpolation::Linear));
        assert_eq!(parse_interpolation("ease-in"), Some(Interpolation::EaseIn));
        assert_eq!(parse_interpolation("easein"), Some(Interpolation::EaseIn));
        assert_eq!(parse_interpolation("ease-out"), Some(Interpolation::EaseOut));
        assert_eq!(parse_interpolation("ease-in-out"), Some(Interpolation::EaseInOut));
        assert_eq!(parse_interpolation("ease"), Some(Interpolation::EaseInOut));
        assert_eq!(parse_interpolation("bounce"), Some(Interpolation::Bounce));
        assert_eq!(parse_interpolation("elastic"), Some(Interpolation::Elastic));
        assert_eq!(parse_interpolation("invalid"), None);
    }

    #[test]
    fn test_parse_motion_path() {
        assert_eq!(parse_motion_path("linear"), Some(MotionPath::Linear));
        assert_eq!(parse_motion_path("arc"), Some(MotionPath::Arc));
        assert_eq!(parse_motion_path("bezier"), Some(MotionPath::Bezier(vec![])));
        assert_eq!(parse_motion_path("invalid"), None);
    }
}
