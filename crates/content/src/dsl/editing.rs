//! Source-preserving option edits and exact numeric input settlement for Studio.

use super::{
    configuration::failure,
    number::{canonical_decimal, input_decimal},
    validation::Diagnostic,
    *,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const ALLOWED: [&str; 6] = [
    "iteration",
    "duration",
    "seed",
    "crit",
    "start_fons",
    "stop",
];

fn same_value(left: Option<&Value>, right: &Value) -> bool {
    match (left, right) {
        (Some(Value::Number(left)), Value::Number(right)) => {
            matches!((input_decimal(&left.to_string()), input_decimal(&right.to_string())), (Ok(left), Ok(right)) if left == right)
        }
        _ => left == Some(right),
    }
}

pub fn parse_option_input(name: &str, text: &str) -> Result<Value, String> {
    if !ALLOWED.contains(&name) {
        return Err(format!("Unsupported option '{name}'."));
    }
    if name == "crit" || name == "stop" {
        return Ok(Value::String(text.into()));
    }
    if (name == "start_fons" || name == "duration") && text.trim().is_empty() {
        return Ok(Value::Null);
    }
    let number = input_decimal(text)?;
    if name == "duration" {
        Ok(json!(number.to_microseconds()? as f64 / 1e6))
    } else {
        Ok(json!(number.to_safe_integer()?))
    }
}

fn byte_offset(source: &str, offset: usize) -> Result<usize, String> {
    let mut utf16 = 0;
    for (byte, character) in source.char_indices() {
        if utf16 == offset {
            return Ok(byte);
        }
        utf16 += character.len_utf16();
    }
    if utf16 == offset {
        Ok(source.len())
    } else {
        Err("Invalid UTF-16 source range.".into())
    }
}

pub fn edit_options(
    source: &str,
    changes: &BTreeMap<String, Value>,
) -> Result<String, Vec<Diagnostic>> {
    // Editing configuration must not execute/unroll the live rotation, or depend
    // on its action support. Syntax and configuration still validate atomically.
    let (script, before) = super::inspection::prepare(source)?;
    let before_values = editable_values(&before.options);
    let fallback = SourceLocation {
        line: 1,
        column: 1,
        offset: 0,
        length: 0,
    };
    let fail = |message| vec![failure(message, &fallback)];
    let mut replacements = Vec::new();
    let mut additions = Vec::new();
    for (name, value) in changes {
        if !ALLOWED.contains(&name.as_str()) {
            return Err(fail(format!("Unsupported option '{name}'.")));
        }
        if same_value(before_values.get(name), value) {
            continue;
        }
        let replacement = if value.is_null() && (name == "start_fons" || name == "duration") {
            None
        } else if let Some(number) = value.as_number() {
            Some(canonical_decimal(&number.to_string()).map_err(fail)?)
        } else if let Some(text) = value.as_str() {
            if !text.bytes().all(|c| c.is_ascii_lowercase()) || text.is_empty() {
                return Err(fail(format!("Invalid option value for {name}.")));
            }
            Some(text.into())
        } else {
            return Err(fail(format!("Invalid option value for {name}.")));
        };
        if let Some(option) = script.options.iter().find(|option| &option.key == name) {
            let start = if replacement.is_some() {
                option.value_range.offset
            } else {
                option.loc.offset
            };
            replacements.push((
                byte_offset(source, start).map_err(fail)?,
                byte_offset(
                    source,
                    option.value_range.offset + option.value_range.length,
                )
                .map_err(fail)?,
                replacement.unwrap_or_default(),
            ));
        } else if let Some(replacement) = replacement {
            additions.push(format!("{name}={replacement}"));
        }
    }
    if !additions.is_empty() {
        let end = script
            .options
            .iter()
            .map(|option| option.value_range.offset + option.value_range.length)
            .max()
            .ok_or_else(|| fail("Missing options declaration.".into()))?;
        let byte = byte_offset(source, end).map_err(fail)?;
        replacements.push((byte, byte, format!(" {}", additions.join(" "))));
    }
    // Insertions at a value's old endpoint happen first, so later replacement of
    // that value cannot consume newly added keys.
    replacements.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    let mut candidate = source.to_owned();
    for (start, end, text) in replacements {
        candidate.replace_range(start..end, &text);
    }
    let (_, after) = super::inspection::prepare(&candidate)?;
    let after_values = editable_values(&after.options);
    for (name, value) in changes {
        if !same_value(after_values.get(name), value) {
            return Err(fail(format!(
                "The edited script does not settle option '{name}' to its requested value."
            )));
        }
    }
    Ok(candidate)
}

fn editable_values(options: &super::options::CompiledOptions) -> Value {
    json!({
        "iteration": options.trials,
        "duration": options.duration_specified.then_some(options.relative_duration_us as f64 / 1e6),
        "seed": options.seed,
        "crit": options.crit,
        "start_fons": options.start_fons,
        "stop": options.stop,
    })
}
