use super::config::GearProcConfig;

#[derive(Clone, Debug, Default)]
pub(crate) struct RuntimeState {
    pub(crate) good_boy: crate::weapons::good_boy::State,
    pub(crate) noros: crate::weapons::noros::State,
    pub(crate) mind_royale: crate::weapons::mind_royale::State,
    pub(crate) motor_candy: crate::weapons::motor_candy::State,
    pub(crate) demon_blade: crate::weapons::demon_blade::State,
    pub(crate) far_side_moon: crate::weapons::far_side_moon::State,
    pub(crate) moon: crate::weapons::moon::State,
    pub(crate) bopu: crate::weapons::bopu::State,
    pub(crate) spider: crate::weapons::spider::State,
    pub(crate) bit_game: crate::weapons::bit_game::State,
    pub(crate) marching_beyond_time: crate::weapons::marching_beyond_time::State,
    pub(crate) be_happy: crate::weapons::be_happy::State,
    pub(crate) nakupeda: Option<crate::weapons::nakupeda::State>,
    pub(crate) jingmotingyuan: Option<crate::weapons::jingmotingyuan::arc::State>,
    pub(crate) bittercake: crate::weapons::bittercake::State,
    pub(crate) whale: crate::weapons::whale::State,
    pub(crate) tiger_tally: crate::weapons::tiger_tally::State,
    pub(crate) rishi: crate::weapons::rishi::State,
    pub(crate) police_rat: crate::weapons::police_rat::State,
    pub(crate) black_book: crate::weapons::black_book::State,
}

pub(crate) fn initialize_runtime(
    config: &crate::simulation::GameActorConfig,
    actor_index: usize,
    seed: u64,
) -> Result<RuntimeState, String> {
    let mut runtime = RuntimeState::default();
    if config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::Nakupeda { .. }))
    {
        runtime.nakupeda = Some(crate::weapons::nakupeda::State::new(seed, actor_index)?);
    }
    if config
        .gear_procs
        .iter()
        .any(|proc| matches!(proc, GearProcConfig::JingmotingyuanCrit { .. }))
    {
        runtime.jingmotingyuan = Some(crate::weapons::jingmotingyuan::arc::State::new(
            seed,
            actor_index,
        )?);
    }
    Ok(runtime)
}

