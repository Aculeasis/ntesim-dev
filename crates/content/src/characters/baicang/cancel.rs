//! Owner cancel windows modify recovery admission, never the continuing real clock.
use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::{GameScenarioRequest, events::GameEventData, state::ActorRuntime};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BaicangCancelWindowsV1;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Window {
    pub(super) skill_at_us: Option<u64>,
    pub(super) jump_at_us: Option<u64>,
    pub(super) command_index: usize,
    pub(super) ability_id: &'static str,
    pub(super) start_us: u64,
    pub(super) hold_us: u64,
    pub(super) emission: EmissionKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EmissionKind {
    Direct,
    Branch1,
    Branch2,
    Skill,
    SkillA,
    SkillB,
    Burst,
    Skia,
}

impl Window {
    pub(super) fn emitted_at(self, effect: &str, impact: u64) -> Result<u64, String> {
        let offset = match (self.emission, effect) {
            (EmissionKind::Branch1, super::branches::BRANCH1_1_PACKET) => {
                Some(self.hold_us + 486_667)
            }
            (EmissionKind::Branch2, super::branches::BRANCH2_1_PACKET) => {
                return impact
                    .checked_sub(450_000)
                    .ok_or("Baicang missile emission underflow".into());
            }
            (EmissionKind::Skill, _) => Some(18_070),
            (EmissionKind::SkillB, _) => Some(16_652),
            (EmissionKind::Skia, _) => Some(378_030),
            (EmissionKind::SkillA, "GE_Player_Cang_SkillA_Damage1") => Some(216_575),
            _ => None,
        };
        offset.map_or(Ok(impact), |offset| {
            self.start_us
                .checked_add(offset)
                .ok_or("Baicang emission overflow".into())
        })
    }

    pub(super) fn first_skill_emission(self) -> Result<Option<u64>, String> {
        let offset = match self.emission {
            EmissionKind::Skill => Some(18_070),
            EmissionKind::SkillA => Some(216_575),
            EmissionKind::SkillB => Some(16_652),
            _ => None,
        };
        offset
            .map(|offset| {
                self.start_us
                    .checked_add(offset)
                    .ok_or("Baicang Skill emission overflow".into())
            })
            .transpose()
    }
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    super::clock::validate(request)?;
    if accepted(request) && !super::words::accepted(request) {
        return Err("baicang.cancel_windows_v1 requires baicang.power_words_v1".into());
    }
    Ok(())
}

pub(crate) fn capture(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request)
        && !super::interrupt::accepted(ctx.request)
        && !super::clock::accepted(ctx.request)
    {
        return Ok(());
    }
    let packet_owned = ctx
        .queue
        .iter()
        .filter_map(|event| match &event.data {
            GameEventData::HitDelivery {
                command_index,
                actor_index,
                target_index,
                damage_effect_id,
                ..
            } if *command_index == ctx.command_index
                && *actor_index == ctx.actor_index
                && *target_index == ctx.target_index =>
            {
                Some((event.time_us, event.seq, damage_effect_id.clone()))
            }
            _ => None,
        })
        .min_by_key(|(time, seq, _)| (*time, *seq))
        .map(|(_, _, packet)| packet);
    let packet = packet_owned.as_deref();
    let hold = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .map_or(0, |input| input.duration_us);
    let (mut skill, mut jump) = match packet {
        Some("GE_Player_Cang_Melee1_Damage") => (Some(378_894), None),
        Some("GE_Player_Cang_Melee2_Damage") => (Some(379_190), None),
        Some("GE_Player_Cang_Melee3_Damage") => (Some(477_995), None),
        Some(super::branches::BRANCH1_PACKET) => (None, Some(hold + 506_294)),
        Some(super::branches::BRANCH2_PACKET) => (None, Some(hold + 300_989)),
        Some("GE_Player_Cang_Skill_Damage") => (None, Some(249_464)),
        Some("GE_Player_Cang_SkillA_Damage") => (None, Some(247_532)),
        Some("GE_Player_Cang_SkillB_Damage") => (None, Some(247_532)),
        Some("GE_Player_Cang_UltraSkill2_Damage") => (None, Some(2_871_112)),
        _ => (None, None),
    };
    if super::clock::accepted(ctx.request) {
        skill = skill.map(|time| {
            time - if packet == Some("GE_Player_Cang_Melee3_Damage") {
                60_300
            } else {
                29_700
            }
        });
        if packet == Some(super::branches::BRANCH1_PACKET) {
            jump = Some(hold + 436_294);
        }
    }
    let emission = match packet {
        Some(super::branches::BRANCH1_PACKET) => EmissionKind::Branch1,
        Some(super::branches::BRANCH2_PACKET) => EmissionKind::Branch2,
        Some("GE_Player_Cang_Skill_Damage") => EmissionKind::Skill,
        Some("GE_Player_Cang_SkillA_Damage") => EmissionKind::SkillA,
        Some("GE_Player_Cang_SkillB_Damage") => EmissionKind::SkillB,
        Some("GE_Player_Cang_UltraSkill2_Damage") => EmissionKind::Burst,
        Some(super::etd::SKIA_PACKET) => EmissionKind::Skia,
        _ => EmissionKind::Direct,
    };
    let ability_id = match ctx.action_metadata.ability_id.as_str() {
        "GA_Cang_Melee" => "GA_Cang_Melee",
        "GA_Cang_Skill" => "GA_Cang_Skill",
        "GA_Cang_UltraSkill" => "GA_Cang_UltraSkill",
        "GA_Cang_QTE" => "GA_Cang_QTE",
        _ => return Err("Baicang cancellation requires a configured native action".into()),
    };
    let deadline = |offset: u64| {
        ctx.time_us
            .checked_add(offset)
            .ok_or("Baicang cancel deadline overflow".to_owned())
    };
    ctx.actor.character.baicang_mut()?.cancel_window = Some(Window {
        command_index: ctx.command_index,
        ability_id,
        start_us: ctx.time_us,
        hold_us: hold,
        emission,
        skill_at_us: skill.map(deadline).transpose()?,
        jump_at_us: jump.map(deadline).transpose()?,
    });
    Ok(())
}

pub(crate) fn recovery_override(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, HypothesisId)> {
    if !accepted(request) || now > actor.locked_until_us {
        return None;
    }
    let crate::characters::RuntimeState::Baicang(state) = &actor.character else {
        return None;
    };
    let window = state.cancel_window?;
    let deadline = match ability {
        "jump" => window.jump_at_us,
        "GA_Cang_Skill" => window.skill_at_us,
        _ => None,
    }?;
    Some((deadline.min(actor.locked_until_us), HYPOTHESIS_ID))
}

pub(crate) fn on_jump(actor: &mut ActorRuntime) {
    if let crate::characters::RuntimeState::Baicang(state) = &mut actor.character {
        state.cancel_window = None;
        state.clock.end_action();
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.heavy_dodge_until_us = 0;
        state.riposte_ready = false;
        state.derived_skill_stage_expires_at_us =
            actor.last_action_start_us.saturating_add(2_000_000);
    }
}
