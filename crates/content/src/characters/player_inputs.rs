use crate::HypothesisId;
use crate::simulation::{PlayerInput, state::ActorRuntime};

pub(crate) const AIM_HYPOTHESIS_ID: HypothesisId = HypothesisId::AimModeTransitionsV1;

pub(crate) fn aim_packet(character: &str) -> Option<&'static str> {
    match character {
        "character:1033" => Some("GE_Player_Adler_Aim_Damage"),
        "character:1010" => Some("GE_Player_Nanally_BranchPraojectile_Damage"),
        "character:1055" => Some("GE_Player_Kuhara_AimAttack_Damage"),
        _ => None,
    }
}

pub(crate) fn validate(character: &str, input: PlayerInput) -> Result<(), String> {
    if input == PlayerInput::Jump {
        return if matches!(
            character,
            "character:1023"
                | "character:1019"
                | "character:1021"
                | "character:1039"
                | "character:1036"
                | "character:1020"
                | "character:1025"
                | "character:1042"
                | "character:1054"
                | "character:1071"
                | "character:1073"
                | "character:1010"
                | "character:1075"
        ) {
            Ok(())
        } else {
            Err(format!("Unsupported jump model for {character}"))
        };
    }
    if input == PlayerInput::Arc {
        return Err("Arc requires a supported equipped weapon".into());
    }
    if input == PlayerInput::Dash
        || character == "character:1004"
        || aim_packet(character).is_some()
    {
        Ok(())
    } else {
        Err(format!("Unsupported {input:?} model for {character}"))
    }
}

pub(crate) fn special_ability(actor: &ActorRuntime) -> Option<(&'static str, &'static str)> {
    match &actor.character {
        super::RuntimeState::Lacrimosa(state) => Some((
            "GA_Lacrimosa_Melee",
            if state.melee {
                "GE_Player_Lacrimosa_Pan_Damage"
            } else {
                "GE_Player_Lacrimosa_SwitchModB_Damage"
            },
        )),
        _ => None,
    }
}

pub(crate) fn apply(
    actor: &mut ActorRuntime,
    input: PlayerInput,
    request: &crate::simulation::GameScenarioRequest,
) {
    if let super::RuntimeState::Baicang(state) = &mut actor.character {
        state.riposte_ready = false;
    }
    if let super::RuntimeState::Nanally(state) = &mut actor.character {
        state.riposte_ready = false;
    }
    if let super::RuntimeState::Mint(state) = &mut actor.character {
        state.riposte_ready = false;
    }
    if let super::RuntimeState::Sakiri(state) = &mut actor.character {
        state.riposte_ready = false;
    }
    if let super::RuntimeState::Iroi(state) = &mut actor.character {
        state.riposte_ready = false;
    }
    match input {
        PlayerInput::Arc => {}
        PlayerInput::Jump => super::baicang::cancel::on_jump(actor),
        PlayerInput::Special => actor.aiming = !actor.aiming,
        PlayerInput::Dash => {
            actor.aiming = false;
            if let super::RuntimeState::Lacrimosa(state) = &mut actor.character {
                state.bat_evade_until_us = actor.locked_until_us;
                state.normal_stage = 0;
                state.normal_started_us = None;
            }
            if let super::RuntimeState::Baicang(state) = &mut actor.character {
                state.normal_stage = 0;
                state.normal_started_us = None;
                state.heavy_dodge_until_us = actor.locked_until_us.saturating_add(2_000_000);
                state.cancel_window = None;
                state.clock.end_action();
                state.derived_skill_stage = 3;
                state.derived_skill_stage_expires_at_us =
                    actor.locked_until_us.saturating_add(2_000_000);
                if super::baicang::words::accepted(request)
                    || super::baicang::riposte::accepted(request)
                    || !request.hold_inputs.is_empty()
                {
                    state.dodge_immune_until_us =
                        actor.last_action_start_us.saturating_add(350_000);
                }
            }
        }
    }
}

pub(crate) fn attack_mode(actor: &ActorRuntime) -> &'static str {
    if let super::RuntimeState::Lacrimosa(state) = &actor.character {
        if state.melee { "melee" } else { "ranged" }
    } else if actor.aiming {
        "aim"
    } else {
        "normal"
    }
}
