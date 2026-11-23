//! A0 action carriers and real retained overlap; A1 makes both living-source permanent.
use super::{parameters::PARAMETERS, state::State};
use crate::simulation::state::ActorRuntime;
use ntesim_core::buffs::dispatch::*;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ZankouFormModifiersV1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Hunt,
    Delusion,
}
const HUNT: &str = "zankou.hunt";
const DELUSION: &str = "zankou_delusion_dot_crit";
fn grant(
    buffs: &mut BuffManager,
    owner: usize,
    kind: Kind,
    end: Option<u64>,
    now: u64,
    world_time_us: u64,
    a1: bool,
) -> Result<(), String> {
    let id = if kind == Kind::Hunt { HUNT } else { DELUSION };
    buffs.remove_owned_buff(id, "character:1036", owner);
    if end.is_some_and(|end| now >= end) {
        return Ok(());
    }
    let (name, value) = if kind == Kind::Hunt {
        let value = if a1 {
            PARAMETERS.a1_hunt_damage_bonus
        } else {
            PARAMETERS.hunt_damage_bonus
        };
        (
            "damage_bonus",
            crate::formula::Ratio {
                numerator: value
                    .numerator
                    .try_into()
                    .map_err(|_| "Hunt magnitude overflow")?,
                denominator: value
                    .denominator
                    .try_into()
                    .map_err(|_| "Hunt magnitude overflow")?,
            },
        )
    } else {
        (
            "crit_damage_bonus_dot",
            if a1 {
                PARAMETERS.a1_team_dot_crit_damage
            } else {
                crate::formula::Ratio {
                    numerator: 1,
                    denominator: 4,
                }
            },
        )
    };
    buffs
        .apply_buff(
            BuffDefinition {
                id: id.into(),
                source: "character:1036".into(),
                recipient: if kind == Kind::Hunt {
                    BuffRecipient::SelfActor
                } else {
                    BuffRecipient::TeamAll
                },
                source_lifetime: BuffSourceLifetime::WhileAlive,
                stacking_scope: BuffStackingScope::PerSource,
                duration_us: end.map(|end| end - now),
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                semantic_tags: Default::default(),
                stat_modifiers: [(name.into(), value)].into(),
            },
            owner,
            world_time_us,
        )
        .map_err(|e| format!("Zankou modifier application: {e:?}"))
}
pub(crate) fn initialize(
    config: &crate::simulation::GameActorConfig,
    owner: usize,
    buffs: &mut BuffManager,
) -> Result<(), String> {
    if config.character_id == "character:1036" && config.awakening.has_effect(1) {
        grant(buffs, owner, Kind::Hunt, None, 0, 0, true)?;
        grant(buffs, owner, Kind::Delusion, None, 0, 0, true)?;
    }
    Ok(())
}
fn sync(
    state: &State,
    owner: usize,
    buffs: &mut BuffManager,
    now: u64,
    world_time_us: u64,
) -> Result<(), String> {
    if state.a1 {
        return Ok(());
    }
    grant(
        buffs,
        owner,
        Kind::Hunt,
        Some(state.hunt_until_us),
        now,
        world_time_us,
        false,
    )?;
    grant(
        buffs,
        owner,
        Kind::Delusion,
        Some(state.delusion_until_us),
        now,
        world_time_us,
        false,
    )
}
pub(crate) fn action(
    actor: &mut ActorRuntime,
    owner: usize,
    buffs: &mut BuffManager,
    kind: Option<Kind>,
    end: u64,
    now: u64,
) -> Result<(), String> {
    let world_time_us = actor.world_time_us;
    let state = actor.character.zankou_mut()?;
    state.hunt_until_us = state.hunt_retained_until_us;
    state.delusion_until_us = state.delusion_retained_until_us;
    match kind {
        Some(Kind::Hunt) => state.hunt_until_us = state.hunt_until_us.max(end),
        Some(Kind::Delusion) => state.delusion_until_us = state.delusion_until_us.max(end),
        None => {}
    }
    sync(state, owner, buffs, now, world_time_us)
}
pub(crate) fn transition(
    actor: &mut ActorRuntime,
    owner: usize,
    buffs: &mut BuffManager,
    entering: bool,
    now: u64,
) -> Result<(), String> {
    let world_time_us = actor.world_time_us;
    let state = actor.character.zankou_mut()?;
    if state.a1 {
        return Ok(());
    }
    let until = if entering {
        &mut state.hunt_until_us
    } else {
        &mut state.delusion_until_us
    };
    if *until > now {
        let end = now
            .checked_add(PARAMETERS.metadata_duration_us)
            .ok_or("Zankou retained modifier overflow")?;
        *until = (*until).max(end);
        if entering {
            state.hunt_retained_until_us = end;
        } else {
            state.delusion_retained_until_us = end;
        }
    }
    sync(state, owner, buffs, now, world_time_us)
}
pub(crate) fn clear(
    actor: &mut ActorRuntime,
    owner: usize,
    buffs: &mut BuffManager,
    now: u64,
) -> Result<(), String> {
    let world_time_us = actor.world_time_us;
    let state = actor.character.zankou_mut()?;
    state.hunt_until_us = 0;
    state.delusion_until_us = 0;
    state.hunt_retained_until_us = 0;
    state.delusion_retained_until_us = 0;
    sync(state, owner, buffs, now, world_time_us)
}
pub(crate) fn vitals(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        for id in [HUNT, DELUSION] {
            ctx.state
                .buffs
                .remove_owned_buff(id, "character:1036", owner);
        }
    } else if ctx.request.actors[owner].awakening.has_effect(1)
        && !ctx
            .state
            .buffs
            .instances()
            .iter()
            .any(|b| b.id == HUNT && b.owner_actor == owner)
    {
        initialize(&ctx.request.actors[owner], owner, &mut ctx.state.buffs)?;
    }
    Ok(())
}
