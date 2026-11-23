use crate::simulation::{ExactFraction, GameActorConfig, state::ActorRuntime};
use ntesim_core::kernel::state::ExactPool;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizExactGrainV1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GrantKey {
    Normal,
    Barren,
    Plunge,
    Riposte1,
    Riposte2,
}

pub(super) fn capacity(config: &GameActorConfig) -> Result<ExactFraction, String> {
    capacity_at(config.character_level, &config.awakening)
}

fn capacity_at(
    level: u32,
    awakening: &ntesim_data::AwakeningSelection,
) -> Result<ExactFraction, String> {
    let mut cap = ExactFraction::new(super::values::at_level(level)?.capacity.into(), 1)?;
    if awakening.has_effect(1) {
        cap = cap.mul(super::parameters::PARAMETERS.capacity_multiplier)?;
    }
    Ok(cap)
}

pub(crate) fn initial_pool(
    level: u32,
    awakening: &ntesim_data::AwakeningSelection,
    initial: Option<crate::formula::Ratio>,
) -> Result<ExactPool, String> {
    let value = initial
        .map(ExactFraction::from_ratio)
        .transpose()?
        .unwrap_or(ExactFraction::new(0, 1)?);
    ExactPool::new(capacity_at(level, awakening)?, value)
        .map_err(|error| format!("Chiz initial Grains/Loans exceed the owner capacity: {error}"))
}

pub(crate) fn initialize(config: &GameActorConfig, actor: &mut ActorRuntime) -> Result<(), String> {
    let grain = initial_pool(
        config.character_level,
        &config.awakening,
        config.initial_resources.grains,
    )?;
    let state = actor.character.chiz_mut()?;
    state.grain = Some(grain);
    Ok(())
}
pub(crate) fn grant_normal(
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
    effect: &str,
    command_index: usize,
) -> Result<(), String> {
    let key = match effect {
        super::barren::PRIMARY | super::barren::SECONDARY => GrantKey::Barren,
        "GE_Player_Chiichan073_AirAttack1_Damage" => GrantKey::Plunge,
        "GE_Player_Chiichan073_PerfectEvadeAttack1_Damage" => GrantKey::Riposte1,
        "GE_Player_Chiichan073_PerfectEvadeAttack2_Damage" => GrantKey::Riposte2,
        _ => {
            let Some(segment) = effect
                .strip_prefix("GE_Player_Chiichan073_Melee")
                .and_then(|s| s.strip_suffix("_Damage"))
            else {
                return Ok(());
            };
            if !matches!(segment, "1" | "2" | "3" | "4" | "5") {
                return Err("UNSUPPORTED_CHIZ_GRAIN_PACKET: normal hit segment unknown".into());
            }
            GrantKey::Normal
        }
    };
    let state = actor.character.chiz_mut()?;
    if state.grain_inputs.contains(&(command_index, key)) {
        return Ok(());
    }
    let mut amount = ExactFraction::new(
        super::values::at_level(config.character_level)?
            .normal_grant
            .into(),
        1,
    )?;
    if let Some(finance) = &mut state.finance {
        amount = amount.mul(finance.acquisition_multiplier(config)?)?;
    } else if config.awakening.has_effect(4) {
        amount = amount.mul(super::parameters::PARAMETERS.normal_acquisition_multiplier)?;
    }
    if let Some(finance) = &mut state.finance {
        finance.acquire(amount)?;
    } else {
        state
            .grain
            .as_mut()
            .ok_or("Chiz Grain not initialized")?
            .grant(amount)?;
    }
    state.grain_inputs.insert((command_index, key));
    Ok(())
}

pub(crate) fn summary(
    state: &super::awakenings::State,
) -> Result<
    std::collections::BTreeMap<String, ntesim_core::kernel::state::ResourcePoolSummary>,
    String,
> {
    if let Some(finance) = &state.finance {
        let mut summaries = std::collections::BTreeMap::from([(
            "chiz.grain".into(),
            finance.wallet.summary()?.into(),
        )]);
        if let Some(debt) = &finance.debt {
            summaries.insert("chiz.loan_debt".into(), debt.summary()?.into());
        }
        if let Some(market) = &finance.market {
            summaries.insert("chiz.market".into(), market.wallet.summary()?.into());
        }
        return Ok(summaries);
    }
    match &state.grain {
        Some(grain) => Ok(std::collections::BTreeMap::from([(
            "chiz.grain".into(),
            grain.summary()?.into(),
        )])),
        None => Ok(Default::default()),
    }
}
