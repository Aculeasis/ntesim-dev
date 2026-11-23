//! Native secondary admissions; input timing remains on the owner clock.
use super::{Kind, profile::Hit};
use crate::runtime_api::{CastContext, PlungeInputContext, SupportEntry};
use crate::simulation::{
    ExactFraction, GameActionCommand, GameScenarioRequest, PlayerInput, state::GameSimulatorState,
};
pub(crate) const SUPPORT_PACKET: &str = "GE_Player_Mint_QTE1_Damage";
const SUPPORT: SupportEntry = SupportEntry {
    ability_id: "GA_Mint019_QTE",
    damage_effect_id: SUPPORT_PACKET,
    hit_delay_us: 739424,
    recovery_us: 1366703,
    dispatch_model: super::MODEL,
    packet_model: Some(super::MODEL),
    extra_damage_effects: &[],
};
const SUPPORT_HITS: &[Hit] = &[(SUPPORT_PACKET, 739424)];
const RIPOSTE_HITS: &[Hit] = &[(super::super::riposte::PACKET, 227216)];
const PLUNGE_HITS: &[Hit] = &[(super::super::plunge::PACKET, 64514)];
pub(super) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    for &command in &req.approach_inputs {
        if !super::accepted(req)
            || !matches!(req.actions.get(command),Some(GameActionCommand::InvokeAbility { actor_index, ability_id, damage_effect_id, .. }) if req.actors.get(*actor_index).is_some_and(|a|a.character_id == "character:1019") && ability_id == "GA_Mint019_Melee" && damage_effect_id == "GE_Player_Mint_Melee1_Damage_New")
            || req.plunge_inputs.contains_key(&command)
        {
            return Err("approach_inputs requires a canonical Mint Normal without plunge under ordinary_clock_v1".into());
        }
    }
    if !super::accepted(req) {
        return Ok(());
    }
    for action in &req.actions {
        if let GameActionCommand::PlayerInput {
            actor_index, input, ..
        } = action
            && req.actors[*actor_index].character_id == "character:1019"
        {
            validate_input(req, *input)?;
        }
    }
    Ok(())
}
pub(crate) fn admission(
    req: &GameScenarioRequest,
    command: usize,
    actor: &crate::simulation::state::ActorRuntime,
    ability: &str,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if !super::accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Mint(state) = &actor.character else {
        return None;
    };
    let plunge = ability == "GA_Mint019_Melee" && req.plunge_inputs.contains_key(&command);
    let message = if state.airborne && !plunge {
        "Mint airborne state requires a declared plunge and landing before another grounded input"
    } else if !state.airborne && plunge {
        "Mint plunge requires a new admitted Jump"
    } else {
        return None;
    };
    Some(crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.into(),
        },
        message: message.into(),
    })
}
pub(super) const ENTRY_HITS: &[Hit] = &[
    ("GE_Player_Mint_Melee1_Damage_New", 199890),
    ("GE_Player_Mint_Melee2_Damage_New", 350109),
];
pub(super) const APPROACH_HITS: &[Hit] = &[
    ("GE_Player_Mint_Melee1_Damage_New", 205847),
    ("GE_Player_Mint_Melee2_Damage_New", 360205),
];
pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    if !super::accepted(ctx.request)
        || ctx.metadata.action.ability_id != "GA_Mint019_Melee"
        || ctx.metadata.packet.damage_effect_id != super::super::plunge::PACKET
    {
        return Err("Mint plunge requires ordinary_clock_v1 and its native landing packet".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|t| t.checked_add(865790))
        .ok_or("Mint plunge deadline overflow")?;
    Ok(())
}
pub(crate) fn validate_input(
    req: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if !super::accepted(req) {
        return Ok(None);
    }
    if !matches!(input, PlayerInput::Dash | PlayerInput::Jump) {
        return Err("Mint native controls support Dodge and Jump".into());
    }
    if input == PlayerInput::Dash && req.combat_stamina.is_none() {
        return Err("Mint Dodge requires explicit combat stamina".into());
    }
    Ok(Some(super::MODEL))
}
pub(crate) fn select_support(
    req: &GameScenarioRequest,
    state: &GameSimulatorState,
    new: usize,
    qualified_support: bool,
) -> Result<Option<&'static SupportEntry>, String> {
    if !super::accepted(req)
        || state.actors[new].locked_until_us > state.current_time_us
        || !qualified_support
    {
        return Ok(None);
    }
    Ok(Some(&SUPPORT))
}
pub(super) fn selection(
    ctx: &mut CastContext<'_>,
) -> Result<Option<(Kind, &'static [Hit], u64)>, String> {
    let effect = ctx
        .packet_metadata
        .map(|p| p.damage_effect_id.as_str())
        .unwrap_or_default();
    let result = match effect {
        super::super::plunge::PACKET => {
            let input = *ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Mint plunge landing profile missing")?;
            if !ctx.actor.character.mint_mut()?.airborne {
                return Err(
                    "Mint plunge requires a new admitted Jump and subsequent landing".into(),
                );
            }
            ctx.record_hypothesis(super::super::plunge::HYPOTHESIS_ID)?;
            ctx.scale_primary_coefficient(
                effect,
                ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(input.height_bonus)?)?,
            )?;
            Some((Kind::Plunge, PLUNGE_HITS, input.landing_delay_us))
        }
        super::super::riposte::PACKET => {
            if super::super::riposte::replacement(ctx.actor).is_none() {
                return Err("Mint Riposte requires a paid successful Critical Dodge".into());
            }
            ctx.record_hypothesis(super::super::riposte::HYPOTHESIS_ID)?;
            Some((Kind::Riposte, RIPOSTE_HITS, 0))
        }
        SUPPORT_PACKET => {
            if !matches!(ctx.request.actions.get(ctx.command_index),Some(GameActionCommand::SwitchActor{target_actor_index,..}) if *target_actor_index==ctx.actor_index)
            {
                return Err("Mint Support requires an engine-qualified switch".into());
            }
            Some((Kind::Support, SUPPORT_HITS, 0))
        }
        _ => None,
    };
    if result.is_some() {
        ctx.actor.character.mint_mut()?.riposte_ready = false;
    }
    Ok(result)
}
pub(super) fn gate(kind: Kind, input: &str) -> u64 {
    let recovery = super::profile::recovery(kind);
    match (kind, input) {
        (Kind::Normal(stage), "jump") => [333348, 248494, 516732, 166470, 668227][stage],
        (Kind::Normal(stage), "dash") => [883359, 783545, 1285228, 798570, 1616400][stage],
        (Kind::Skill, "jump") => 684683,
        (Kind::Skill, "dash") => 1202017,
        (Kind::Burst, "switch_actor") => 3792167,
        (Kind::Entry, "jump") => 416638,
        (Kind::Entry, "dash") => 883359,
        (Kind::Approach, "jump") => 450835,
        (Kind::Approach, "dash") => 916835,
        (Kind::Plunge, "jump") => 216850,
        (Kind::Plunge, "dash") => 865790,
        (Kind::Riposte, "jump") | (Kind::End, "jump") | (Kind::Hold, "jump") => 0,
        (Kind::Support, "switch_actor") => 1333238,
        (Kind::Hold, "dash") => 6049112,
        (Kind::Hold, _) => 1818746,
        (Kind::Dodge(false), "jump") => 82544,
        (Kind::Dodge(false), "GA_Mint019_Skill") => 340205,
        (Kind::Dodge(true), "dash") => 797188,
        (Kind::Dodge(true), "GA_Mint019_Skill") => 448540,
        _ => recovery,
    }
}
