use crate::runtime_api::{CallbackContext, CastContext};

/// An admitted parent, retained across switches without retaining source payloads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Source(crate::runtime_api::OwnedAction);

impl Source {
    pub(super) fn command(&self) -> usize {
        self.0.command()
    }
    pub(crate) fn capture(ctx: &CastContext<'_>) -> Result<Self, String> {
        Ok(Self(crate::runtime_api::OwnedAction::capture(ctx)?))
    }

    pub(crate) fn emit(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
        effect: &str,
        target: Option<usize>,
        doubled: bool,
    ) -> Result<(), String> {
        self.with_cast(ctx, owner, target.unwrap_or(0), |cast| {
            cast.record_hypothesis(super::hootie::HYPOTHESIS)?;
            match target {
                Some(target) => {
                    cast.schedule_scaled_native_child(effect, target, cast.time_us, doubled)
                }
                None => cast.schedule_native_aoe_packet(effect, cast.time_us),
            }
        })
    }

    pub(super) fn activate_ace(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        self.with_cast(ctx, owner, 0, |cast| {
            super::hootie::capture_burst(cast)?;
            super::skill::activate_ace(cast)?;
            super::hootie::schedule_expiry(cast)
        })
    }

    pub(super) fn cannon(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
        primary: Option<usize>,
    ) -> Result<(), String> {
        self.with_cast(ctx, owner, primary.unwrap_or(0), |cast| {
            cast.record_hypothesis(super::cannon::MODEL)?;
            let group = cast.schedule_native_aoe_group(super::cannon::PACKET, cast.time_us)?;
            if let Some(primary) = primary {
                cast.queue.retain(|event| !matches!(&event.data,
                    crate::simulation::events::GameEventData::HitDelivery { target_index, metadata, .. }
                        if metadata.binding.energy_group_id.as_deref() == Some(group.as_str()) && *target_index != primary));
            }
            Ok(())
        })
    }

    fn with_cast<T>(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
        target: usize,
        apply: impl FnOnce(&mut CastContext<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        self.0.with_cast(ctx, owner, target, apply)
    }
}
