//! User-defined transform support with expression evaluation (TRF-10)
//!
//! Provides expression evaluation for keyframe animations and dynamic transforms.

use std::collections::HashMap;

use super::parsing::{parse_transform_str, parse_transform_value};
use super::types::{Transform, TransformError};

/// Error type for expression evaluation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExpressionError {
    /// Unknown variable in expression
    #[error("unknown variable: {0}")]
    UnknownVariable(String),
    /// Unknown function in expression
    #[error("unknown function: {0}")]
    UnknownFunction(String),
    /// Invalid syntax
    #[error("syntax error: {0}")]
    SyntaxError(String),
    /// Division by zero
    #[error("division by zero")]
    DivisionByZero,
    /// Invalid number of arguments
    #[error("function {func} expected {expected} arguments, got {got}")]
    WrongArity { func: String, expected: usize, got: usize },
}

/// Simple expression evaluator for keyframe animations.
///
/// Supports:
/// - Variables: `frame`, `t`, `total_frames`, and user-defined params
/// - Operators: `+`, `-`, `*`, `/`, `%`, `^` (power)
/// - Functions: `sin`, `cos`, `tan`, `pow`, `sqrt`, `min`, `max`, `abs`, `floor`, `ceil`, `round`
/// - Parentheses for grouping
/// - Parameter substitution with `${param_name}` syntax
pub struct ExpressionEvaluator {
    variables: HashMap<String, f64>,
}

impl ExpressionEvaluator {
    /// Create a new evaluator with the given variables.
    pub fn new(variables: HashMap<String, f64>) -> Self {
        Self { variables }
    }

    /// Create an evaluator for keyframe animation with standard variables.
    ///
    /// Sets up `frame`, `t` (normalized 0.0-1.0), and `total_frames`.
    pub fn for_keyframe(frame: u32, total_frames: u32) -> Self {
        let mut vars = HashMap::new();
        vars.insert("frame".to_string(), frame as f64);
        vars.insert("total_frames".to_string(), total_frames as f64);
        let t = if total_frames > 1 { frame as f64 / (total_frames - 1) as f64 } else { 0.0 };
        vars.insert("t".to_string(), t);
        Self { variables: vars }
    }

    /// Add a variable to the evaluator.
    pub fn with_var(mut self, name: &str, value: f64) -> Self {
        self.variables.insert(name.to_string(), value);
        self
    }

    /// Add multiple variables from a map.
    pub fn with_vars(mut self, vars: &HashMap<String, f64>) -> Self {
        for (k, v) in vars {
            self.variables.insert(k.clone(), *v);
        }
        self
    }

    /// Substitute `${param}` placeholders in the expression.
    fn substitute_params(&self, expr: &str) -> String {
        let mut result = expr.to_string();
        for (name, value) in &self.variables {
            let placeholder = format!("${{{}}}", name);
            result = result.replace(&placeholder, &value.to_string());
        }
        result
    }

    /// Evaluate an expression string.
    pub fn evaluate(&self, expr: &str) -> Result<f64, ExpressionError> {
        let expr = self.substitute_params(expr);
        self.parse_expression(&expr)
    }

