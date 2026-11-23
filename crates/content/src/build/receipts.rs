use super::*;

pub(crate) fn receipt(value: &Value) -> Result<ExactFraction, String> {
    let number = |key| {
        value[key]
            .as_str()
            .ok_or_else(|| format!("missing exact build receipt {key}"))?
            .parse::<u128>()
            .map_err(|_| format!("invalid exact build receipt {key}"))
    };
    ExactFraction::new(number("numerator")?, number("denominator")?)
}

pub(crate) fn ratio(value: ExactFraction) -> Result<Ratio, String> {
    Ok(Ratio {
        numerator: value
            .numerator
            .try_into()
            .map_err(|_| "build numerator exceeds runner u64")?,
        denominator: value
            .denominator
            .try_into()
            .map_err(|_| "build denominator exceeds runner u64")?,
    })
}

pub(crate) fn integer(value: &Value, name: &str) -> Result<u64, String> {
    let value = receipt(value)?;
    if value.denominator != 1 {
        return Err(format!(
            "fractional build {name} requires an evidenced quantization policy"
        ));
    }
    value
        .numerator
        .try_into()
        .map_err(|_| format!("build {name} exceeds runner u64"))
}

pub(crate) fn nearest_integer(value: &Value, name: &str) -> Result<u64, String> {
    receipt(value)?
        .round_u64()
        .map_err(|error| format!("build {name} nearest-integer settlement failed: {error}"))
}
