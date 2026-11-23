use crate::HypothesisId;
pub(crate) mod projection;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, PacketMetadata};
use crate::simulation::{ExactFraction, GameScenarioRequest, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffManager, BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsDiabolosV2;
const POWERED_MODIFIER: &str = "diabolos.chaos_res_ignore";

fn eligible_reaction(reaction: &ntesim_data::reaction_values::ReactionResult) -> bool {
    matches!(
        reaction,
        ntesim_data::reaction_values::ReactionResult::Nova
            | ntesim_data::reaction_values::ReactionResult::Scorch
    )
}

pub(crate) fn on_reaction_participation(
    request: &GameScenarioRequest,
    actor_index: usize,
    time_us: u64,
    reaction: &ntesim_data::reaction_values::ReactionResult,
    buffs: &mut BuffManager,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<bool, String> {
    if !eligible_reaction(reaction) {
        return Ok(false);
    }
    let mut triggered = false;
    for proc in &request.actors[actor_index].gear_procs {
        let GearProcConfig::DiabolosChaosIgnore {
            powered_chaos_res_ignore,
            duration_us,
            ..
        } = proc
        else {
            continue;
        };
        crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
        crate::simulation::state::apply_timed_buff(
            buffs,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "diabolos.reaction_window",
                "set:Suit1",
                BuffRecipient::SelfActor,
                *duration_us,
                BuffStackingScope::PerSource,
                BTreeMap::from([(
                    POWERED_MODIFIER.into(),
                    ExactFraction::from_ratio(*powered_chaos_res_ignore)?,
                )]),
            ),
            actor_index,
            time_us,
        )?;
        triggered = true;
    }
    Ok(triggered)
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if packet.element != Some(DamageElement::Chaos) {
        return Ok(modifiers);
    }
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::DiabolosChaosIgnore {
            chaos_res_ignore, ..
        } = proc
        else {
            continue;
        };
        let value = ctx
            .buff_stats
            .get(POWERED_MODIFIER)
            .copied()
            .unwrap_or(*chaos_res_ignore);
        modifiers = modifiers.add_resistance_reduction(ExactFraction::from_ratio(value)?)?;
    }
    Ok(modifiers)
}

pub(crate) fn validate_config(
    proc: &GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let GearProcConfig::DiabolosChaosIgnore {
        chaos_res_ignore,
        powered_chaos_res_ignore,
        duration_us,
    } = proc
    else {
        return Err("Diabolos validator received the wrong proc kind".into());
    };
    if ctx.actor.set_id.as_deref() != Some("set:Suit1") {
        return Err("Diabolos requires equipped set:Suit1".into());
    }
    for ratio in [chaos_res_ignore, powered_chaos_res_ignore] {
        let value = ExactFraction::from_ratio(*ratio)?;
        if value.is_zero() || value.numerator > value.denominator {
            return Err("Diabolos requires a positive fractional Chaos ignore <= 1".into());
        }
    }
    if *duration_us == 0 {
        return Err("Diabolos requires positive reaction duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "diabolos",
        duration_us: *duration_us,
    })
}