    /// Parse and evaluate an expression.
    fn parse_expression(&self, expr: &str) -> Result<f64, ExpressionError> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(ExpressionError::SyntaxError("empty expression".to_string()));
        }

        // Try parsing as a simple number first
        if let Ok(n) = expr.parse::<f64>() {
            return Ok(n);
        }

        // Check for variable reference
        if expr.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return self
                .variables
                .get(expr)
                .copied()
                .ok_or_else(|| ExpressionError::UnknownVariable(expr.to_string()));
        }

        // Handle parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            let inner = &expr[1..expr.len() - 1];
            if self.count_parens(inner) == 0 {
                return self.parse_expression(inner);
            }
        }

        // Handle binary operators (lowest precedence first)
        // We scan right-to-left for left-associativity
        for ops in &[&['+', '-'][..], &['*', '/', '%'][..]] {
            let mut paren_depth = 0;
            let chars: Vec<char> = expr.chars().collect();
            for i in (0..chars.len()).rev() {
                match chars[i] {
                    ')' => paren_depth += 1,
                    '(' => paren_depth -= 1,
                    c if paren_depth == 0 && ops.contains(&c) => {
                        // Handle negative numbers at start
                        if i == 0 && c == '-' {
                            continue;
                        }
                        // Check if this is a binary operator (not unary minus)
                        if i > 0 {
                            let prev = chars[i - 1];
                            if c == '-'
                                && (prev == '('
                                    || prev == '+'
                                    || prev == '-'
                                    || prev == '*'
                                    || prev == '/'
                                    || prev == '%'
                                    || prev == '^'
                                    || prev == ',')
                            {
                                continue;
                            }
                        }
                        let left = &expr[..i];
                        let right = &expr[i + 1..];
                        if !left.is_empty() && !right.is_empty() {
                            let l = self.parse_expression(left)?;
                            let r = self.parse_expression(right)?;
                            return match c {
                                '+' => Ok(l + r),
                                '-' => Ok(l - r),
                                '*' => Ok(l * r),
                                '/' => {
                                    if r == 0.0 {
                                        Err(ExpressionError::DivisionByZero)
                                    } else {
                                        Ok(l / r)
                                    }
                                }
                                '%' => {
                                    if r == 0.0 {
                                        Err(ExpressionError::DivisionByZero)
                                    } else {
                                        Ok(l % r)
                                    }
                                }
                                _ => unreachable!(),
                            };
                        }
                    }
                    _ => {}
                }
            }
        }

        // Handle power operator (^) - right-to-left
        {
            let mut paren_depth = 0;
            let chars: Vec<char> = expr.chars().collect();
            for i in 0..chars.len() {
                match chars[i] {
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    '^' if paren_depth == 0 => {
                        let left = &expr[..i];
                        let right = &expr[i + 1..];
                        if !left.is_empty() && !right.is_empty() {
                            let l = self.parse_expression(left)?;
                            let r = self.parse_expression(right)?;
                            return Ok(l.powf(r));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Handle function calls
        if let Some(paren_pos) = expr.find('(') {
            if expr.ends_with(')') {
                let func_name = expr[..paren_pos].trim();
                let args_str = &expr[paren_pos + 1..expr.len() - 1];
                let args = self.parse_args(args_str)?;
                return self.call_function(func_name, &args);
            }
        }

        // Handle unary minus at start
        if let Some(inner) = expr.strip_prefix('-') {
            return Ok(-self.parse_expression(inner)?);
        }

        Err(ExpressionError::SyntaxError(format!("cannot parse: {}", expr)))
    }

    /// Count unmatched parentheses (positive = more opens, negative = more closes).
    fn count_parens(&self, s: &str) -> i32 {
        let mut count = 0;
        for c in s.chars() {
            match c {
                '(' => count += 1,
                ')' => count -= 1,
                _ => {}
            }
        }
        count
    }

    /// Parse function arguments, handling nested parentheses.
    fn parse_args(&self, args_str: &str) -> Result<Vec<f64>, ExpressionError> {
        if args_str.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut args = Vec::new();
        let mut current = String::new();
        let mut paren_depth = 0;

        for c in args_str.chars() {
            match c {
                '(' => {
                    paren_depth += 1;
                    current.push(c);
                }
                ')' => {
                    paren_depth -= 1;
                    current.push(c);
                }
                ',' if paren_depth == 0 => {
                    args.push(self.parse_expression(&current)?);
                    current.clear();
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            args.push(self.parse_expression(&current)?);
        }

        Ok(args)
    }

    /// Call a built-in function with the given arguments.
    fn call_function(&self, name: &str, args: &[f64]) -> Result<f64, ExpressionError> {
        match name.to_lowercase().as_str() {
            "sin" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].sin())
            }
            "cos" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].cos())
            }
            "tan" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].tan())
            }
            "pow" => {
                self.check_arity(name, args, 2)?;
                Ok(args[0].powf(args[1]))
            }
            "sqrt" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].sqrt())
            }
            "min" => {
                self.check_arity(name, args, 2)?;
                Ok(args[0].min(args[1]))
            }
            "max" => {
                self.check_arity(name, args, 2)?;
                Ok(args[0].max(args[1]))
            }
            "abs" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].abs())
            }
            "floor" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].floor())
            }
            "ceil" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].ceil())
            }
            "round" => {
                self.check_arity(name, args, 1)?;
                Ok(args[0].round())
            }
            "clamp" => {
                self.check_arity(name, args, 3)?;
                Ok(args[0].clamp(args[1], args[2]))
            }
            _ => Err(ExpressionError::UnknownFunction(name.to_string())),
        }
    }

    fn check_arity(
        &self,
        name: &str,
        args: &[f64],
        expected: usize,
    ) -> Result<(), ExpressionError> {
        if args.len() != expected {
            Err(ExpressionError::WrongArity { func: name.to_string(), expected, got: args.len() })
        } else {
            Ok(())
        }
    }
}

