//! Shared scalar authoring rules; combat eligibility remains with content.
use super::evaluate::Value;

pub(crate) fn window_name(name: &str) -> Result<crate::simulation::SourceActionWindow, String> {
    use crate::simulation::SourceActionWindow;
    match name {
        "attack" => Ok(SourceActionWindow::Attack),
        "hold" => Ok(SourceActionWindow::Hold),
        "jump" => Ok(SourceActionWindow::Jump),
        "dash" => Ok(SourceActionWindow::Dash),
        "skill" => Ok(SourceActionWindow::Skill),
        "swap" => Ok(SourceActionWindow::Swap),
        _ => Err("window requires attack, hold, jump, dash, skill or swap.".into()),
    }
}

pub(crate) fn window(
    expression: &super::Expression,
) -> Result<crate::simulation::SourceActionWindow, String> {
    let super::ExpressionKind::Identifier(name) = &expression.kind else {
        return Err("window requires a static input-kind identifier.".into());
    };
    window_name(name)
}

pub(crate) fn target_name(name: &str, target_count: usize) -> Result<usize, String> {
    let index = name
        .strip_prefix('t')
        .and_then(|digits| digits.parse::<usize>().ok())
        .filter(|index| name == format!("t{index}"))
        .ok_or("target requires a dummy identifier tN.")?;
    if index >= target_count {
        return Err(format!("Unknown target '{name}'."));
    }
    Ok(index)
}

pub(crate) fn target(expression: &super::Expression, count: usize) -> Result<usize, String> {
    let super::ExpressionKind::Identifier(name) = &expression.kind else {
        return Err("target requires a dummy identifier tN.".into());
    };
    target_name(name, count)
}

pub(crate) fn boolean(value: &Value, label: &str) -> Result<bool, String> {
    match value {
        Value::Boolean(flag) => return Ok(*flag),
        Value::Number(number) => match number.to_safe_integer() {
            Ok(0) => return Ok(false),
            Ok(1) => return Ok(true),
            _ => {}
        },
        _ => {}
    }
    Err(format!("{label} must be true or false (0 or 1)."))
}

pub(crate) fn stocks(value: &Value) -> Result<u8, String> {
    let count = value
        .number()
        .and_then(|number| number.to_safe_integer())
        .map_err(|message| format!("stocks: {message}"))?;
    u8::try_from(count).map_err(|_| "stocks exceed the supported integer count range.".into())
}

pub(crate) fn branch_landing(value: &Value) -> Result<u64, String> {
    let duration = value.number()?.to_microseconds()?;
    if duration == 0 {
        return Err("Branch landing must be positive.".into());
    }
    Ok(duration)
}

pub(crate) fn branch_height(value: &Value) -> Result<crate::formula::Ratio, String> {
    let (numerator, denominator) = value.number()?.to_safe_ratio()?;
    if numerator > denominator {
        return Err("Branch height_bonus must be within0..1.".into());
    }
    Ok(crate::formula::Ratio {
        numerator,
        denominator,
    })
}

/// Source shape and units only; the owner validates Branch eligibility and gates.
pub(crate) fn branch(
    params: &[(&str, Value)],
    has_hold: bool,
) -> Result<Option<crate::simulation::GameBranchInput>, String> {
    let mut launchable = None;
    let mut landing = None;
    let mut height = None;
    for (key, value) in params {
        match *key {
            "launchable" => launchable = Some(boolean(value, "launchable")?),
            "landing" => landing = Some(branch_landing(value)?),
            "height_bonus" => height = Some(branch_height(value)?),
            _ => {}
        }
    }
    if launchable.is_none() && landing.is_none() && height.is_none() {
        return Ok(None);
    }
    if !has_hold {
        return Err("Branch choices require an explicit hold duration.".into());
    }
    Ok(Some(crate::simulation::GameBranchInput {
        can_be_launched: launchable.ok_or("Branch requires launchable.")?,
        landing_delay_us: landing.ok_or("Branch requires landing.")?,
        height_bonus: height.unwrap_or_else(crate::formula::Ratio::zero),
    }))
}
