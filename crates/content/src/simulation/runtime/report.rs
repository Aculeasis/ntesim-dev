use crate::simulation::GameScenarioReport;
use serde::Serialize;
use serde_json::{Value, value::RawValue};
use std::collections::BTreeMap;

/// Source admission binds native owner inputs and exact projected builds. This
/// assesses the selected, disclosed model, never evidence-bound strict readiness.
pub(super) fn source_quality(
    request: &crate::simulation::GameScenarioRequest,
    resolved: &BTreeMap<String, String>,
) -> ntesim_contracts::Quality {
    use ntesim_contracts::{HypothesisId, Quality};
    if request.action_admission_order
        != crate::simulation::GameActionAdmissionOrder::AfterSettledEvents
    {
        return Quality::Incomplete;
    }
    let known = |id: &str, description: &str| {
        HypothesisId::parse(id).is_some_and(|model| model.description() == description)
            || (id, description) == crate::build::studio_projection::STATIC_ADDITIVE_MODEL
    };
    if !request.action_metadata.is_empty()
        || request
            .model_assumptions
            .iter()
            .chain(resolved)
            .any(|(id, description)| !known(id, description))
    {
        return Quality::UnboundedHypothesis;
    }
    Quality::BoundedHypothesis
}
pub(crate) fn merge_model_assumptions(
    resolved: &mut BTreeMap<String, String>,
    assumptions: impl IntoIterator<Item = (String, String)>,
) -> Result<(), String> {
    for (id, description) in assumptions {
        if resolved
            .insert(id.clone(), description.clone())
            .is_some_and(|previous| previous != description)
        {
            return Err(format!(
                "resolved model assumption has conflicting descriptions: {id}"
            ));
        }
    }
    Ok(())
}

pub struct BorrowedGameScenarioReport<'a> {
    pub(super) report: GameScenarioReport,
    pub(super) talents: &'a [Value],
    pub(super) serialized_talents: &'a RawValue,
}

impl BorrowedGameScenarioReport<'_> {
    pub fn into_owned(mut self) -> GameScenarioReport {
        self.report.talent_projection_receipts = self.talents.to_vec();
        self.report
    }
}

impl Serialize for BorrowedGameScenarioReport<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct FullReport<'a> {
            #[serde(flatten)]
            report: &'a GameScenarioReport,
            #[serde(skip_serializing_if = "Option::is_none")]
            talent_projection_receipts: Option<&'a RawValue>,
        }
        FullReport {
            report: &self.report,
            talent_projection_receipts: (!self.talents.is_empty())
                .then_some(self.serialized_talents),
        }
        .serialize(serializer)
    }
}