/// Interpolate between keyframes for a given frame number.
///
/// Given a list of (frame, value) keyframes, interpolates the value at the specified frame
/// using the provided easing function.
pub fn interpolate_keyframes(
    keyframes: &[[f64; 2]],
    frame: f64,
    easing: &crate::models::Easing,
) -> f64 {
    if keyframes.is_empty() {
        return 0.0;
    }

    if keyframes.len() == 1 {
        return keyframes[0][1];
    }

    // Sort keyframes by frame number
    let mut sorted: Vec<[f64; 2]> = keyframes.to_vec();
    sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

    // Before first keyframe
    if frame <= sorted[0][0] {
        return sorted[0][1];
    }

    // After last keyframe
    if frame >= sorted[sorted.len() - 1][0] {
        return sorted[sorted.len() - 1][1];
    }

    // Find surrounding keyframes
    for i in 0..sorted.len() - 1 {
        let (f1, v1) = (sorted[i][0], sorted[i][1]);
        let (f2, v2) = (sorted[i + 1][0], sorted[i + 1][1]);

        if frame >= f1 && frame <= f2 {
            // Calculate normalized time between keyframes
            let t = if (f2 - f1).abs() < f64::EPSILON { 0.0 } else { (frame - f1) / (f2 - f1) };

            // Apply easing
            let eased_t = easing.apply(t);

            // Linear interpolation with eased t
            return v1 + (v2 - v1) * eased_t;
        }
    }

    // Fallback
    sorted[sorted.len() - 1][1]
}

