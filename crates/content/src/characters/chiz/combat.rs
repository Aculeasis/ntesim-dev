//! Battle entry IDs own A5 deposits; field entry never rearms them.
use crate::runtime_api::CallbackContext;
use crate::simulation::{
    ExactFraction, GameActorCombatTransitionReceipt, GameResourceDepositReceipt,
    GameScenarioRequest,
};
use ntesim_core::kernel::state::wide_fraction;
use num_rational::BigRational;
use num_traits::Zero;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizBattleEntryV1;
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        != Some(MODEL.description())
    {
        return Err("Chiz combat transitions require battle_entry_v1".into());
    }
    Ok(())
}
fn receipt(
    input: BigRational,
    repaid: BigRational,
    credited: BigRational,
) -> GameResourceDepositReceipt {
    GameResourceDepositReceipt {
        input: (&input).into(),
        repaid: (&repaid).into(),
        credited: (&credited).into(),
        overflow: (&(&input - &repaid - &credited)).into(),
    }
}
/// Initial entry uses this same writer after savings and principal are seeded.
pub(crate) fn deposit(
    config: &crate::simulation::GameActorConfig,
    actor: &mut crate::simulation::state::ActorRuntime,
    entry: usize,
) -> Result<GameResourceDepositReceipt, String> {
    let state = actor.character.chiz_mut()?;
    if state.battle_entries.contains(&entry) || !config.awakening.has_effect(5) {
        state.battle_entries.insert(entry);
        return Ok(receipt(
            BigRational::zero(),
            BigRational::zero(),
            BigRational::zero(),
        ));
    }
    let cap = super::resource::capacity(config)?;
    let result = if let Some(finance) = &mut state.finance {
        let balance = finance.balance().clone();
        let debt = finance
            .debt
            .as_ref()
            .map_or_else(BigRational::zero, |debt| debt.current().clone());
        finance.acquire(cap)?;
        let repaid = debt
            - finance
                .debt
                .as_ref()
                .map_or_else(BigRational::zero, |debt| debt.current().clone());
        receipt(wide_fraction(cap)?, repaid, finance.balance() - balance)
    } else {
        let (credited, _) = state
            .grain
            .as_mut()
            .ok_or("Chiz battle entry Grain is uninitialized")?
            .grant(cap)?;
        receipt(
            wide_fraction(cap)?,
            BigRational::zero(),
            wide_fraction(credited)?,
        )
    };
    state.battle_entries.insert(entry);
    Ok(result)
}
pub(crate) fn transition(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<GameActorCombatTransitionReceipt, String> {
    let leaving = ctx.request.combat_transition_events[event].leaving;
    let entry = event
        .checked_add(1)
        .ok_or("Chiz battle entry ID overflow")?;
    let actor = &mut ctx.state.actors[owner];
    let duplicate = !leaving && actor.character.chiz_mut()?.battle_entries.contains(&entry);
    if actor.hp == 0 || !duplicate && actor.character.chiz_mut()?.out_of_combat == leaving {
        return Err(
            "Chiz combat transition requires a living actor and alternating combat state".into(),
        );
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let mut deposits = std::collections::BTreeMap::new();
    if !leaving {
        let result = deposit(&ctx.request.actors[owner], actor, entry)?;
        if ctx.request.actors[owner].awakening.has_effect(5) {
            deposits.insert("chiz.grain".into(), result);
        }
    }
    if !duplicate {
        actor.character.chiz_mut()?.out_of_combat = leaving;
    }
    super::finance::refresh(ctx, owner)?;
    Ok(GameActorCombatTransitionReceipt {
        event_index: event,
        actor_index: owner,
        time_us: ctx.state.current_time_us,
        leaving,
        malice_granted: ExactFraction::new(0, 1)?,
        leave_grant_due_us: None,
        resource_deposits: deposits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_battle_entry_callback_never_regrants_initial_or_already_claimed_capacity() {
        let mut req = crate::dsl::compiler::compile_scenario(
            "options duration=1; chiz char lvl=80/80 awakening=6 talent=8,8,8 qte=8;
             chiz add weapon=fork_mamen refine=1;
             target hp=100000000 def=1080 resist=0; active chiz;",
            Default::default(),
        )
        .unwrap()
        .request;
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4, 5, 6]).unwrap();
        for model in [
            crate::HypothesisId::ChizExactFinanceV1,
            crate::HypothesisId::ChizHeldSkillV1,
        ] {
            req.model_assumptions
                .insert(model.into(), model.description().into());
        }
        let mut state = crate::simulation::state::GameSimulatorState::new(&req);
        let actor = &mut state.actors[0];
        crate::registry::initialize_actor(&req.actors[0], actor, &req).unwrap();
        assert_eq!(
            deposit(&req.actors[0], actor, 0).unwrap().input.numerator,
            "0"
        );
        actor
            .character
            .chiz_mut()
            .unwrap()
            .finance
            .as_mut()
            .unwrap()
            .spend(wide_fraction(ExactFraction::new(30000, 1).unwrap()).unwrap())
            .unwrap();
        let first = deposit(&req.actors[0], actor, 1).unwrap();
        assert_eq!(first.repaid.numerator, "15000");
        assert_eq!(first.credited.numerator, "0");
        let again = deposit(&req.actors[0], actor, 1).unwrap();
        assert_eq!(again.input.numerator, "0");
        assert!(
            actor
                .character
                .chiz_mut()
                .unwrap()
                .finance
                .as_ref()
                .unwrap()
                .wallet
                .current()
                .is_zero()
        );
        assert_eq!(
            deposit(&req.actors[0], actor, 2)
                .unwrap()
                .credited
                .numerator,
            "15000"
        );
    }
}
