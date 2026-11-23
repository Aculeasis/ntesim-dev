//! Delay Warning captures one enhanced shared Remora generation.
use crate::runtime_api::RemoraEnhancement;
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorB2RemoraV1;

pub(crate) fn policy(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<Option<RemoraEnhancement>, String> {
    for (owner, config) in request.actors.iter().enumerate() {
        if config.character_id != "character:1025" || state.actors[owner].hp == 0 {
            continue;
        }
        let progression = ntesim_data::skill_levels::progression(
            &config.character_id,
            config.character_level,
            config.character_max_level,
        )?;
        if progression["breakthrough"].as_u64().unwrap_or(0) < 2 {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(request, assumptions, MODEL)?;
        // Delay Warning's selected description:12s Remora and target CR+.10.
        return Ok(Some(RemoraEnhancement {
            duration_us: 12_000_000,
            crit_rate_bonus: crate::formula::Ratio {
                numerator: 1,
                denominator: 10,
            },
        }));
    }
    Ok(None)
}