pub(crate) fn validate_rng_selectors(
    request: &crate::simulation::GameScenarioRequest,
) -> Result<(), String> {
    let Some(policy) = &request.stochastic_crit else {
        return Ok(());
    };
    for (index, actor) in request.actors.iter().enumerate() {
        if actor
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::JingmotingyuanCrit { .. }))
            && policy
                .actor_sequences
                .values()
                .any(|sequence| *sequence == crate::weapons::jingmotingyuan::arc::selector(index))
        {
            return Err("Silent Garden proc RNG selector collides with stochastic_crit".into());
        }
        if actor
            .gear_procs
            .iter()
            .any(|proc| matches!(proc, GearProcConfig::Nakupeda { .. }))
        {
            let selector = crate::weapons::nakupeda::rng_selector(index)?;
            if policy
                .actor_sequences
                .values()
                .any(|sequence| *sequence == selector)
            {
                return Err(format!(
                    "Nacupeda proc RNG selector {selector} collides with stochastic_crit"
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn proc_rng_receipts(
    _request: &crate::simulation::GameScenarioRequest,
    actors: &[crate::simulation::state::ActorRuntime],
) -> Result<Vec<serde_json::Value>, String> {
    actors
        .iter()
        .enumerate()
        .flat_map(|(actor_index, actor)| {
            [
                actor.equipment.nakupeda.as_ref().map(|state| &state.rng),
                actor
                    .equipment
                    .jingmotingyuan
                    .as_ref()
                    .map(|state| &state.rng),
            ]
            .into_iter()
            .flatten()
            .map(move |rng| (actor_index, rng))
        })
        .map(|(actor_index, rng)| {
            Ok(serde_json::json!({
                "actor_index": actor_index,
                "rng": crate::weapons::nakupeda::rng_snapshot(rng)?
            }))
        })
        .collect()
}

pub(crate) fn proc_selection_receipts(
    actors: &[crate::simulation::state::ActorRuntime],
) -> Vec<crate::simulation::GameProcSelectionReceipt> {
    let mut receipts = actors
        .iter()
        .flat_map(|actor| {
            [
                actor
                    .equipment
                    .nakupeda
                    .as_ref()
                    .map(|state| state.receipts.as_slice()),
                actor
                    .equipment
                    .jingmotingyuan
                    .as_ref()
                    .map(|state| state.receipts.as_slice()),
            ]
            .into_iter()
            .flatten()
            .flat_map(|receipts| receipts.iter().cloned())
        })
        .collect::<Vec<_>>();
    receipts.sort_by_key(|receipt| (receipt.time_us, receipt.actor_index));
    receipts
}

pub(crate) fn initialize(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut assumptions = crate::weapons::motor_candy::initialize(ctx)?;
    assumptions.extend(crate::weapons::bit_game::initialize(ctx)?);
    crate::weapons::whale::initialize(ctx)?;
    crate::weapons::twinbirds::initialize(ctx)?;
    for actor in &ctx.request.actors {
        for proc in &actor.gear_procs {
            let hypothesis_id = match proc {
                GearProcConfig::DevilsBloodDamage { .. } => {
                    Some(crate::sets::devils_blood::HYPOTHESIS_ID)
                }
                GearProcConfig::DiabolosChaosIgnore { .. } => {
                    Some(crate::sets::diabolos::HYPOTHESIS_ID)
                }
                GearProcConfig::StreetBoxerCrit { .. } => {
                    Some(crate::sets::street_boxer::HYPOTHESIS_ID)
                }
                GearProcConfig::JiaojuanBreak { .. } => {
                    Some(crate::weapons::jiaojuan::HYPOTHESIS_ID)
                }
                GearProcConfig::SnowmanShieldAtk { .. } => {
                    Some(crate::weapons::snowman::HYPOTHESIS_ID)
                }
                GearProcConfig::YurenHp { .. } => Some(crate::weapons::yuren::HYPOTHESIS_ID),
                GearProcConfig::TuansanlangTeam { .. } => {
                    Some(crate::weapons::tuansanlang::HYPOTHESIS_ID)
                }
                GearProcConfig::YaodaoParry { .. } => Some(crate::weapons::yaodao::HYPOTHESIS_ID),
                GearProcConfig::ThiefCandyDodge { .. } => {
                    Some(crate::weapons::thief_candy::HYPOTHESIS_ID)
                }
                GearProcConfig::DustbinBreak { .. } => Some(crate::weapons::dustbin::HYPOTHESIS_ID),
                GearProcConfig::PoliceRatBoss { .. } => {
                    Some(crate::weapons::police_rat::HYPOTHESIS_ID)
                }
                GearProcConfig::RoseThorn { .. } => Some(crate::weapons::rose::HYPOTHESIS_ID),
                GearProcConfig::TigerTallyAtk { .. } => {
                    Some(crate::weapons::tiger_tally::HYPOTHESIS_ID)
                }
                GearProcConfig::Nakupeda { .. } => Some(crate::weapons::nakupeda::HYPOTHESIS_ID),
                GearProcConfig::BlackBookBreak { .. } => {
                    Some(crate::weapons::black_book::HYPOTHESIS_ID)
                }
                GearProcConfig::JingmotingyuanCrit { .. } => {
                    Some(crate::weapons::jingmotingyuan::HYPOTHESIS_ID)
                }
                GearProcConfig::RishiAtk { .. } => Some(crate::weapons::rishi::HYPOTHESIS_ID),
                GearProcConfig::MamenFons { .. } => Some(crate::weapons::mamen::HYPOTHESIS_ID),
                _ => None,
            };
            if let Some(id) = hypothesis_id {
                crate::runtime_api::record_hypothesis_id(
                    ctx.request,
                    &mut ctx.state.execution_model_assumptions,
                    id,
                )?;
            }
        }
    }
    Ok(assumptions)
}
