//! Exact ordinary Crime is independent from Ultimate charge and Esper Cycle.
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest, state::ActorRuntime};
use crate::{HypothesisId, runtime_api::ActorHitContext};
use ntesim_core::kernel::state::{ExactPool, ExactPoolSummary};
use std::collections::BTreeMap;

pub(crate) const MODEL: HypothesisId = HypothesisId::ChaosOrdinaryCrimeV1;

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if accepted(request) {
        actor.character.chaos_mut()?.crime = Some(initial_pool(config.initial_resources.crime)?);
    }
    super::clock::initialize(request, actor)
}

pub(crate) fn initial_pool(initial: Option<crate::formula::Ratio>) -> Result<ExactPool, String> {
    ExactPool::new(
        ExactFraction::new(1000, 1)?,
        initial
            .map(ExactFraction::from_ratio)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?),
    )
}

pub(crate) fn record_initial(
    config: &GameActorConfig,
    request: &GameScenarioRequest,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if config.character_id == "character:1071" && accepted(request) {
        crate::runtime_api::record_hypothesis_id(request, assumptions, MODEL)?;
    }
    Ok(())
}

pub(crate) fn summary(
    state: &super::skill::State,
) -> Result<BTreeMap<String, ExactPoolSummary>, String> {
    match &state.crime {
        Some(pool) => Ok(BTreeMap::from([("chaos.crime".into(), pool.summary()?)])),
        None => Ok(BTreeMap::new()),
    }
}

// BP_ChaosLogicObj's DT_ChaosEffectFigure GE-key lookup. Each mapped primary is
// consumed by on_hit and independently summed across the twelve-packet chain.
fn amount(effect: &str) -> u128 {
    let effect = match effect {
        "GE_Player_Chaos_Melee2_Damage_Ultra" => "GE_Player_Chaos_Melee2_1_Damage",
        "GE_Player_Chaos_Melee2_1_Damage_Ultra" => "GE_Player_Chaos_Melee2_Damage",
        _ => effect.strip_suffix("_Ultra").unwrap_or(effect),
    };
    match effect {
        "GE_Player_Chaos_Melee1_Damage" => 22,
        "GE_Player_Chaos_Melee2_Damage" => 33,
        "GE_Player_Chaos_Melee2_1_Damage" => 15,
        "GE_Player_Chaos_Melee3_Damage" => 28,
        "GE_Player_Chaos_Melee4_Damage" => 35,
        "GE_Player_Chaos_Melee4_1_Damage" => 13,
        "GE_Player_Chaos_Melee4_2_Damage" => 19,
        "GE_Player_Chaos_Melee5_Damage" => 41,
        "GE_Player_Chaos_Melee5_1_Damage" => 94,
        "GE_Player_Chaos_Skill1_Damage" | "GE_Player_Chaos_Skill2_Damage" => 50,
        "GE_Player_Chaos_Airattack_Damage" => 30,
        "GE_Player_Chaos_PerfectEvadeAttack1_Damage" => 100,
        _ => 0,
    }
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.applied_hp_loss == 0 || ctx.actor.hp == 0 || !accepted(ctx.request) {
        return Ok(());
    }
    let mapped = amount(ctx.damage_effect_id);
    if mapped == 0 {
        return Ok(());
    }
    let group = ctx
        .packet_group_id
        .ok_or("Chaos Crime requires a causal primary packet group")?;
    let marked = ctx.config.awakening.has_effect(1)
        && crate::simulation::state::target_has_status(
            ctx.buffs,
            ctx.actor_index,
            &ctx.config.character_id,
            ctx.target_index,
            "chaos.warrant",
        );
    let state = ctx.actor.character.chaos_mut()?;
    let base = if state.crime_packets.contains(group) {
        0
    } else {
        mapped
    };
    let bonus = if marked && !state.crime_a1_packets.contains(group) {
        ExactFraction::new(mapped * 2, 5)?
    } else {
        ExactFraction::new(0, 1)?
    };
    let gain = ExactFraction::new(base, 1)?.add(bonus)?;
    if gain.is_zero() {
        return Ok(());
    }
    let (applied, overflow) = state
        .crime
        .as_mut()
        .ok_or("Chaos exact Crime pool missing")?
        .grant(gain)?;
    state.crime_packets.insert(group.into());
    if marked {
        state.crime_a1_packets.insert(group.into());
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.time_us,
        phase: "HitDelivery".into(),
        event_kind: "chaos_crime_gain".into(),
        active_actor: ctx.active_actor_index,
        hit_index: None,
        from_actor: Some(ctx.actor_index),
        to_actor: Some(ctx.actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            packet_group_id: Some(group.into()),
            exact_amount: Some(applied),
            exact_overflow: Some(overflow),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn cycle(
    request: &GameScenarioRequest,
    effect: &str,
) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
    if !accepted(request) || !effect.starts_with("GE_Player_Chaos_") {
        return Ok(None);
    }
    ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
        .map(|raw| {
            Ok(crate::runtime_api::PacketCycleGrant {
                raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                hypothesis: HypothesisId::CombatPacketCycleAccrualV1,
            })
        })
        .transpose()
}