/// Generate transforms for a specific frame from a user-defined transform.
///
/// This handles:
/// - Expression-based keyframes (evaluates the expression for the frame)
/// - Explicit keyframes (interpolates between them)
/// - Cycling transforms (picks the transform for this frame)
pub fn generate_frame_transforms(
    transform_def: &crate::models::TransformDef,
    frame: u32,
    total_frames: u32,
    params: &HashMap<String, f64>,
) -> Result<Vec<Transform>, TransformError> {
    // Handle simple ops-only transform
    if transform_def.is_simple() {
        if let Some(ops) = &transform_def.ops {
            return ops.iter().map(parse_transform_spec_internal).collect();
        }
        return Ok(vec![]);
    }

    // Handle cycling transforms
    if transform_def.is_cycling() {
        if let Some(cycle) = &transform_def.cycle {
            let cycle_len = cycle.len();
            if cycle_len > 0 {
                let cycle_index = (frame as usize) % cycle_len;
                return cycle[cycle_index].iter().map(parse_transform_spec_internal).collect();
            }
        }
        return Ok(vec![]);
    }

    // Handle keyframe animation
    if transform_def.generates_animation() {
        let keyframes = transform_def.keyframes.as_ref().unwrap();
        let default_easing = transform_def.easing.clone().unwrap_or_default();

        let mut transforms = Vec::new();

        // Create evaluator with standard variables and user params
        let eval = ExpressionEvaluator::for_keyframe(frame, total_frames).with_vars(params);

        match keyframes {
            crate::models::KeyframeSpec::Array(kfs) => {
                // Collect all unique property names
                let mut property_values: HashMap<String, f64> = HashMap::new();

                for kf in kfs {
                    for prop in kf.values.keys() {
                        // Build keyframes for this property
                        let kf_pairs: Vec<[f64; 2]> = kfs
                            .iter()
                            .filter_map(|k| k.values.get(prop).map(|v| [k.frame as f64, *v]))
                            .collect();

                        let interpolated =
                            interpolate_keyframes(&kf_pairs, frame as f64, &default_easing);
                        property_values.insert(prop.clone(), interpolated);
                    }
                }

                // Convert property values to transforms
                transforms.extend(property_values_to_transforms(&property_values)?);
            }
            crate::models::KeyframeSpec::Properties(props) => {
                let mut property_values: HashMap<String, f64> = HashMap::new();

                for (prop, prop_kf) in props {
                    let easing = prop_kf.easing.as_ref().unwrap_or(&default_easing);

                    let value = if let Some(expr) = &prop_kf.expr {
                        // Evaluate expression
                        eval.evaluate(expr).map_err(|e| TransformError::InvalidParameter {
                            op: "keyframe".to_string(),
                            message: e.to_string(),
                        })?
                    } else if let Some(kfs) = &prop_kf.keyframes {
                        // Interpolate keyframes
                        interpolate_keyframes(kfs, frame as f64, easing)
                    } else {
                        0.0
                    };

                    property_values.insert(prop.clone(), value);
                }

                transforms.extend(property_values_to_transforms(&property_values)?);
            }
        }

        return Ok(transforms);
    }

    // Handle compose (parallel composition)
    if let Some(compose) = &transform_def.compose {
        return compose.iter().map(parse_transform_spec_internal).collect();
    }

    Ok(vec![])
}

/// Convert property name/value pairs to Transform enum variants.
fn property_values_to_transforms(
    properties: &HashMap<String, f64>,
) -> Result<Vec<Transform>, TransformError> {
    let mut transforms = Vec::new();

    // Handle shift properties
    let shift_x = properties.get("shift-x").or_else(|| properties.get("shift_x"));
    let shift_y = properties.get("shift-y").or_else(|| properties.get("shift_y"));

    if shift_x.is_some() || shift_y.is_some() {
        transforms.push(Transform::Shift {
            x: shift_x.map(|v| v.round() as i32).unwrap_or(0),
            y: shift_y.map(|v| v.round() as i32).unwrap_or(0),
        });
    }

    // Handle scale properties
    let scale_x = properties.get("scale-x").or_else(|| properties.get("scale_x"));
    let scale_y = properties.get("scale-y").or_else(|| properties.get("scale_y"));
    let scale = properties.get("scale");

    if scale_x.is_some() || scale_y.is_some() || scale.is_some() {
        let x = scale_x.or(scale).copied().unwrap_or(1.0) as f32;
        let y = scale_y.or(scale).copied().unwrap_or(1.0) as f32;
        transforms.push(Transform::Scale { x, y });
    }

    // Handle rotation
    if let Some(degrees) = properties.get("rotate").or_else(|| properties.get("rotation")) {
        let deg = degrees.round() as i32;
        // Normalize to 0, 90, 180, 270
        let normalized = ((deg % 360) + 360) % 360;
        if normalized == 90 || normalized == 180 || normalized == 270 {
            transforms.push(Transform::Rotate { degrees: normalized as u16 });
        }
    }

    // Handle pad
    if let Some(pad) = properties.get("pad").or_else(|| properties.get("padding")) {
        transforms.push(Transform::Pad { size: pad.max(0.0).round() as u32 });
    }

    // Handle subpixel
    let subpixel_x = properties.get("subpixel-x").or_else(|| properties.get("subpixel_x"));
    let subpixel_y = properties.get("subpixel-y").or_else(|| properties.get("subpixel_y"));

    if subpixel_x.is_some() || subpixel_y.is_some() {
        transforms.push(Transform::Subpixel {
            x: subpixel_x.copied().unwrap_or(0.0),
            y: subpixel_y.copied().unwrap_or(0.0),
        });
    }

    Ok(transforms)
}

