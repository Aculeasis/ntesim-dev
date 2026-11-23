use crate::HypothesisId;
use crate::runtime_api::CastContext;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) full_kit: bool,
    pub(crate) riposte_ready: bool,
    pub(crate) marks: std::collections::BTreeMap<usize, super::effects::Mark>,
    pub(crate) transfers: std::collections::BTreeMap<String, super::effects::Transfer>,
    pub(crate) mark_generation: u64,
    pub(crate) membership: Option<Box<super::stats::Membership>>,
    pub(crate) out_of_combat: bool,
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) lilith_until_us: Option<u64>,
    pub(crate) lilith_stage: usize,
    pub(crate) high_q: bool,
}

pub(crate) const CHAIN_HYPOTHESIS_ID: HypothesisId = HypothesisId::FadiaNormalChainV1;

pub(crate) const BURST_HYPOTHESIS_ID: HypothesisId = HypothesisId::FadiaFullHpInitialBurstV1;

pub(crate) const ALL_IN_RANGE_HYPOTHESIS_ID: HypothesisId = HypothesisId::FadiaAoeAllInRangeDummyV1;

pub(crate) fn validate_admission(
    config: &crate::simulation::GameActorConfig,
    actor: &crate::simulation::state::ActorRuntime,
    ability_id: &str,
    packet_effect: &str,
    time_us: u64,
) -> Result<(), String> {
    if config.character_id != "character:1039" {
        return Ok(());
    }
    let state = match &actor.character {
        crate::characters::RuntimeState::Fadia(state) => state,
        _ => return Err("Fadia runtime extension unavailable".into()),
    };
    if state.full_kit {
        if state.out_of_combat {
            return Err("Fadia action requires being in combat".into());
        }
        return super::clock::validate_live(state, ability_id, packet_effect, time_us);
    }
    let in_lilith = state.lilith_until_us.is_some_and(|expiry| time_us < expiry);
    if in_lilith && matches!(ability_id, "GA_Fadia_Melee" | "GA_Fadia_UltraSkill") {
        return Err("Fadia Normal and Burst are unsupported during the Lilith form window".into());
    }
    match ability_id {
        "GA_Fadia_Melee" if packet_effect == "GE_Player_Fadia_Melee1_Damage" => Ok(()),
        "GA_Fadia_Melee" => {
            Err("Fadia normal chain requires its modeled first packet as input".into())
        }
        "GA_Fadia_UltraSkill" => {
            if packet_effect != "GE_Player_Fadia_UltraSkill1_Damage" {
                return Err(
                    "Fadia initial Burst requires native UltraSkill1 as its primary packet".into(),
                );
            }
            if config.awakening.count() != 0 {
                return Err("Fadia initial Burst model supports awakening 0 only".into());
            }
            if actor.hp != actor.hp_max {
                return Err("Fadia initial Burst model requires exactly full runtime HP".into());
            }
            Ok(())
        }
        "GA_Fadia_Skill" => Err("Fadia Skill HP-scaled behavior is unsupported".into()),
        "GA_Fadia_QTE" => {
            Err("Fadia Support requires full_kit_v1 and qualified engine entry".into())
        }
        _ => Ok(()),
    }
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if super::clock::accepted(ctx.request) {
        return super::clock::cast(ctx);
    }
    if ctx.config.character_id != "character:1039" {
        return Ok(());
    }

    let state = ctx.actor.character.fadia_mut()?;
    if state
        .lilith_until_us
        .is_some_and(|expiry| ctx.time_us >= expiry)
    {
        state.lilith_until_us = None;
    }

    if ctx.ability_id == "GA_Fadia_Melee" {
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != super::primary_packet(ctx.ability_id)
        {
            return Err(
                "Fadia normal chain requires the modeled first packet as its admitted input".into(),
            );
        }
        ctx.record_hypothesis(CHAIN_HYPOTHESIS_ID)?;
        let state = ctx.actor.character.fadia_mut()?;
        state.normal_stage = crate::characters::normal::after_gap(
            state.normal_stage,
            state.normal_started_us,
            ctx.time_us,
        );
        let stage = state.normal_stage;
        state.normal_stage = (stage + 1) % 5;
        state.normal_started_us = Some(ctx.time_us);
        let packets: &[&str] = match stage {
            0 => &["GE_Player_Fadia_Melee1_Damage"],
            1 => &[
                "GE_Player_Fadia_Melee2_Damage",
                "GE_Player_Fadia_Melee3_Damage",
            ],
            2 => &["GE_Player_Fadia_Melee4_Damage"],
            3 => &[
                "GE_Player_Fadia_Melee5_Damage",
                "GE_Player_Fadia_Melee6_Damage",
            ],
            _ => &[
                "GE_Player_Fadia_Melee7_Damage",
                "GE_Player_Fadia_Melee8_Damage",
            ],
        };
        ctx.replace_native_primary_packet(packets[0])?;
        for packet in &packets[1..] {
            ctx.schedule_native_child_packet(packet, ctx.target_index, ctx.hit_time_us)?;
        }
    } else if ctx.ability_id == "GA_Fadia_UltraSkill" {
        let state = ctx.actor.character.fadia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        if ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != super::primary_packet(ctx.ability_id)
        {
            return Err(
                "Fadia initial Burst requires native UltraSkill1 as its primary packet".into(),
            );
        }
        if ctx.config.awakening.count() != 0 {
            return Err("Fadia initial Burst model supports awakening 0 only".into());
        }
        if ctx.actor.hp != ctx.actor.hp_max {
            return Err("Fadia initial Burst model requires exactly full runtime HP".into());
        }
        ctx.record_hypothesis(BURST_HYPOTHESIS_ID)?;
        for _ in 0..4 {
            ctx.schedule_native_child_packet(
                "GE_Player_Fadia_UltraSkill2_Damage",
                ctx.target_index,
                ctx.hit_time_us,
            )?;
        }
        ctx.schedule_native_child_packet(
            "GE_Player_Fadia_UltraSkill3_Damage",
            ctx.target_index,
            ctx.hit_time_us,
        )?;
        let expires = ctx
            .time_us
            .checked_add(5_000_000)
            .ok_or("Fadia Lilith expiry overflow")?;
        ctx.actor.character.fadia_mut()?.lilith_until_us = Some(expires);
    } else {
        let state = ctx.actor.character.fadia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.lilith_until_us = None;
    }

    if ctx.request.targets.len() > 1 && ctx.ability_id == "GA_Fadia_UltraSkill" {
        ctx.record_hypothesis(ALL_IN_RANGE_HYPOTHESIS_ID)?;
        for target_index in 0..ctx.request.targets.len() {
            if target_index == ctx.target_index {
                continue;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Fadia_UltraSkill1_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
            for _ in 0..4 {
                ctx.schedule_native_child_packet(
                    "GE_Player_Fadia_UltraSkill2_Damage",
                    target_index,
                    ctx.hit_time_us,
                )?;
            }
            ctx.schedule_native_child_packet(
                "GE_Player_Fadia_UltraSkill3_Damage",
                target_index,
                ctx.hit_time_us,
            )?;
        }
    }

    Ok(())
}
