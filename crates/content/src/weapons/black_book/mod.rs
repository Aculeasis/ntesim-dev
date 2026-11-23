use crate::HypothesisId;
mod config;
pub(crate) use config::validate_config;
pub(crate) mod arc;
pub(crate) mod projection;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsBlackBookV1;

pub(crate) use arc::State;

pub(crate) fn damage_bonus(
    ctx: &crate::runtime_api::DamageContext<'_>,
    packet: &crate::simulation::metadata::PacketMetadata,
    mut modifiers: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    let state = &ctx.actor.equipment.black_book;
    if ctx.time_us < state.arc_active_until_us
        && state.designated_target == Some(ctx.target_index)
        && packet.element == Some(crate::simulation::metadata::DamageElement::Chaos)
    {
        modifiers = modifiers.add_damage(crate::simulation::ExactFraction::from_ratio(
            state.chaos_damage_bonus,
        )?)?;
    }
    Ok(modifiers)
}

pub(crate) fn observe_team_admission(
    request: &crate::simulation::GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    _command_index: usize,
    action: &crate::simulation::metadata::ActionMetadata,
) -> Result<(), String> {
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::Qte)
        && action.native_ability_index == Some(4)
        && state.actors[actor_index].hp > 0
    {
        for (i, config) in request.actors.iter().enumerate() {
            if config
                .weapon_id
                .eq_ignore_ascii_case("weapon:fork_BlackBook")
            {
                let runtime = &mut state.actors[i].equipment.black_book;
                runtime.chains_remaining = runtime.chains_remaining.saturating_sub(1);
            }
        }
    }
    Ok(())
}

pub(crate) fn on_target_break_damage(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target_index: usize,
) -> Result<(), String> {
    arc::on_target_break_damage(ctx, target_index)
}
