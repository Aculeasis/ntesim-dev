//! Paid evade, Critical Riposte (ComboSectionMap-preserving), Stage Dive plunge, and QTE.
use crate::runtime_api::{
    CastContext, DirectPacketEnergyPolicy, NormalActionOverride, PlungeInputContext,
};
use crate::simulation::{ExactFraction, GameScenarioRequest, PlayerInput, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::AkaneSecondaryActionsV1;
pub(crate) const PLUNGE: &str = "GE_Player_Akane_Airattack_Damage";
pub(crate) const RIPOSTE: &str = "GE_Player_Akane_PerfectEvadeAttack_Damage";

pub(crate) fn public_plunge() -> crate::build::PublicAction {
    crate::build::PublicAction {
        command: crate::simulation::GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Akane_Melee".into(),
            damage_effect_id: PLUNGE.into(),
            array_index: 0,
            atk_rate: crate::formula::Ratio::one(),
            scaling_stat: Default::default(),
            energy_cost: 0,
            recovery_us: super::clock::profile::PLUNGE_RECOVERY,
            cooldown_us: 0,
            hit_delay_us: super::clock::profile::PLUNGE_HIT,
        },
        advance_us: super::clock::profile::PLUNGE_RECOVERY,
        model_assumptions: [
            MODEL,
            super::clock::MODEL,
            super::skill::STRING_ENERGY_HYPOTHESIS_ID,
        ]
        .into_iter()
        .map(|id| (id.into(), id.description().into()))
        .collect(),
    }
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

fn require(request: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(request) || !super::clock::accepted(request) {
        return Err(
            "Akane secondary actions require akane.secondary_actions_v1 and akane.ordinary_clock_v1"
                .into(),
        );
    }
    Ok(())
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request) {
        require(request)?;
    }
    for (index, command) in request.actions.iter().enumerate() {
        let crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            damage_effect_id,
            ..
        } = command
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1057" {
            continue;
        }
        if damage_effect_id == RIPOSTE {
            return Err(
                "Akane Critical Riposte is owner-selected: use canonical Normal after Critical Dodge"
                    .into(),
            );
        }
        if damage_effect_id == PLUNGE && !request.plunge_inputs.contains_key(&index) {
            return Err(
                "Akane Stage Dive plunge requires an explicit positive airborne landing/height profile"
                    .into(),
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_input(
    request: &GameScenarioRequest,
    input: PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    if input != PlayerInput::Dash || !accepted(request) {
        return Ok(None);
    }
    require(request)?;
    if request.combat_stamina.is_none() {
        return Err("Akane Evade requires an explicit combat stamina profile".into());
    }
    Ok(Some(MODEL))
}

pub(crate) fn validate_plunge(ctx: &PlungeInputContext<'_>) -> Result<(), String> {
    require(ctx.request)?;
    if ctx.metadata.action.ability_id != "GA_Akane_Melee"
        || ctx.metadata.packet.damage_effect_id != PLUNGE
    {
        return Err("Akane Stage Dive requires Normal-family plunge row".into());
    }
    ctx.timestamp_us
        .checked_add(ctx.input.landing_delay_us)
        .and_then(|time| time.checked_add(super::clock::profile::PLUNGE_RECOVERY))
        .ok_or("Akane Stage Dive deadline overflow")?;
    Ok(())
}

pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    matches!(&actor.character, crate::characters::RuntimeState::Akane(state) if state.riposte_ready)
        .then_some(NormalActionOverride {
            packet: RIPOSTE,
            role: Some(crate::simulation::metadata::InputActionRole::CriticalRiposte),
            model: MODEL,
            timing: Some((
                super::clock::profile::RIPOSTE_RECOVERY,
                super::clock::profile::RIPOSTE_HIT,
            )),
        })
}

pub(crate) fn validate_packet(
    actor: &ActorRuntime,
    ability: &str,
    packet: &str,
) -> Result<(), String> {
    if packet == RIPOSTE && (ability != "GA_Akane_Melee" || replacement(actor).is_none()) {
        return Err("Akane Critical Riposte requires one paid successful Critical Dodge".into());
    }
    Ok(())
}

pub(crate) fn critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    require(ctx.request)?;
    ctx.record_hypothesis(MODEL)?;
    let state = ctx.actor.character.akane_mut()?;
    state.riposte_ready = true;
    state.melee_pre_ready = false;
    Ok(())
}

