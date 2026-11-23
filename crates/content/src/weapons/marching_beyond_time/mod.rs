use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameScenarioRequest, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsMarchingBeyondTimeV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) wastelab_active: bool,
    pub(crate) stacks: u32,
    pub(crate) ultimate_crit_damage: std::collections::BTreeMap<usize, ExactFraction>,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .config
        .gear_procs
        .iter()
        .any(|p| matches!(p, GearProcConfig::MarchingBeyondTime { .. }))
    {
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
    }
    Ok(())
}

pub(crate) fn observe_team_admission(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    actor_index: usize,
    command_index: usize,
    role: Option<InputActionRole>,
) -> Result<(), String> {
    for (wearer_index, config) in request.actors.iter().enumerate() {
        let Some(GearProcConfig::MarchingBeyondTime {
            state_crit_damage_bonus,
            crit_damage_per_stack,
            def_ignore,
            def_ignore_duration_us,
            ..
        }) = config
            .gear_procs
            .iter()
            .find(|p| matches!(p, GearProcConfig::MarchingBeyondTime { .. }))
        else {
            continue;
        };
        let runtime = &mut state.actors[wearer_index].equipment.marching_beyond_time;
        if actor_index == wearer_index {
            match role {
                Some(InputActionRole::RedirectSkill) => {
                    runtime.wastelab_active = true;
                    runtime.stacks = 0;
                }
                Some(InputActionRole::Ultimate) if runtime.wastelab_active => {
                    let stacks = runtime.stacks;
                    runtime.wastelab_active = false;
                    runtime.stacks = 0;
                    let bonus = ExactFraction::from_ratio(*state_crit_damage_bonus)?.add(
                        ExactFraction::from_ratio(*crit_damage_per_stack)?
                            .mul(ExactFraction::new(stacks.into(), 1)?)?,
                    )?;
                    runtime.ultimate_crit_damage.insert(command_index, bonus);
                    if stacks == 3 {
                        state.apply_buff(
                            wearer_index,
                            TimedBuffDeclaration::reset(
                                "marching_beyond_time.def_ignore",
                                "weapon:fork_Time",
                                BuffRecipient::SelfActor,
                                *def_ignore_duration_us,
                                BuffStackingScope::PerSource,
                                std::collections::BTreeMap::from([(
                                    "def_ignore".into(),
                                    ExactFraction::from_ratio(*def_ignore)?,
                                )]),
                            ),
                        )?;
                    }
                }
                _ => {}
            }
        } else if runtime.wastelab_active
            && matches!(
                role,
                Some(InputActionRole::RedirectSkill | InputActionRole::Qte)
            )
        {
            runtime.stacks = (runtime.stacks + 1).min(3);
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::MarchingBeyondTime { .. } = proc
            && ctx.action_metadata.input_role == Some(InputActionRole::Ultimate)
            && let Some(bonus) = ctx
                .actor
                .equipment
                .marching_beyond_time
                .ultimate_crit_damage
                .get(&ctx.command_index)
        {
            modifiers = modifiers.add_crit_damage(*bonus)?;
        }
    }
    Ok(modifiers)
}
