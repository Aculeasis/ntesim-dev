use crate::HypothesisId;
pub(crate) mod projection;
use crate::runtime_api::DamageContext;
use crate::simulation::{ExactFraction, GameScenarioRequest, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffManager, BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsStreetBoxerV2;
const POWERED_MODIFIER: &str = "street_boxer.crit_rate";

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    let mut total = ExactFraction::new(0, 1)?;
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::StreetBoxerCrit {
            crit_rate_bonus,
            baseline_in_stats,
            ..
        } = proc
        {
            let bonus = ctx
                .buff_stats
                .get(POWERED_MODIFIER)
                .copied()
                .unwrap_or(*crit_rate_bonus);
            let mut applied = ExactFraction::from_ratio(bonus)?;
            if *baseline_in_stats {
                applied = applied.sub(ExactFraction::from_ratio(*crit_rate_bonus)?)?;
            }
            total = total.add(applied)?;
        }
    }
    Ok(total)
}

pub(crate) fn on_team_reaction(
    request: &GameScenarioRequest,
    actor_index: usize,
    time_us: u64,
    reaction: &ntesim_data::reaction_values::ReactionResult,
    buffs: &mut BuffManager,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<bool, String> {
    use ntesim_data::reaction_values::ReactionResult;
    if !matches!(reaction, ReactionResult::Stain | ReactionResult::Remora) {
        return Ok(false);
    }
    let mut triggered = false;
    for proc in &request.actors[actor_index].gear_procs {
        let GearProcConfig::StreetBoxerCrit {
            powered_crit_rate_bonus,
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
                "street_boxer.reaction_window",
                "set:Suit5",
                BuffRecipient::SelfActor,
                *duration_us,
                BuffStackingScope::PerSource,
                BTreeMap::from([(
                    POWERED_MODIFIER.into(),
                    ExactFraction::from_ratio(*powered_crit_rate_bonus)?,
                )]),
            ),
            actor_index,
            time_us,
        )?;
        triggered = true;
    }
    Ok(triggered)
}

pub(crate) fn validate_config(
    proc: &GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let GearProcConfig::StreetBoxerCrit {
        crit_rate_bonus,
        powered_crit_rate_bonus,
        duration_us,
        baseline_in_stats,
    } = proc
    else {
        return Err("Street Boxer validator received the wrong proc kind".into());
    };
    if ctx.actor.set_id.as_deref() != Some("set:Suit5") {
        return Err("Street Boxer requires equipped set:Suit5".into());
    }
    for ratio in [crit_rate_bonus, powered_crit_rate_bonus] {
        let value = ExactFraction::from_ratio(*ratio)?;
        if value.is_zero() || value.numerator > value.denominator {
            return Err("Street Boxer requires a positive fractional crit bonus <= 1".into());
        }
    }
    if *duration_us == 0 {
        return Err("Street Boxer requires positive reaction duration".into());
    }
    if *baseline_in_stats {
        let base = ExactFraction::from_ratio(*crit_rate_bonus)?;
        if ExactFraction::from_ratio(*powered_crit_rate_bonus)?
            .try_cmp(base)
            .map_err(String::from)?
            .is_lt()
            || ExactFraction::from_ratio(ctx.actor.static_stats.crit_rate)?
                .try_cmp(base)
                .map_err(String::from)?
                .is_lt()
        {
            return Err("Street Boxer compiled baseline requires sufficient static CR and a powered replacement >= baseline".into());
        }
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "street_boxer",
        duration_us: *duration_us,
    })
}
