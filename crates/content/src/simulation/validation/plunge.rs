//! Shared profile shape; the owning kit validates its action and timing.
use super::*;
pub(super) fn validate_inputs(
    request: &GameScenarioRequest,
    packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
) -> Result<(), String> {
    for (&index, input) in &request.plunge_inputs {
        let Some(GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            ..
        }) = request.actions.get(index)
        else {
            return Err(format!(
                "plunge_inputs index {index} must identify an InvokeAbility"
            ));
        };
        let metadata = packets[index]
            .as_ref()
            .ok_or("plunge packet metadata missing")?;
        if input.landing_delay_us == 0
            || input.height_bonus.denominator == 0
            || input.height_bonus.numerator > input.height_bonus.denominator
        {
            return Err(format!(
                "plunge_inputs index {index} requires positive landing time and height bonus0..1"
            ));
        }
        crate::registry::validate_plunge_binding(
            &request.actors[*actor_index].character_id,
            &crate::runtime_api::PlungeInputContext {
                request,
                metadata,
                input,
                timestamp_us: *timestamp_us,
            },
        )
        .map_err(|error| format!("plunge_inputs index {index}: {error}"))?;
    }
    Ok(())
}