/// Parse a TransformSpec into a Transform (internal version for this module).
fn parse_transform_spec_internal(
    spec: &crate::models::TransformSpec,
) -> Result<Transform, TransformError> {
    match spec {
        crate::models::TransformSpec::String(s) => parse_transform_str(s),
        crate::models::TransformSpec::Object { op, params } => {
            // Convert params to serde_json::Value object for parsing
            let mut obj = serde_json::Map::new();
            obj.insert("op".to_string(), serde_json::Value::String(op.clone()));
            for (k, v) in params {
                obj.insert(k.clone(), v.clone());
            }
            parse_transform_value(&serde_json::Value::Object(obj))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_evaluator_simple_number() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("42").unwrap(), 42.0);
        assert_eq!(eval.evaluate("3.14").unwrap(), 3.14);
        assert_eq!(eval.evaluate("-5").unwrap(), -5.0);
    }

    #[test]
    fn test_expression_evaluator_variables() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 10.0);
        vars.insert("y".to_string(), 20.0);
        let eval = ExpressionEvaluator::new(vars);

        assert_eq!(eval.evaluate("x").unwrap(), 10.0);
        assert_eq!(eval.evaluate("y").unwrap(), 20.0);
    }

    #[test]
    fn test_expression_evaluator_unknown_variable() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert!(eval.evaluate("unknown").is_err());
    }

    #[test]
    fn test_expression_evaluator_addition() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("2+3").unwrap(), 5.0);
        assert_eq!(eval.evaluate("10 + 20").unwrap(), 30.0);
    }

    #[test]
    fn test_expression_evaluator_subtraction() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("10-3").unwrap(), 7.0);
        assert_eq!(eval.evaluate("5 - 10").unwrap(), -5.0);
    }

    #[test]
    fn test_expression_evaluator_multiplication() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("3*4").unwrap(), 12.0);
        assert_eq!(eval.evaluate("2.5 * 4").unwrap(), 10.0);
    }

    #[test]
    fn test_expression_evaluator_division() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("10/2").unwrap(), 5.0);
        assert_eq!(eval.evaluate("7 / 2").unwrap(), 3.5);
    }

    #[test]
    fn test_expression_evaluator_division_by_zero() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert!(eval.evaluate("10/0").is_err());
    }

    #[test]
    fn test_expression_evaluator_modulo() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("10%3").unwrap(), 1.0);
        assert_eq!(eval.evaluate("7 % 2").unwrap(), 1.0);
    }

    #[test]
    fn test_expression_evaluator_power() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("2^3").unwrap(), 8.0);
        assert_eq!(eval.evaluate("3^2").unwrap(), 9.0);
    }

    #[test]
    fn test_expression_evaluator_parentheses() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("(2+3)*4").unwrap(), 20.0);
        assert_eq!(eval.evaluate("2*(3+4)").unwrap(), 14.0);
        assert_eq!(eval.evaluate("((2+3))").unwrap(), 5.0);
    }

    #[test]
    fn test_expression_evaluator_operator_precedence() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert_eq!(eval.evaluate("2+3*4").unwrap(), 14.0); // 2 + 12 = 14
        assert_eq!(eval.evaluate("10-2*3").unwrap(), 4.0); // 10 - 6 = 4
    }

    #[test]
    fn test_expression_evaluator_functions() {
        let eval = ExpressionEvaluator::new(HashMap::new());

        // abs
        assert_eq!(eval.evaluate("abs(-5)").unwrap(), 5.0);
        assert_eq!(eval.evaluate("abs(5)").unwrap(), 5.0);

        // min/max
        assert_eq!(eval.evaluate("min(3, 7)").unwrap(), 3.0);
        assert_eq!(eval.evaluate("max(3, 7)").unwrap(), 7.0);

        // floor/ceil/round
        assert_eq!(eval.evaluate("floor(3.7)").unwrap(), 3.0);
        assert_eq!(eval.evaluate("ceil(3.2)").unwrap(), 4.0);
        assert_eq!(eval.evaluate("round(3.5)").unwrap(), 4.0);

        // sqrt
        assert_eq!(eval.evaluate("sqrt(16)").unwrap(), 4.0);

        // pow
        assert_eq!(eval.evaluate("pow(2, 3)").unwrap(), 8.0);

        // clamp
        assert_eq!(eval.evaluate("clamp(5, 0, 10)").unwrap(), 5.0);
        assert_eq!(eval.evaluate("clamp(-5, 0, 10)").unwrap(), 0.0);
        assert_eq!(eval.evaluate("clamp(15, 0, 10)").unwrap(), 10.0);
    }

    #[test]
    fn test_expression_evaluator_trig() {
        let eval = ExpressionEvaluator::new(HashMap::new());

        // sin(0) = 0
        assert!((eval.evaluate("sin(0)").unwrap() - 0.0).abs() < 0.0001);

        // cos(0) = 1
        assert!((eval.evaluate("cos(0)").unwrap() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_expression_evaluator_unknown_function() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert!(eval.evaluate("unknown_func(5)").is_err());
    }

    #[test]
    fn test_expression_evaluator_wrong_arity() {
        let eval = ExpressionEvaluator::new(HashMap::new());
        assert!(eval.evaluate("abs(1, 2)").is_err());
        assert!(eval.evaluate("min(1)").is_err());
    }

    #[test]
    fn test_expression_evaluator_parameter_substitution() {
        let mut vars = HashMap::new();
        vars.insert("scale".to_string(), 2.0);
        let eval = ExpressionEvaluator::new(vars);

        assert_eq!(eval.evaluate("${scale}*10").unwrap(), 20.0);
    }

    #[test]
    fn test_expression_evaluator_for_keyframe() {
        let eval = ExpressionEvaluator::for_keyframe(5, 10);

        assert_eq!(eval.evaluate("frame").unwrap(), 5.0);
        assert_eq!(eval.evaluate("total_frames").unwrap(), 10.0);
        // t = 5 / (10-1) = 5/9 ≈ 0.555...
        assert!((eval.evaluate("t").unwrap() - 5.0 / 9.0).abs() < 0.0001);
    }

    #[test]
    fn test_expression_evaluator_complex_expression() {
        let eval = ExpressionEvaluator::for_keyframe(5, 10);

        // frame * 2 + 3 = 5 * 2 + 3 = 13
        assert_eq!(eval.evaluate("frame * 2 + 3").unwrap(), 13.0);

        // sin(t * 3.14159) at t=5/9 ≈ sin(0.555 * π) ≈ 0.985
        let result = eval.evaluate("sin(t * 3.14159)").unwrap();
        assert!(result > 0.5 && result < 1.0);
    }

    #[test]
    fn test_expression_evaluator_negative_numbers() {
        let eval = ExpressionEvaluator::new(HashMap::new());

        assert_eq!(eval.evaluate("-5 + 3").unwrap(), -2.0);
        assert_eq!(eval.evaluate("3 + (-5)").unwrap(), -2.0); // Use parens for negative
        assert_eq!(eval.evaluate("3 * (-2)").unwrap(), -6.0); // Use parens for negative
        assert_eq!(eval.evaluate("(-3) * 2").unwrap(), -6.0);
    }

    #[test]
    fn test_interpolate_keyframes_empty() {
        let keyframes: Vec<[f64; 2]> = vec![];
        let easing = crate::models::Easing::default();
        assert_eq!(interpolate_keyframes(&keyframes, 5.0, &easing), 0.0);
    }

    #[test]
    fn test_interpolate_keyframes_single() {
        let keyframes = vec![[0.0, 100.0]];
        let easing = crate::models::Easing::default();
        assert_eq!(interpolate_keyframes(&keyframes, 5.0, &easing), 100.0);
    }

    #[test]
    fn test_interpolate_keyframes_before_first() {
        let keyframes = vec![[5.0, 100.0], [10.0, 200.0]];
        let easing = crate::models::Easing::default();
        assert_eq!(interpolate_keyframes(&keyframes, 0.0, &easing), 100.0);
    }

    #[test]
    fn test_interpolate_keyframes_after_last() {
        let keyframes = vec![[5.0, 100.0], [10.0, 200.0]];
        let easing = crate::models::Easing::default();
        assert_eq!(interpolate_keyframes(&keyframes, 15.0, &easing), 200.0);
    }

    #[test]
    fn test_interpolate_keyframes_at_keyframe() {
        let keyframes = vec![[0.0, 100.0], [10.0, 200.0]];
        let easing = crate::models::Easing::default();
        assert_eq!(interpolate_keyframes(&keyframes, 0.0, &easing), 100.0);
        assert_eq!(interpolate_keyframes(&keyframes, 10.0, &easing), 200.0);
    }

    #[test]
    fn test_interpolate_keyframes_midpoint() {
        let keyframes = vec![[0.0, 100.0], [10.0, 200.0]];
        let easing = crate::models::Easing::default();
        // Midpoint should be 150 with linear easing
        assert_eq!(interpolate_keyframes(&keyframes, 5.0, &easing), 150.0);
    }

    #[test]
    fn test_property_values_to_transforms_shift() {
        let mut props = HashMap::new();
        props.insert("shift-x".to_string(), 10.5);
        props.insert("shift-y".to_string(), -5.2);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Shift { x: 11, y: -5 }));
    }

    #[test]
    fn test_property_values_to_transforms_scale() {
        let mut props = HashMap::new();
        props.insert("scale-x".to_string(), 2.0);
        props.insert("scale-y".to_string(), 0.5);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Scale { x, y } if (x - 2.0).abs() < 0.001 && (y - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_property_values_to_transforms_uniform_scale() {
        let mut props = HashMap::new();
        props.insert("scale".to_string(), 3.0);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Scale { x, y } if (x - 3.0).abs() < 0.001 && (y - 3.0).abs() < 0.001));
    }

    #[test]
    fn test_property_values_to_transforms_rotate() {
        let mut props = HashMap::new();
        props.insert("rotate".to_string(), 90.0);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Rotate { degrees: 90 }));
    }

    #[test]
    fn test_property_values_to_transforms_subpixel() {
        let mut props = HashMap::new();
        props.insert("subpixel-x".to_string(), 0.5);
        props.insert("subpixel-y".to_string(), 0.25);

        let transforms = property_values_to_transforms(&props).unwrap();
        assert_eq!(transforms.len(), 1);
        assert!(matches!(transforms[0], Transform::Subpixel { x, y } if (x - 0.5).abs() < 0.001 && (y - 0.25).abs() < 0.001));
    }

    #[test]
    fn test_property_values_to_transforms_empty() {
        let props = HashMap::new();
        let transforms = property_values_to_transforms(&props).unwrap();
        assert!(transforms.is_empty());
    }
}
