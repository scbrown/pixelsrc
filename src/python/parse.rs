//! Parsing, listing, and formatting functions for the Python API.
//!
//! Exposes `.pxl` content inspection without rendering:
//! - `parse()` -- returns all parsed objects as Python dicts
//! - `list_sprites()` -- returns sprite names
//! - `list_palettes()` -- returns palette names
//! - `format_pxl()` -- formats `.pxl` content for readability

use std::io::Cursor;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::fmt::format_pixelsrc;
use crate::models::TtpObject;
use crate::parser::parse_stream;

/// Parse a `.pxl` string and return a list of parsed objects as dicts.
///
/// Each dict has a "type" key ("palette", "sprite", "variant", etc.)
/// plus all the fields for that object type, serialized via serde.
///
/// Parse warnings are collected as dicts with "warning" and "line" keys
/// appended to the end of the list.
#[pyfunction]
pub fn parse(py: Python<'_>, pxl: &str) -> PyResult<Vec<PyObject>> {
    let result = parse_stream(Cursor::new(pxl));
    let mut objects = Vec::with_capacity(result.objects.len() + result.warnings.len());

    for obj in &result.objects {
        let json_value = serde_json::to_value(obj).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("serialization error: {e}"))
        })?;
        let py_obj = json_value_to_py(py, &json_value)?;
        objects.push(py_obj);
    }

    for warning in &result.warnings {
        let dict = PyDict::new(py);
        dict.set_item("warning", &warning.message)?;
        dict.set_item("line", warning.line)?;
        objects.push(dict.into_any().unbind());
    }

    Ok(objects)
}

/// Return a list of sprite names found in a `.pxl` string.
#[pyfunction]
pub fn list_sprites(pxl: &str) -> Vec<String> {
    let result = parse_stream(Cursor::new(pxl));
    result
        .objects
        .iter()
        .filter_map(|obj| match obj {
            TtpObject::Sprite(s) => Some(s.name.clone()),
            _ => None,
        })
        .collect()
}

/// Return a list of palette names found in a `.pxl` string.
#[pyfunction]
pub fn list_palettes(pxl: &str) -> Vec<String> {
    let result = parse_stream(Cursor::new(pxl));
    result
        .objects
        .iter()
        .filter_map(|obj| match obj {
            TtpObject::Palette(p) => Some(p.name.clone()),
            _ => None,
        })
        .collect()
}

/// Format a `.pxl` string for readability.
///
/// Parses the input and reformats each object:
/// - Sprites: grid arrays expanded to one row per line
/// - Compositions: layer maps expanded to one row per line
/// - Palettes, Animations, Variants: kept as single-line JSON
///
/// Returns the formatted string, or raises an error on parse failure.
#[pyfunction]
pub fn format_pxl(pxl: &str) -> PyResult<String> {
    format_pixelsrc(pxl).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
}

/// Convert a `serde_json::Value` into a Python object.
fn json_value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(pyo3::types::PyBool::new(py, *b).to_owned().into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        serde_json::Value::Array(arr) => {
            let py_list = pyo3::types::PyList::empty(py);
            for item in arr {
                py_list.append(json_value_to_py(py, item)?)?;
            }
            Ok(py_list.into_any().unbind())
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_value_to_py(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_sprites_basic() {
        let pxl = r##"{"type": "palette", "name": "pal", "colors": {"x": "#FF0000"}}
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "pal", "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;
        let names = list_sprites(pxl);
        assert_eq!(names, vec!["hero"]);
    }

    #[test]
    fn test_list_palettes_basic() {
        let pxl = r##"{"type": "palette", "name": "warm", "colors": {"x": "#FF0000"}}
{"type": "palette", "name": "cool", "colors": {"y": "#0000FF"}}
{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "warm", "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;
        let names = list_palettes(pxl);
        assert_eq!(names, vec!["warm", "cool"]);
    }

    #[test]
    fn test_list_sprites_empty() {
        let pxl = r##"{"type": "palette", "name": "pal", "colors": {}}"##;
        let names = list_sprites(pxl);
        assert!(names.is_empty());
    }

    #[test]
    fn test_list_palettes_empty() {
        let pxl = r##"{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;
        let names = list_palettes(pxl);
        assert!(names.is_empty());
    }

    #[test]
    fn test_list_sprites_multiple() {
        let pxl = r##"{"type": "sprite", "name": "a", "size": [1, 1], "palette": {"x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}
{"type": "sprite", "name": "b", "size": [1, 1], "palette": {"x": "#00FF00"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}"##;
        let names = list_sprites(pxl);
        assert_eq!(names, vec!["a", "b"]);
    }
}
