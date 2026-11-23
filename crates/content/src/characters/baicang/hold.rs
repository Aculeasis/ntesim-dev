//! Release-triggered held inputs; owner state, never caller packet IDs, selects the branch.
use crate::runtime_api::NormalActionOverride;
use crate::simulation::{
    GameActionCommand, GameHoldInput, GameScenarioRequest, state::ActorRuntime,
};

pub(crate) const THRESHOLD_US: u64 = 200_000;
pub(crate) const MAX_RECOVERY_US: u64 = 1_481_224;

pub(crate) fn validate_inputs(
    request: &GameScenarioRequest,
    packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
) -> Result<(), String> {
    for (&index, input) in &request.hold_inputs {
        if crate::characters::hathor::branch::entry_hold(request, index) {
            continue;
        }
        let Some(GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            ..
        }) = request.actions.get(index)
        else {
            return Err(format!(
                "hold_inputs index {index} must identify an InvokeAbility"
            ));
        };
        if matches!(
            request.actors[*actor_index].character_id.as_str(),
            "character:1042"
                | "character:1010"
                | "character:1071"
                | "character:1073"
                | "character:1054"
                | "character:1036"
                | "character:1021"
                | "character:1020"
                | "character:1025"
                | "character:1019"
                | "character:1075"
        ) {
            continue;
        }
        let metadata = packets[index]
            .as_ref()
            .ok_or("hold packet metadata missing")?;
        if request.actors[*actor_index].character_id != "character:1023"
            || metadata.action.ability_id != "GA_Cang_Melee"
            || metadata.packet.damage_effect_id != "GE_Player_Cang_Melee1_Damage"
            || request.plunge_inputs.contains_key(&index)
        {
            return Err(format!(
                "hold_inputs index {index} has no supported owner Normal binding"
            ));
        }
        if input.moving && input.duration_us < THRESHOLD_US {
            return Err("movement intent requires a Baicang heavy hold".into());
        }
        timestamp_us
            .checked_add(input.duration_us)
            .and_then(|time| time.checked_add(MAX_RECOVERY_US))
            .ok_or("Baicang hold deadline overflow")?;
    }
    Ok(())
}

pub(crate) fn replacement(
    actor: &ActorRuntime,
    input: GameHoldInput,
    now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    if input.duration_us < THRESHOLD_US {
        return Ok(None);
    }
    let crate::characters::RuntimeState::Baicang(state) = &actor.character else {
        return Err("unsupported held-input owner".into());
    };
    let after_dodge = state.heavy_dodge_until_us > now;
    let stage = if state
        .normal_started_us
        .is_some_and(|previous| now.saturating_sub(previous) <= 2_000_000)
    {
        state.normal_stage
    } else {
        0
    };
    let vertical = if after_dodge {
        input.moving
    } else if input.moving {
        return Err("Baicang moving heavy requires a preceding admitted dash".into());
    } else {
        match stage {
            1 | 2 => false,
            3 => true,
            _ => {
                return Err(
                    "Baicang heavy requires Normal1/2/3 or a preceding admitted dash".into(),
                );
            }
        }
    };
    Ok(Some(NormalActionOverride {
        packet: if vertical {
            super::branches::BRANCH2_PACKET
        } else {
            super::branches::BRANCH1_PACKET
        },
        role: None,
        model: super::branches::HYPOTHESIS_ID,
        timing: None,
    }))
}
