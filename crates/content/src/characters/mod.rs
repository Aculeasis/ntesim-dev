pub(crate) mod action_packet_bindings;
pub(crate) mod action_roles;
pub(crate) mod adler;
pub(crate) mod aurelia;
pub(crate) mod baicang;
pub(crate) mod blackbird;
pub(crate) mod chaos;
pub(crate) mod chiz;
pub(crate) mod cinematics;
pub(crate) mod daffodil;
pub(crate) mod edgar;
pub(crate) mod fadia;
pub(crate) mod haniel;
pub(crate) mod hathor;
pub mod hotori;
pub(crate) mod iroi;
pub(crate) mod jiuyuan;
pub(crate) mod lacrimosa;
pub(crate) mod linko;
pub(crate) mod mint;
pub(crate) mod nanally;
pub(crate) mod normal;
pub(crate) mod packet_semantics;
pub(crate) mod parry_inputs;
pub(crate) mod player_inputs;
mod queries;
pub(crate) mod replay_clock;
pub(crate) mod resonance;
pub(crate) mod sakiri;
pub(crate) mod shinku;
pub(crate) mod skia;
pub(crate) mod source_choices;
mod state;
pub(crate) mod zankou;
pub(crate) mod zero;
pub(crate) use state::RuntimeState;
pub(crate) mod team;

pub(crate) fn on_field_exit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    ctx.state.actors[actor_index].aiming = false;
    ctx.state.actors[actor_index].character.reset_normal_chain();
    let char_id = &ctx.request.actors[actor_index].character_id;
    if char_id == "character:1010" {
        nanally::skill::on_field_exit(ctx, actor_index)?;
    } else if char_id == "character:1039" {
        let state = ctx.state.actors[actor_index].character.fadia_mut()?;
        state.lilith_until_us = None;
    } else if char_id == "character:1025" {
        let state = ctx.state.actors[actor_index].character.hathor_mut()?;
        state.emergency_delivery_until_us = 0;
    } else if char_id == "character:1019" {
        mint::skill::on_field_exit(ctx, actor_index)?;
    } else if char_id == "character:1075" {
        let now = ctx.state.current_time_us;
        let is_living = ctx.state.actors[actor_index].hp > 0;
        iroi::resource::update_off_field_moon(
            &ctx.request.actors[actor_index],
            ctx.state.actors[actor_index].character.iroi_mut()?,
            now,
            is_living,
            false,
        )?;
    }
    Ok(())
}

pub(crate) fn on_field_entry(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
) -> Result<(), String> {
    adler::skill::transfer_party_shield(ctx, actor_index)?;
    let char_id = &ctx.request.actors[actor_index].character_id;
    if char_id == "character:1019" {
        mint::skill::on_field_entry(ctx, actor_index)?;
    } else if char_id == "character:1075" {
        let now = ctx.state.current_time_us;
        let is_living = ctx.state.actors[actor_index].hp > 0;
        iroi::resource::update_off_field_moon(
            &ctx.request.actors[actor_index],
            ctx.state.actors[actor_index].character.iroi_mut()?,
            now,
            is_living,
            true,
        )?;
    }
    Ok(())
}
