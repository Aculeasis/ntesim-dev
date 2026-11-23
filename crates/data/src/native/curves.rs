use super::{Scalar, Value};
use serde_json::json;

fn coordinate(value: &Value) -> Result<u32, String> {
    let receipt = Scalar::parse(value)?.receipt();
    if receipt["denominator"] != "1" {
        return Err("native curve requires integral serialized key coordinates".into());
    }
    receipt["numerator"]
        .as_str()
        .ok_or("curve coordinate missing")?
        .parse()
        .map_err(|_| "native curve coordinate exceeds u32".into())
}

pub fn curve_selection_receipt(curve: &Value, time: u32) -> Result<Value, String> {
    let keys = curve["Keys"]
        .as_array()
        .ok_or("native curve keys missing")?;
    if keys.is_empty() {
        return Err("native curve requires serialized keys".into());
    }
    let mut coordinates = Vec::new();
    for key in keys {
        let key_time = coordinate(&key["Time"])?;
        if coordinates
            .last()
            .is_some_and(|previous| *previous >= key_time)
        {
            return Err("native curve coordinates must be strictly increasing".into());
        }
        Scalar::parse(&key["Value"])?;
        coordinates.push(key_time);
    }
    let (index, selection) = if let Some(index) = coordinates.iter().position(|t| *t == time) {
        (index, "exact_serialized_key")
    } else {
        if curve["InterpMode"] != "ERichCurveInterpMode::RCIM_Constant" {
            return Err("non-key coordinate requires supported native Constant mode".into());
        }
        if time < coordinates[0] || time > *coordinates.last().unwrap() {
            return Err("native curve extrapolation is unsupported".into());
        }
        (
            coordinates
                .iter()
                .rposition(|t| *t < time)
                .ok_or("curve left key missing")?,
            "serialized_constant_left_key",
        )
    };
    Ok(
        json!({"coordinate":time,"selection":selection,"selected_key_index":index,
        "selected_key":keys[index],"value":Scalar::parse(&keys[index]["Value"])?.receipt()}),
    )
}

pub fn constant_curve_receipt(curve: &Value, time: u32) -> Result<Value, String> {
    if curve["InterpMode"] != "ERichCurveInterpMode::RCIM_Constant" {
        return Err("native constant curve requires Constant interpolation".into());
    }
    curve_selection_receipt(curve, time)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_new_curve_has_exact_selection_and_checked_left_intervals() {
        let curve = json!({"InterpMode":"ERichCurveInterpMode::RCIM_Constant",
            "Keys":[{"Time":1.0,"Value":0.125},{"Time":4,"Value":1.75}]});
        assert_eq!(
            constant_curve_receipt(&curve, 3).unwrap()["value"],
            json!({"numerator":"1","denominator":"8"})
        );
        assert_eq!(
            constant_curve_receipt(&curve, 4).unwrap()["value"],
            json!({"numerator":"7","denominator":"4"})
        );
        assert!(constant_curve_receipt(&curve, 0).is_err());
        assert!(constant_curve_receipt(&curve, 5).is_err());
    }
    #[test]
    fn malformed_unused_keys_and_near_coordinates_are_rejected() {
        for keys in [
            json!([]),
            json!([{"Time":1,"Value":1},{"Time":1,"Value":2}]),
            json!([{"Time":1,"Value":1},{"Time":4,"Value":-1}]),
            json!([{"Time":20.0000001,"Value":1}]),
        ] {
            assert!(curve_selection_receipt(&json!({"Keys":keys}), 20).is_err());
        }
        let linear = json!({"InterpMode":"ERichCurveInterpMode::RCIM_Linear",
            "Keys":[{"Time":1,"Value":1},{"Time":3,"Value":3}]});
        assert!(curve_selection_receipt(&linear, 1).is_ok());
        assert!(curve_selection_receipt(&linear, 2).is_err());
        assert!(constant_curve_receipt(&linear, 1).is_err());
    }
}
