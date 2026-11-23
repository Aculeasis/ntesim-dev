//! B2 pending-settlement value, captured base-ATK cap and bounded Nova lineage.
use crate::HypothesisId;
use crate::runtime_api::NovaEndEffect;
use crate::simulation::{
    ExactFraction, GameScenarioRequest, metadata::ActionPacketMetadata, state::GameSimulatorState,
};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::BlackbirdB2NovaV1;
pub(crate) const PACKET: &str = "GE_Reaction_4_new_1042_Damage";

#[derive(Clone, Debug)]
pub(crate) struct Accumulator {
    pub(crate) until_us: u64,
    pub(crate) excluded_command: Option<usize>,
    pub(crate) applied_hp: u128,
    pub(crate) cap: ExactFraction,
    pub(crate) reapply: bool,
}

pub(crate) fn validate(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<bool, String> {
    let mut enabled = false;
    for (owner, actor) in request.actors.iter().enumerate() {
        if eligible(actor, state.actors[owner].hp)? {
            crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
            enabled = true;
        }
    }
    Ok(enabled)
}

fn eligible(config: &crate::simulation::GameActorConfig, hp: u64) -> Result<bool, String> {
    if config.character_id != "character:1042" || hp == 0 {
        return Ok(false);
    }
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    Ok(progression["breakthrough"].as_u64().unwrap_or(0) >= 2)
}

pub(crate) fn create(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    target: usize,
    generation: u64,
    command: usize,
    until_us: u64,
    reapply: bool,
) -> Result<(), String> {
    let now = state.current_time_us;
    let mut assumptions = BTreeMap::new();
    validate(request, state, &mut assumptions)?;
    for (owner, config) in request.actors.iter().enumerate() {
        if !eligible(config, state.actors[owner].hp)? {
            continue;
        }
        let ledger = &mut state.actors[owner].character.blackbird_mut()?.nova;
        // Authored replacement closes the old window at admission, before
        // same-time packets belonging to the new generation can be observed.
        for ((recipient, _), old) in ledger.iter_mut() {
            if *recipient == target {
                old.until_us = old.until_us.min(now);
            }
        }
        ledger.insert(
            (target, generation),
            Accumulator {
                until_us,
                excluded_command: reapply.then_some(command),
                applied_hp: 0,
                cap: ExactFraction::new(u128::from(config.static_stats.base_atk) * 1000, 1)?,
                reapply,
            },
        );
    }
    state.execution_model_assumptions.extend(assumptions);
    Ok(())
}

pub(crate) fn observe(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    target: usize,
    command: usize,
    metadata: &ActionPacketMetadata,
    applied_hp: u64,
) -> Result<(), String> {
    if applied_hp == 0
        || matches!(
            metadata.packet.damage_effect_id.as_str(),
            "ntesim.reaction.nova" | PACKET
        )
    {
        return Ok(());
    }
    let now = state.current_time_us;
    for (owner, config) in request.actors.iter().enumerate() {
        if config.character_id != "character:1042" || state.actors[owner].hp == 0 {
            continue;
        }
        for ((recipient, _), entry) in &mut state.actors[owner].character.blackbird_mut()?.nova {
            if *recipient == target
                && now < entry.until_us
                && entry.excluded_command != Some(command)
            {
                entry.applied_hp = entry
                    .applied_hp
                    .checked_add(u128::from(applied_hp))
                    .ok_or("Blackbird Nova HP accumulator overflow")?;
            }
        }
    }
    Ok(())
}

pub(crate) fn end(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    owner: usize,
    target: usize,
    generation: u64,
) -> Result<Option<NovaEndEffect>, String> {
    let entry = state.actors[owner]
        .character
        .blackbird_mut()?
        .nova
        .remove(&(target, generation));
    let Some(entry) = entry else {
        return Ok(None);
    };
    if !eligible(&request.actors[owner], state.actors[owner].hp)?
        || state.targets[target].hp_current == 0
    {
        return Ok(None);
    }
    // DT_ReactionExtendDataTable1042 supplies.20 and1000*baseATK. The
    // selected pending getter already includes both; there is no second*.20.
    let uncapped = ExactFraction::new(entry.applied_hp, 5)?;
    let threshold = entry
        .cap
        .numerator
        .checked_mul(uncapped.denominator)
        .ok_or("Blackbird Nova cap comparison overflow")?;
    let pending_damage = if uncapped.numerator > threshold {
        entry.cap
    } else {
        uncapped
    };
    Ok(Some(NovaEndEffect {
        owner_actor_index: owner,
        pending_damage,
        damage_effect_id: PACKET,
        ability_id: "GA_BlackBird_Passive_1",
        hypothesis: HYPOTHESIS_ID,
        reapply: entry.reapply,
    }))
}
