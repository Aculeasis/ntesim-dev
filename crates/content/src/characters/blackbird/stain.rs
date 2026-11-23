//! B2 source values and the selected generation-captured Stain policy.
use crate::HypothesisId;
use crate::runtime_api::StainEnhancement;
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdB2StainV1;

pub(crate) fn policy(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<Option<StainEnhancement>, String> {
    for (index, actor) in request.actors.iter().enumerate() {
        if actor.character_id != "character:1042" || state.actors[index].hp == 0 {
            continue;
        }
        let progression = ntesim_data::skill_levels::progression(
            &actor.character_id,
            actor.character_level,
            actor.character_max_level,
        )?;
        if progression["breakthrough"].as_u64().unwrap_or(0) < 2 {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
        // DT_ReactionExtendDataTable1042 DamageUpRatio=.10; native cue's
        // eight-second time change extends the shared12-second generation once.
        return Ok(Some(StainEnhancement {
            duration_us: 20_000_000,
            base_bonus: crate::formula::Ratio {
                numerator: 1,
                denominator: 10,
            },
        }));
    }
    Ok(None)
}
