use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CallbackContext, CastContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameActorConfig, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffRecipient, BuffSourceLifetime, BuffStackingScope, DecayPolicy,
    RefreshPolicy,
};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsTwinbirdsV2;

pub(crate) fn initialize(ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    for (actor_index, config) in ctx.request.actors.iter().enumerate() {
        for proc in &config.gear_procs {
            let GearProcConfig::Twinbirds {
                cycle_intensity_bonus,
                ..
            } = proc
            else {
                continue;
            };
            crate::runtime_api::record_hypothesis_id(
                ctx.request,
                &mut ctx.state.execution_model_assumptions,
                HYPOTHESIS_ID,
            )?;
            ctx.state
                .buffs
                .apply_buff(
                    BuffDefinition {
                        id: "twinbirds.cycle_intensity".into(),
                        source: "weapon:fork_twinbirds".into(),
                        recipient: BuffRecipient::SelfActor,
                        stacking_scope: BuffStackingScope::PerSource,
                        source_lifetime: BuffSourceLifetime::WhileAlive,
                        semantic_tags: Default::default(),
                        duration_us: None,
                        refresh_policy: RefreshPolicy::Disallow,
                        max_stacks: 1,
                        decay_policy: DecayPolicy::AllAtOnce,
                        stat_modifiers: std::collections::BTreeMap::from([(
                            "cycle_intensity".into(),
                            *cycle_intensity_bonus,
                        )]),
                    },
                    actor_index,
                    ctx.state.mode_clock.relative_us(),
                )
                .map_err(|error| {
                    format!("Twinflight Cycle Intensity initialization failed: {error:?}")
                })?;
        }
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::Twinbirds {
            team_crit_damage_bonus,
            self_crit_damage_bonus,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::Ultimate) {
                let team_crit = ExactFraction::from_ratio(*team_crit_damage_bonus)?;
                let self_crit = ExactFraction::from_ratio(*self_crit_damage_bonus)?;
                let self_decl = TimedBuffDeclaration::reset(
                    "twinbirds.self_crit_damage",
                    "weapon:fork_twinbirds",
                    BuffRecipient::SelfActor,
                    *duration_us,
                    BuffStackingScope::PerSource,
                    std::collections::BTreeMap::from([("crit_damage_bonus".into(), self_crit)]),
                );
                let grant = crate::modifiers::BuffGrantBuilder::new(
                    ctx.actor_index,
                    "twinbirds.team_crit_damage",
                    "weapon:fork_twinbirds",
                )
                .recipient(BuffRecipient::TeamAll)
                .stacking_scope(BuffStackingScope::Shared)
                .duration_us(*duration_us)
                .stat("crit_damage_bonus", team_crit)
                .add_declaration(self_decl)
                .receipt(
                    "twinbirds_crit_damage",
                    GameEventPayload {
                        crit_damage_bonus: Some(team_crit),
                        exact_amount: Some(self_crit),
                        duration_us: Some(*duration_us),
                        ..Default::default()
                    },
                )
                .into_event();
                crate::gear::schedule_cast_buff(ctx, grant)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    config: &GameActorConfig,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if packet.element == Some(DamageElement::Psyche) {
        for proc in &config.gear_procs {
            if let GearProcConfig::Twinbirds { psyche_bonus, .. } = proc {
                modifiers = modifiers.add_damage(ExactFraction::from_ratio(*psyche_bonus)?)?;
            }
        }
    }
    Ok(modifiers)
}
