//! Unlocked passives read actual field state and real shared Charge callbacks.
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    ExactFraction, GameActorConfig, GameScenarioRequest, state::GameSimulatorState,
};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizChargePassivesV1;

pub(crate) fn energy(
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    (ability == "GA_Chiichan073_Skill"
        && matches!(
            packet,
            "GE_Player_Chiichan073_Skill1_1Damage"
                | "GE_Player_Chiichan073_Skill1_Damage"
                | "GE_Player_Chiichan073_Skill2_2Damage"
                | "GE_Player_Chiichan073_Skill3_Damage"
                | "GE_Player_Chiichan073_Skill4_Damage"
        ))
    .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
        trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        hypothesis_id: if packet == "GE_Player_Chiichan073_Skill1_1Damage" {
            MODEL
        } else {
            super::pressed::MODEL
        },
        event_kind: if packet == "GE_Player_Chiichan073_Skill1_1Damage" {
            "chiz_no_grain_energy_grant"
        } else {
            "chiz_skill_energy_grant"
        },
    })
}

fn unlocked(config: &GameActorConfig, breakthrough: u64) -> Result<bool, String> {
    Ok(ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?["breakthrough"]
        .as_u64()
        .unwrap_or(0)
        >= breakthrough)
}

pub(crate) fn field(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    active: bool,
) -> Result<(), String> {
    let config = &ctx.request.actors[owner];
    if !unlocked(config, 4)? {
        return Ok(());
    }
    let actor = &mut ctx.state.actors[owner];
    if actor.character.chiz_mut()?.hoarding_active == active {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let current = ExactFraction::from_ratio(actor.charge_multiplier)?;
    let bonus = super::parameters::PARAMETERS.hoarding_efficiency_add;
    let updated = if active {
        current.add(bonus)?
    } else {
        current.sub(bonus)?
    };
    actor.charge_multiplier = crate::formula::Ratio {
        numerator: updated
            .numerator
            .try_into()
            .map_err(|_| "Chiz Hoarding numerator overflow")?,
        denominator: updated
            .denominator
            .try_into()
            .map_err(|_| "Chiz Hoarding denominator overflow")?,
    };
    actor.character.chiz_mut()?.hoarding_active = active;
    Ok(())
}

pub(crate) fn charge_bonus(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<u64, String> {
    for (owner, config) in request.actors.iter().enumerate() {
        if config.character_id == "character:1073"
            && state.actors[owner].hp > 0
            && unlocked(config, 2)?
        {
            crate::runtime_api::record_hypothesis_id(request, assumptions, MODEL)?;
            return Ok(super::parameters::PARAMETERS.hanahaki_charge_add);
        }
    }
    Ok(0)
}
