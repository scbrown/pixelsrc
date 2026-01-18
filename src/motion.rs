//! Motion path interpolation for keyframe animations
//!
//! This module provides bezier curve interpolation and easing functions
//! for smooth motion in animations. It supports:
//!
//! - Linear interpolation
//! - Standard easing functions (ease-in, ease-out, ease-in-out)
//! - Cubic bezier interpolation with control points
//! - CSS steps() timing function
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
use std::fmt;

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
        Point2D { x: self.x + (other.x - self.x) * t, y: self.y + (other.y - self.y) * t }
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
        Self { position: Point2D::new(x, y), control: None }
    }

    pub fn with_control(x: f64, y: f64, cx: f64, cy: f64) -> Self {
        Self { position: Point2D::new(x, y), control: Some(Point2D::new(cx, cy)) }
    }
}

/// Step position for CSS steps() timing function.
///
/// Controls when the step occurs within each interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepPosition {
    /// Step occurs at the start of each interval (step-start, jump-start)
    JumpStart,
    /// Step occurs at the end of each interval (step-end, jump-end) - default
    #[default]
    JumpEnd,
    /// No step at 0% or 100%, steps occur only in between
    JumpNone,
    /// Steps occur at both 0% and 100%
    JumpBoth,
}

impl fmt::Display for StepPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepPosition::JumpStart => write!(f, "jump-start"),
            StepPosition::JumpEnd => write!(f, "jump-end"),
            StepPosition::JumpNone => write!(f, "jump-none"),
            StepPosition::JumpBoth => write!(f, "jump-both"),
        }
    }
}

/// Interpolation method for keyframe animation
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Interpolation {
    /// Constant speed between keyframes
    #[default]
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
    /// CSS steps() timing function - discrete jumps between values
    Steps {
        /// Number of steps
        count: u32,
        /// When the step occurs within each interval
        position: StepPosition,
    },
}

/// Motion path type for position interpolation
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MotionPath {
    /// Straight line between keyframes
    #[default]
    Linear,
    /// Automatic arc curve fitting (parabolic for throw/jump)
    Arc,
    /// Explicit bezier with control points
    Bezier(Vec<ControlPoint>),
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

        Interpolation::Steps { count, position } => {
            // CSS steps() timing function - discrete jumps
            steps_ease(t, *count, *position)
        }
    }
}