pub(crate) fn on_input(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    input: PlayerInput,
) -> Result<(), String> {
    let state = actor.character.akane_mut()?;
    state.riposte_ready = false;
    if input == PlayerInput::Dash && accepted(request) {
        // Dodge resets entry state and stages MeleePre (-> Melee2) while keeping chain options clean.
        state.entry_normal_ready = false;
        state.melee_pre_ready = true;
        actor.locked_until_us = actor
            .last_action_start_us
            .checked_add(super::clock::profile::EVADE_FORWARD_RECOVERY)
            .ok_or("Akane Evade recovery overflow")?;
    } else if input == PlayerInput::Jump {
        state.entry_normal_ready = false;
        state.melee_pre_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    Ok(())
}

pub(crate) fn committed_input(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if input == PlayerInput::Dash && accepted(ctx.request) {
        super::clock::secondary::dodge(ctx, owner, command)
    } else if input == PlayerInput::Jump {
        super::clock::cancel::on_jump(ctx, owner)
    } else {
        Ok(())
    }
}

pub(super) fn on_cast(ctx: &mut CastContext<'_>) -> Result<Option<super::clock::Kind>, String> {
    let packet = ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        .unwrap_or_default();
    let kind = match packet {
        PLUNGE => super::clock::Kind::Plunge,
        RIPOSTE => super::clock::Kind::Riposte,
        _ => return Ok(None),
    };
    require(ctx.request)?;
    if ctx.ability_id != "GA_Akane_Melee" {
        return Err("Akane secondary requires GA_Akane_Melee initiator".into());
    }
    ctx.record_hypothesis(MODEL)?;
    if ctx.targets.len() > 1 {
        ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
    }
    if matches!(kind, super::clock::Kind::Plunge) {
        let input = ctx
            .request
            .plunge_inputs
            .get(&ctx.command_index)
            .ok_or("Akane Stage Dive profile missing")?;
        ctx.scale_primary_coefficient(
            PLUNGE,
            ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(input.height_bonus)?)?,
        )?;
        ctx.scope_primary_aoe_group(PLUNGE)?;
        let state = ctx.actor.character.akane_mut()?;
        state.riposte_ready = false;
        state.entry_normal_ready = false;
        state.melee_pre_ready = false;
        state.normal_stage = 0;
        state.normal_started_us = None;
    } else {
        // Featherblade Reverb preserves current normal stage (ComboSectionMap: 1->2->3->4->1, Pre/Appear->2).
        ctx.scope_primary_aoe_group(RIPOSTE)?;
        let state = ctx.actor.character.akane_mut()?;
        state.riposte_ready = false;
        state.entry_normal_ready = false;
        state.melee_pre_ready = false;
        state.normal_started_us = Some(ctx.time_us);
    }
    Ok(Some(kind))
}

pub(crate) fn energy(ability: &str, packet: &str) -> Option<DirectPacketEnergyPolicy> {
    match (ability, packet) {
        ("GA_Akane_PerfectEvade", RIPOSTE)
        | ("GA_Akane_Melee", RIPOSTE | PLUNGE)
        | ("GA_Akane_QTE", "GE_Player_Akane_QTE_Damage") => Some(DirectPacketEnergyPolicy {
            hypothesis_id: crate::HypothesisId::AkaneOrdinaryHitEnergyV1,
            event_kind: "akane_direct_packet_energy_grant",
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
        }),
        _ => None,
    }
}
