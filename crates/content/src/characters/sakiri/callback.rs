use crate::runtime_api::CallbackContext;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GameEventReceipt};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    a4: bool,
    source_actor_index: usize,
    atk_add: ExactFraction,
    duration_us: u64,
) -> Result<(), String> {
    let id = if a4 {
        "Sagiri_Level4_AtkAdd"
    } else {
        "Sagiri_QDamUp_AtkAdd"
    };
    let state = &mut *ctx.state;
    let trace = &mut *ctx.trace;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    for recipient_index in 0..ctx.request.actors.len() {
        if recipient_index != source_actor_index && state.actors[recipient_index].hp > 0 {
            state.apply_buff(
                source_actor_index,
                TimedBuffDeclaration::reset(
                    id,
                    "character:1003",
                    BuffRecipient::SpecificActor(recipient_index),
                    duration_us,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([("atk_add".into(), atk_add)]),
                ),
            )?;
        }
    }
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "sakiri_atk_buff".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(source_actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            buff_id: Some(id.into()),
            atk_add: Some(atk_add),
            duration_us: Some(duration_us),
            ..Default::default()
        }),
    });
    Ok(())
}