/// CSS steps() easing calculation
///
/// Implements the CSS steps() timing function with all step position variants.
/// See: https://www.w3.org/TR/css-easing-1/#step-easing-functions
fn steps_ease(t: f64, count: u32, position: StepPosition) -> f64 {
    if count == 0 {
        return t;
    }

    let steps = count as f64;

    match position {
        StepPosition::JumpStart => {
            // Jump at start: ceiling((t * steps)) / steps
            // step-start = steps(1, jump-start)
            (t * steps).ceil() / steps
        }
        StepPosition::JumpEnd => {
            // Jump at end: floor((t * steps)) / steps
            // step-end = steps(1, jump-end)
            (t * steps).floor() / steps
        }
        StepPosition::JumpNone => {
            // No jump at 0 or 1, only in between
            // Effectively steps-1 intervals, output ranges from 0 to 1
            if count == 1 {
                // Special case: with 1 step and jump-none, output is always 0 until end
                if t >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            } else {
                let intervals = steps - 1.0;
                ((t * intervals).floor() / intervals).min(1.0)
            }
        }
        StepPosition::JumpBoth => {
            // Jump at both 0 and 1
            // steps+1 output values for steps intervals
            let intervals = steps;
            let output_steps = steps + 1.0;
            ((t * intervals).floor() + 1.0) / output_steps
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
pub fn interpolate_point(
    start: &Point2D,
    end: &Point2D,
    t: f64,
    interpolation: &Interpolation,
) -> Point2D {
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
                        Point2D { x: pp.x + dx * eased_t, y: pp.y + dy * eased_t - arc_offset }
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

/// Error type for timing function parsing
#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunctionError {
    /// Empty input string
    Empty,
    /// Unknown timing function name
    UnknownFunction(String),
    /// Invalid cubic-bezier parameters
    InvalidBezier(String),
    /// Invalid steps parameters
    InvalidSteps(String),
    /// Syntax error in function call
    Syntax(String),
}

impl fmt::Display for TimingFunctionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimingFunctionError::Empty => write!(f, "empty timing function"),
            TimingFunctionError::UnknownFunction(name) => {
                write!(f, "unknown timing function: {}", name)
            }
            TimingFunctionError::InvalidBezier(msg) => {
                write!(f, "invalid cubic-bezier: {}", msg)
            }
            TimingFunctionError::InvalidSteps(msg) => {
                write!(f, "invalid steps: {}", msg)
            }
            TimingFunctionError::Syntax(msg) => {
                write!(f, "syntax error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TimingFunctionError {}

/// Parse a CSS timing function string.
///
/// Supports:
/// - Named functions: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`,
///   `step-start`, `step-end`
/// - `cubic-bezier(x1, y1, x2, y2)` - custom bezier curve
/// - `steps(count)` or `steps(count, position)` - discrete steps
///
/// # Examples
///
/// ```
/// use pixelsrc::motion::{parse_timing_function, Interpolation, StepPosition};
///
/// // Named functions
/// let ease = parse_timing_function("ease").unwrap();
/// assert!(matches!(ease, Interpolation::EaseInOut));
///
/// // Cubic bezier
/// let bezier = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)").unwrap();
/// assert!(matches!(bezier, Interpolation::Bezier { .. }));
///
/// // Steps
/// let steps = parse_timing_function("steps(4, jump-end)").unwrap();
/// assert!(matches!(steps, Interpolation::Steps { count: 4, position: StepPosition::JumpEnd }));
/// ```
pub fn parse_timing_function(s: &str) -> Result<Interpolation, TimingFunctionError> {
    let s = s.trim();

    if s.is_empty() {
        return Err(TimingFunctionError::Empty);
    }

    let lower = s.to_lowercase();

    // Check for named functions first
    match lower.as_str() {
        "linear" => return Ok(Interpolation::Linear),
        "ease" => return Ok(Interpolation::EaseInOut),
        "ease-in" => return Ok(Interpolation::EaseIn),
        "ease-out" => return Ok(Interpolation::EaseOut),
        "ease-in-out" => return Ok(Interpolation::EaseInOut),
        "step-start" => {
            return Ok(Interpolation::Steps { count: 1, position: StepPosition::JumpStart })
        }
        "step-end" => {
            return Ok(Interpolation::Steps { count: 1, position: StepPosition::JumpEnd })
        }
        // Also support our custom named functions
        "bounce" => return Ok(Interpolation::Bounce),
        "elastic" => return Ok(Interpolation::Elastic),
        _ => {}
    }

    // Check for function syntax: name(args)
    if let Some(paren_start) = s.find('(') {
        let paren_end = s.rfind(')').ok_or_else(|| {
            TimingFunctionError::Syntax("missing closing parenthesis".to_string())
        })?;

        if paren_end <= paren_start {
            return Err(TimingFunctionError::Syntax("invalid parentheses".to_string()));
        }

        let func_name = s[..paren_start].trim().to_lowercase();
        let args_str = s[paren_start + 1..paren_end].trim();

        match func_name.as_str() {
            "cubic-bezier" => parse_cubic_bezier(args_str),
            "steps" => parse_steps(args_str),
            _ => Err(TimingFunctionError::UnknownFunction(func_name)),
        }
    } else {
        Err(TimingFunctionError::UnknownFunction(s.to_string()))
    }
}

/// Parse cubic-bezier(x1, y1, x2, y2) arguments
fn parse_cubic_bezier(args: &str) -> Result<Interpolation, TimingFunctionError> {
    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

    if parts.len() != 4 {
        return Err(TimingFunctionError::InvalidBezier(format!(
            "expected 4 values, got {}",
            parts.len()
        )));
    }

    let x1: f64 = parts[0].parse().map_err(|_| {
        TimingFunctionError::InvalidBezier(format!("invalid x1 value: {}", parts[0]))
    })?;
    let y1: f64 = parts[1].parse().map_err(|_| {
        TimingFunctionError::InvalidBezier(format!("invalid y1 value: {}", parts[1]))
    })?;
    let x2: f64 = parts[2].parse().map_err(|_| {
        TimingFunctionError::InvalidBezier(format!("invalid x2 value: {}", parts[2]))
    })?;
    let y2: f64 = parts[3].parse().map_err(|_| {
        TimingFunctionError::InvalidBezier(format!("invalid y2 value: {}", parts[3]))
    })?;

    // CSS spec: x values must be in [0, 1]
    if !(0.0..=1.0).contains(&x1) {
        return Err(TimingFunctionError::InvalidBezier(format!(
            "x1 must be between 0 and 1, got {}",
            x1
        )));
    }
    if !(0.0..=1.0).contains(&x2) {
        return Err(TimingFunctionError::InvalidBezier(format!(
            "x2 must be between 0 and 1, got {}",
            x2
        )));
    }
    // y values can be outside [0, 1] for overshoot effects

    Ok(Interpolation::Bezier { p1: (x1, y1), p2: (x2, y2) })
}

/// Parse steps(count) or steps(count, position) arguments
fn parse_steps(args: &str) -> Result<Interpolation, TimingFunctionError> {
    let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

    if parts.is_empty() || parts.len() > 2 {
        return Err(TimingFunctionError::InvalidSteps(format!(
            "expected 1 or 2 values, got {}",
            parts.len()
        )));
    }

    let count: u32 = parts[0].parse().map_err(|_| {
        TimingFunctionError::InvalidSteps(format!("invalid step count: {}", parts[0]))
    })?;

    if count == 0 {
        return Err(TimingFunctionError::InvalidSteps("step count must be at least 1".to_string()));
    }

    let position = if parts.len() == 2 {
        parse_step_position(parts[1])?
    } else {
        StepPosition::JumpEnd // CSS default
    };

    // Validate jump-none requires count >= 2
    if position == StepPosition::JumpNone && count < 2 {
        return Err(TimingFunctionError::InvalidSteps(
            "jump-none requires at least 2 steps".to_string(),
        ));
    }

    Ok(Interpolation::Steps { count, position })
}

/// Parse step position keyword
fn parse_step_position(s: &str) -> Result<StepPosition, TimingFunctionError> {
    match s.to_lowercase().as_str() {
        "jump-start" | "start" => Ok(StepPosition::JumpStart),
        "jump-end" | "end" => Ok(StepPosition::JumpEnd),
        "jump-none" => Ok(StepPosition::JumpNone),
        "jump-both" => Ok(StepPosition::JumpBoth),
        _ => Err(TimingFunctionError::InvalidSteps(format!("unknown step position: {}", s))),
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
        let bezier = Interpolation::Bezier { p1: (0.25, 0.1), p2: (0.25, 1.0) };
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
        let keyframes = vec![ControlPoint::new(0.0, 0.0), ControlPoint::new(100.0, 50.0)];

        let mid = interpolate_path(&keyframes, 0.5, &MotionPath::Linear, &Interpolation::Linear);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_path_arc() {
        let keyframes = vec![ControlPoint::new(0.0, 0.0), ControlPoint::new(100.0, 0.0)];

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
        let keyframes = vec![(0, Point2D::new(0.0, 0.0)), (10, Point2D::new(100.0, 50.0))];

        let frames =
            generate_motion_frames(&keyframes, 11, &MotionPath::Linear, &Interpolation::Linear);
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
        let keyframes = vec![(0, Point2D::new(0.0, 0.0)), (10, Point2D::new(100.0, 0.0))];

        let frames =
            generate_motion_frames(&keyframes, 11, &MotionPath::Arc, &Interpolation::Linear);
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

    // ========================================================================
    // CSS Timing Function Tests (CSS-8)
    // ========================================================================

    #[test]
    fn test_step_position_default() {
        assert_eq!(StepPosition::default(), StepPosition::JumpEnd);
    }

    #[test]
    fn test_step_position_display() {
        assert_eq!(format!("{}", StepPosition::JumpStart), "jump-start");
        assert_eq!(format!("{}", StepPosition::JumpEnd), "jump-end");
        assert_eq!(format!("{}", StepPosition::JumpNone), "jump-none");
        assert_eq!(format!("{}", StepPosition::JumpBoth), "jump-both");
    }

    #[test]
    fn test_ease_steps_jump_end() {
        // steps(4, jump-end): outputs 0, 0.25, 0.5, 0.75 at intervals
        let steps = Interpolation::Steps { count: 4, position: StepPosition::JumpEnd };

        assert!((ease(0.0, &steps) - 0.0).abs() < 0.001);
        assert!((ease(0.24, &steps) - 0.0).abs() < 0.001);
        assert!((ease(0.25, &steps) - 0.25).abs() < 0.001);
        assert!((ease(0.49, &steps) - 0.25).abs() < 0.001);
        assert!((ease(0.5, &steps) - 0.5).abs() < 0.001);
        assert!((ease(0.99, &steps) - 0.75).abs() < 0.001);
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_steps_jump_start() {
        // steps(4, jump-start): outputs 0.25, 0.5, 0.75, 1.0 at intervals
        let steps = Interpolation::Steps { count: 4, position: StepPosition::JumpStart };

        assert!((ease(0.0, &steps) - 0.0).abs() < 0.001);
        assert!((ease(0.01, &steps) - 0.25).abs() < 0.001);
        assert!((ease(0.25, &steps) - 0.25).abs() < 0.001);
        assert!((ease(0.26, &steps) - 0.5).abs() < 0.001);
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_steps_jump_none() {
        // steps(4, jump-none): outputs 0, 0.33, 0.67, 1.0
        let steps = Interpolation::Steps { count: 4, position: StepPosition::JumpNone };

        assert!((ease(0.0, &steps) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_steps_jump_both() {
        // steps(4, jump-both): outputs 0.2, 0.4, 0.6, 0.8 initially, then 1.0
        let steps = Interpolation::Steps { count: 4, position: StepPosition::JumpBoth };

        // At t=0, should be 1/5 = 0.2
        assert!((ease(0.0, &steps) - 0.2).abs() < 0.001);
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_step_start_step_end() {
        // step-start = steps(1, jump-start)
        let step_start = Interpolation::Steps { count: 1, position: StepPosition::JumpStart };
        assert!((ease(0.0, &step_start) - 0.0).abs() < 0.001);
        assert!((ease(0.01, &step_start) - 1.0).abs() < 0.001);

        // step-end = steps(1, jump-end)
        let step_end = Interpolation::Steps { count: 1, position: StepPosition::JumpEnd };
        assert!((ease(0.0, &step_end) - 0.0).abs() < 0.001);
        assert!((ease(0.99, &step_end) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &step_end) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_timing_function_named() {
        assert_eq!(parse_timing_function("linear").unwrap(), Interpolation::Linear);
        assert_eq!(parse_timing_function("ease").unwrap(), Interpolation::EaseInOut);
        assert_eq!(parse_timing_function("ease-in").unwrap(), Interpolation::EaseIn);
        assert_eq!(parse_timing_function("ease-out").unwrap(), Interpolation::EaseOut);
        assert_eq!(parse_timing_function("ease-in-out").unwrap(), Interpolation::EaseInOut);
        assert_eq!(parse_timing_function("bounce").unwrap(), Interpolation::Bounce);
        assert_eq!(parse_timing_function("elastic").unwrap(), Interpolation::Elastic);
    }

    #[test]
    fn test_parse_timing_function_step_keywords() {
        assert_eq!(
            parse_timing_function("step-start").unwrap(),
            Interpolation::Steps { count: 1, position: StepPosition::JumpStart }
        );
        assert_eq!(
            parse_timing_function("step-end").unwrap(),
            Interpolation::Steps { count: 1, position: StepPosition::JumpEnd }
        );
    }

    #[test]
    fn test_parse_timing_function_cubic_bezier() {
        let result = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)").unwrap();
        match result {
            Interpolation::Bezier { p1, p2 } => {
                assert!((p1.0 - 0.25).abs() < 0.001);
                assert!((p1.1 - 0.1).abs() < 0.001);
                assert!((p2.0 - 0.25).abs() < 0.001);
                assert!((p2.1 - 1.0).abs() < 0.001);
            }
            _ => panic!("Expected Bezier interpolation"),
        }
    }

    #[test]
    fn test_parse_timing_function_cubic_bezier_spaces() {
        // Should handle extra whitespace
        let result = parse_timing_function("cubic-bezier( 0.42 , 0 , 0.58 , 1 )").unwrap();
        match result {
            Interpolation::Bezier { p1, p2 } => {
                assert!((p1.0 - 0.42).abs() < 0.001);
                assert!((p1.1 - 0.0).abs() < 0.001);
                assert!((p2.0 - 0.58).abs() < 0.001);
                assert!((p2.1 - 1.0).abs() < 0.001);
            }
            _ => panic!("Expected Bezier interpolation"),
        }
    }

    #[test]
    fn test_parse_timing_function_cubic_bezier_errors() {
        // Wrong number of arguments
        assert!(parse_timing_function("cubic-bezier(0.25, 0.1, 0.25)").is_err());

        // Invalid x values (out of [0, 1])
        assert!(parse_timing_function("cubic-bezier(-0.1, 0.1, 0.25, 1.0)").is_err());
        assert!(parse_timing_function("cubic-bezier(0.25, 0.1, 1.5, 1.0)").is_err());

        // Non-numeric values
        assert!(parse_timing_function("cubic-bezier(a, 0.1, 0.25, 1.0)").is_err());
    }

    #[test]
    fn test_parse_timing_function_steps() {
        let result = parse_timing_function("steps(4)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpEnd });

        let result = parse_timing_function("steps(4, jump-start)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpStart });

        let result = parse_timing_function("steps(4, jump-end)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpEnd });

        let result = parse_timing_function("steps(4, jump-none)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpNone });

        let result = parse_timing_function("steps(4, jump-both)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpBoth });
    }

    #[test]
    fn test_parse_timing_function_steps_short_positions() {
        // CSS also accepts "start" and "end" as shorthand
        let result = parse_timing_function("steps(4, start)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpStart });

        let result = parse_timing_function("steps(4, end)").unwrap();
        assert_eq!(result, Interpolation::Steps { count: 4, position: StepPosition::JumpEnd });
    }

    #[test]
    fn test_parse_timing_function_steps_errors() {
        // Step count must be at least 1
        assert!(parse_timing_function("steps(0)").is_err());

        // jump-none requires at least 2 steps
        assert!(parse_timing_function("steps(1, jump-none)").is_err());
        assert!(parse_timing_function("steps(2, jump-none)").is_ok());

        // Invalid step count
        assert!(parse_timing_function("steps(abc)").is_err());

        // Unknown step position
        assert!(parse_timing_function("steps(4, unknown)").is_err());
    }

    #[test]
    fn test_parse_timing_function_errors() {
        // Empty input
        assert!(parse_timing_function("").is_err());

        // Unknown function
        assert!(parse_timing_function("unknown-function").is_err());
        assert!(parse_timing_function("unknown(1, 2)").is_err());

        // Missing parenthesis
        assert!(parse_timing_function("steps(4").is_err());
    }

    #[test]
    fn test_parse_timing_function_case_insensitive() {
        assert_eq!(parse_timing_function("LINEAR").unwrap(), Interpolation::Linear);
        assert_eq!(parse_timing_function("EASE-IN-OUT").unwrap(), Interpolation::EaseInOut);
        assert_eq!(
            parse_timing_function("Cubic-Bezier(0.25, 0.1, 0.25, 1.0)").unwrap(),
            Interpolation::Bezier { p1: (0.25, 0.1), p2: (0.25, 1.0) }
        );
    }

    #[test]
    fn test_timing_function_error_display() {
        let err = TimingFunctionError::Empty;
        assert_eq!(format!("{}", err), "empty timing function");

        let err = TimingFunctionError::UnknownFunction("foo".to_string());
        assert_eq!(format!("{}", err), "unknown timing function: foo");

        let err = TimingFunctionError::InvalidBezier("bad value".to_string());
        assert_eq!(format!("{}", err), "invalid cubic-bezier: bad value");

        let err = TimingFunctionError::InvalidSteps("bad count".to_string());
        assert_eq!(format!("{}", err), "invalid steps: bad count");

        let err = TimingFunctionError::Syntax("missing paren".to_string());
        assert_eq!(format!("{}", err), "syntax error: missing paren");
    }

    // ========================================================================
    // Bounce and Elastic Easing Tests (CSS-10)
    // ========================================================================

    #[test]
    fn test_ease_bounce() {
        // Bounce easing should start at 0 and end at 1
        assert!((ease(0.0, &Interpolation::Bounce) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &Interpolation::Bounce) - 1.0).abs() < 0.001);

        // Bounce is a "bounce in" style - fast at start, bouncing effect at end
        // The mid value varies based on implementation
        let mid = ease(0.5, &Interpolation::Bounce);
        assert!(mid >= 0.0 && mid <= 1.0, "Bounce mid should be in valid range");

        // Values should stay in reasonable range (no extreme overshoot)
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let val = ease(t, &Interpolation::Bounce);
            assert!(val >= -0.1 && val <= 1.1, "Bounce at t={} gave {}", t, val);
        }
    }

    #[test]
    fn test_ease_elastic() {
        // Elastic easing should start at 0 and end at 1
        assert!((ease(0.0, &Interpolation::Elastic) - 0.0).abs() < 0.001);
        assert!((ease(1.0, &Interpolation::Elastic) - 1.0).abs() < 0.001);

        // Elastic overshoots - at some point before t=1 the value should be > 1
        let mut has_overshoot = false;
        for i in 1..100 {
            let t = i as f64 / 100.0;
            let val = ease(t, &Interpolation::Elastic);
            if val > 1.0 || val < 0.0 {
                has_overshoot = true;
                break;
            }
        }
        assert!(has_overshoot, "Elastic easing should overshoot");
    }

    // ========================================================================
    // Cubic Bezier Edge Cases (CSS-10)
    // ========================================================================

    #[test]
    fn test_ease_bezier_overshoot() {
        // CSS allows y values outside [0,1] for spring-like effects
        // Use a more aggressive overshoot curve
        let bezier = Interpolation::Bezier {
            p1: (0.2, 2.0),  // High y1 creates strong overshoot early
            p2: (0.8, 1.0),
        };

        // Should still start and end correctly
        assert!((ease(0.0, &bezier) - 0.0).abs() < 0.01);
        assert!((ease(1.0, &bezier) - 1.0).abs() < 0.01);

        // Should have values > 1 at some point (overshoot)
        let mut max_val = 0.0;
        for i in 1..100 {
            let t = i as f64 / 100.0;
            let val = ease(t, &bezier);
            if val > max_val {
                max_val = val;
            }
        }
        // With y1=2.0, we should see values exceeding 1.0
        assert!(max_val > 1.0, "Bezier with y1=2.0 should overshoot, max was {}", max_val);
    }

    #[test]
    fn test_ease_bezier_undershoot() {
        // y values < 0 create anticipation/undershoot
        let bezier = Interpolation::Bezier {
            p1: (0.5, -0.5),  // y < 0 creates undershoot
            p2: (0.5, 0.5),
        };

        assert!((ease(0.0, &bezier) - 0.0).abs() < 0.01);
        assert!((ease(1.0, &bezier) - 1.0).abs() < 0.01);

        // Should have values < 0 at some point (undershoot)
        let mut has_undershoot = false;
        for i in 1..100 {
            let t = i as f64 / 100.0;
            let val = ease(t, &bezier);
            if val < 0.0 {
                has_undershoot = true;
                break;
            }
        }
        assert!(has_undershoot, "Bezier with y1<0 should undershoot");
    }

    #[test]
    fn test_ease_bezier_standard_curves() {
        // CSS standard "ease" curve: cubic-bezier(0.25, 0.1, 0.25, 1.0)
        // This curve has a slight ease-in at start then accelerates
        let ease_curve = Interpolation::Bezier {
            p1: (0.25, 0.1),
            p2: (0.25, 1.0),
        };
        // The CSS "ease" starts slow but then accelerates quickly
        // At t=0.1, the output should be less than t (slow start)
        let very_early = ease(0.1, &ease_curve);
        assert!(very_early < 0.15, "ease curve should start slow, got {} at t=0.1", very_early);

        // CSS "ease-in-out" curve: cubic-bezier(0.42, 0, 0.58, 1)
        let ease_in_out_curve = Interpolation::Bezier {
            p1: (0.42, 0.0),
            p2: (0.58, 1.0),
        };
        // Should pass through ~0.5 at t=0.5 (symmetric curve)
        let mid = ease(0.5, &ease_in_out_curve);
        assert!((mid - 0.5).abs() < 0.15, "ease-in-out should be ~0.5 at midpoint, got {}", mid);
    }

    // ========================================================================
    // Steps Sprite Animation Semantics (CSS-10)
    // ========================================================================

    #[test]
    fn test_steps_sprite_animation_8_frames() {
        // Typical sprite sheet scenario: 8 frames, each frame shows for equal time
        // With steps(8, jump-end), frame indices are 0-7
        // Input t=0.0 to 1.0, output should map to frame indices 0-7
        let steps = Interpolation::Steps {
            count: 8,
            position: StepPosition::JumpEnd,
        };

        // Frame 0: t in [0, 0.125)
        assert!((ease(0.0, &steps) * 8.0).floor() == 0.0);
        assert!((ease(0.12, &steps) * 8.0).floor() == 0.0);

        // Frame 1: t in [0.125, 0.25)
        assert!((ease(0.125, &steps) * 8.0).floor() == 1.0);
        assert!((ease(0.24, &steps) * 8.0).floor() == 1.0);

        // Frame 7: t in [0.875, 1.0)
        assert!((ease(0.875, &steps) * 8.0).floor() == 7.0);
        assert!((ease(0.99, &steps) * 8.0).floor() == 7.0);

        // At exactly t=1.0, we get output 1.0 (end of last frame)
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_steps_sprite_animation_jump_start() {
        // With jump-start, the first visible frame is frame 1 (not 0)
        // Useful for animations where you don't want to show the "idle" frame
        let steps = Interpolation::Steps {
            count: 4,
            position: StepPosition::JumpStart,
        };

        // At t=0 (before animation starts), output is 0
        assert!((ease(0.0, &steps) - 0.0).abs() < 0.001);

        // Immediately after start, we're at first step (0.25)
        assert!((ease(0.001, &steps) - 0.25).abs() < 0.001);

        // We reach 1.0 (last frame) in the final interval
        assert!((ease(0.76, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_steps_jump_none_3_steps() {
        // jump-none with 3 steps gives outputs: 0, 0.5, 1.0
        // Useful when you want both ends to hold
        let steps = Interpolation::Steps {
            count: 3,
            position: StepPosition::JumpNone,
        };

        // 3 steps with jump-none = 2 intervals
        // Interval 1: [0, 0.5) -> output 0
        // Interval 2: [0.5, 1.0) -> output 0.5
        // At 1.0 -> output 1.0

        assert!((ease(0.0, &steps) - 0.0).abs() < 0.001);
        assert!((ease(0.49, &steps) - 0.0).abs() < 0.001);
        assert!((ease(0.5, &steps) - 0.5).abs() < 0.001);
        assert!((ease(0.99, &steps) - 0.5).abs() < 0.001);
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_steps_jump_both_3_steps() {
        // jump-both with 3 steps gives outputs: 0.25, 0.5, 0.75 in intervals, then 1.0
        // 3 intervals, 4 output values including start/end
        let steps = Interpolation::Steps {
            count: 3,
            position: StepPosition::JumpBoth,
        };

        // At t=0, we're already at first step (1/4 = 0.25)
        assert!((ease(0.0, &steps) - 0.25).abs() < 0.001);

        // At t=1, we reach 1.0
        assert!((ease(1.0, &steps) - 1.0).abs() < 0.001);
    }

    // ========================================================================
    // Interpolate Path Edge Cases (CSS-10)
    // ========================================================================

    #[test]
    fn test_interpolate_path_empty_keyframes() {
        let keyframes: Vec<ControlPoint> = vec![];
        let result = interpolate_path(&keyframes, 0.5, &MotionPath::Linear, &Interpolation::Linear);
        assert!((result.x - 0.0).abs() < 0.001);
        assert!((result.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_path_single_keyframe() {
        let keyframes = vec![ControlPoint::new(42.0, 17.0)];
        let result = interpolate_path(&keyframes, 0.5, &MotionPath::Linear, &Interpolation::Linear);
        assert!((result.x - 42.0).abs() < 0.001);
        assert!((result.y - 17.0).abs() < 0.001);
    }

    #[test]
    fn test_interpolate_path_bezier() {
        let keyframes = vec![
            ControlPoint::with_control(0.0, 0.0, 25.0, 50.0),
            ControlPoint::with_control(100.0, 0.0, 75.0, 50.0),
        ];

        let mid = interpolate_path(&keyframes, 0.5, &MotionPath::Bezier(vec![]), &Interpolation::Linear);

        // Should be at midpoint x-wise, with some y offset from control points
        assert!((mid.x - 50.0).abs() < 5.0);
        // Y should be influenced by control points (which are at y=50)
        assert!(mid.y > 0.0);
    }

    // ========================================================================
    // Interpolate Point Tests (CSS-10)
    // ========================================================================

    #[test]
    fn test_interpolate_point_with_easing() {
        let start = Point2D::new(0.0, 0.0);
        let end = Point2D::new(100.0, 100.0);

        // With ease-in, at t=0.5 we should be less than halfway
        let mid_ease_in = interpolate_point(&start, &end, 0.5, &Interpolation::EaseIn);
        assert!(mid_ease_in.x < 50.0);
        assert!(mid_ease_in.y < 50.0);

        // With ease-out, at t=0.5 we should be more than halfway
        let mid_ease_out = interpolate_point(&start, &end, 0.5, &Interpolation::EaseOut);
        assert!(mid_ease_out.x > 50.0);
        assert!(mid_ease_out.y > 50.0);

        // With linear, we should be exactly halfway
        let mid_linear = interpolate_point(&start, &end, 0.5, &Interpolation::Linear);
        assert!((mid_linear.x - 50.0).abs() < 0.001);
        assert!((mid_linear.y - 50.0).abs() < 0.001);
    }

    // ========================================================================
    // Clamping Behavior Tests (CSS-10)
    // ========================================================================

    #[test]
    fn test_ease_clamps_input() {
        // Values outside [0, 1] should be clamped
        assert!((ease(-0.5, &Interpolation::Linear) - 0.0).abs() < 0.001);
        assert!((ease(1.5, &Interpolation::Linear) - 1.0).abs() < 0.001);

        // Same for steps
        let steps = Interpolation::Steps {
            count: 4,
            position: StepPosition::JumpEnd,
        };
        assert!((ease(-0.5, &steps) - 0.0).abs() < 0.001);
        assert!((ease(1.5, &steps) - 1.0).abs() < 0.001);
    }

    // ========================================================================
    // Default Implementations Tests (CSS-10)
    // ========================================================================

    #[test]
    fn test_interpolation_default() {
        assert_eq!(Interpolation::default(), Interpolation::Linear);
    }

    #[test]
    fn test_motion_path_default() {
        assert_eq!(MotionPath::default(), MotionPath::Linear);
    }

    // ========================================================================
    // Control Point Tests (CSS-10)
    // ========================================================================

    #[test]
    fn test_control_point_new() {
        let cp = ControlPoint::new(10.0, 20.0);
        assert!((cp.position.x - 10.0).abs() < 0.001);
        assert!((cp.position.y - 20.0).abs() < 0.001);
        assert!(cp.control.is_none());
    }

    #[test]
    fn test_control_point_with_control() {
        let cp = ControlPoint::with_control(10.0, 20.0, 15.0, 30.0);
        assert!((cp.position.x - 10.0).abs() < 0.001);
        assert!((cp.position.y - 20.0).abs() < 0.001);
        assert!(cp.control.is_some());
        let ctrl = cp.control.unwrap();
        assert!((ctrl.x - 15.0).abs() < 0.001);
        assert!((ctrl.y - 30.0).abs() < 0.001);
    }

    // ========================================================================
    // Value Interpolation with Easing (CSS-10)
    // ========================================================================

    #[test]
    fn test_interpolate_value_with_easing() {
        // Opacity fade with ease-out (fast start, slow end)
        let start = 0.0;
        let end = 1.0;

        let mid_ease_out = interpolate_value(start, end, 0.5, &Interpolation::EaseOut);
        assert!(mid_ease_out > 0.5, "ease-out should be > 0.5 at midpoint");

        let mid_ease_in = interpolate_value(start, end, 0.5, &Interpolation::EaseIn);
        assert!(mid_ease_in < 0.5, "ease-in should be < 0.5 at midpoint");
    }

    // ========================================================================
    // Generate Motion Frames Edge Cases (CSS-10)
    // ========================================================================

    #[test]
    fn test_generate_motion_frames_empty() {
        let keyframes: Vec<(u32, Point2D)> = vec![];
        let frames = generate_motion_frames(&keyframes, 10, &MotionPath::Linear, &Interpolation::Linear);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_generate_motion_frames_zero_frames() {
        let keyframes = vec![(0, Point2D::new(0.0, 0.0))];
        let frames = generate_motion_frames(&keyframes, 0, &MotionPath::Linear, &Interpolation::Linear);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_generate_motion_frames_single_keyframe() {
        let keyframes = vec![(5, Point2D::new(50.0, 50.0))];
        let frames = generate_motion_frames(&keyframes, 10, &MotionPath::Linear, &Interpolation::Linear);
        assert_eq!(frames.len(), 10);

        // All frames should be at the single keyframe position
        for frame in &frames {
            assert!((frame.x - 50.0).abs() < 0.001);
            assert!((frame.y - 50.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_generate_motion_frames_with_easing() {
        let keyframes = vec![
            (0, Point2D::new(0.0, 0.0)),
            (10, Point2D::new(100.0, 0.0)),
        ];

        // With ease-in, motion should be slower at start
        let frames_ease_in = generate_motion_frames(&keyframes, 11, &MotionPath::Linear, &Interpolation::EaseIn);

        // Frame 2 (t=0.2) should have moved less than 20 pixels with ease-in
        assert!(frames_ease_in[2].x < 20.0);

        // With ease-out, motion should be faster at start
        let frames_ease_out = generate_motion_frames(&keyframes, 11, &MotionPath::Linear, &Interpolation::EaseOut);

        // Frame 2 (t=0.2) should have moved more than 20 pixels with ease-out
        assert!(frames_ease_out[2].x > 20.0);
    }

    // ========================================================================
    // CSS Timing Function Integration Tests (CSS-10)
    // ========================================================================

    #[test]
    fn test_parse_and_ease_cubic_bezier() {
        // Parse a cubic-bezier and verify it produces expected easing
        let interp = parse_timing_function("cubic-bezier(0, 0, 1, 1)").unwrap();

        // (0,0,1,1) should be approximately linear
        let mid = ease(0.5, &interp);
        assert!((mid - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_parse_and_ease_steps() {
        // Parse steps and verify easing behavior
        let interp = parse_timing_function("steps(5, jump-end)").unwrap();

        // Should have 5 discrete levels
        let v0 = ease(0.0, &interp);
        let v1 = ease(0.19, &interp);
        let v2 = ease(0.2, &interp);

        assert!((v0 - 0.0).abs() < 0.001);
        assert!((v1 - 0.0).abs() < 0.001);  // Still in first step
        assert!((v2 - 0.2).abs() < 0.001);  // Jumped to second step
    }

    #[test]
    fn test_steps_with_interpolate_value() {
        // Sprite frame selection: map animation progress to frame index
        let steps = Interpolation::Steps {
            count: 4,
            position: StepPosition::JumpEnd,
        };

        // Map [0, 1] to frame indices [0, 3]
        let frame_at_start = interpolate_value(0.0, 3.0, 0.0, &steps).round() as i32;
        let frame_at_quarter = interpolate_value(0.0, 3.0, 0.26, &steps).round() as i32;
        let frame_at_half = interpolate_value(0.0, 3.0, 0.51, &steps).round() as i32;
        let frame_at_end = interpolate_value(0.0, 3.0, 1.0, &steps).round() as i32;

        assert_eq!(frame_at_start, 0);
        assert_eq!(frame_at_quarter, 1);
        assert_eq!(frame_at_half, 2);
        assert_eq!(frame_at_end, 3);
    }
}
