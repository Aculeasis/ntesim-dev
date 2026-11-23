use crate::simulation::GameScenarioRequest;
use std::collections::BTreeMap;

pub(crate) fn record_hypothesis(
    request: &GameScenarioRequest,
    execution_assumptions: &mut BTreeMap<String, String>,
    id: impl AsRef<str>,
    description: impl AsRef<str>,
) -> Result<(), String> {
    let id = id.as_ref();
    let description = description.as_ref();
    if request
        .model_assumptions
        .get(id)
        .is_some_and(|value| value != description)
    {
        return Err(format!("conflicting reached model hypothesis {id}"));
    }
    if execution_assumptions
        .get(id)
        .is_some_and(|value| value != description)
    {
        return Err(format!("conflicting reached model hypothesis {id}"));
    }
    execution_assumptions.insert(id.into(), description.into());
    Ok(())
}

pub(crate) fn record_hypothesis_id(
    request: &GameScenarioRequest,
    execution_assumptions: &mut BTreeMap<String, String>,
    id: impl AsRef<str>,
) -> Result<(), String> {
    let id_str = id.as_ref();
    let desc = ntesim_contracts::get_hypothesis_description(id_str)
        .ok_or_else(|| format!("unknown hypothesis id: {id_str}"))?;
    record_hypothesis(request, execution_assumptions, id_str, desc)
}
