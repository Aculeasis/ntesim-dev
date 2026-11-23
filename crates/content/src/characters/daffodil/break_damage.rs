use super::parameters::PARAMETERS;
use crate::runtime_api::{BreakContext, BreakModifiers};
use crate::simulation::ExactFraction as F;

pub(crate) fn modifiers(ctx: &BreakContext<'_>) -> Result<BreakModifiers, String> {
    let config = ctx.config;
    let stacks = super::insight::stacks(ctx)?;
    Ok(BreakModifiers {
        damage_bonus: PARAMETERS.break_damage_bonus.add(
            PARAMETERS.a4_break_damage_per_insight.mul(F::new(
                if config.awakening.has_effect(4) {
                    u128::from(stacks)
                } else {
                    0
                },
                1,
            )?)?,
        )?,
        defense_ignore: if config.awakening.has_effect(6) {
            PARAMETERS.defense_ignore
        } else {
            F::new(0, 1)?
        },
        extra_contributions: if config.awakening.has_effect(5) {
            stacks
        } else {
            0
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a6_remains_personal_while_r6_is_supplied_by_the_target_callback() {
        let mut request = crate::simulation::tests::research_request();
        request.actors[0].character_id = "character:1054".into();
        request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
        let state = crate::simulation::state::GameSimulatorState::new(&request);
        let modifiers = modifiers(&BreakContext {
            world_time_us: 0,
            config: &request.actors[0],
            actor_index: 0,
            target_index: 0,
            buffs: &state.buffs,
        })
        .unwrap();
        assert_eq!(modifiers.defense_ignore, F::new(3, 20).unwrap());
        assert_eq!(modifiers.damage_bonus, F::new(1, 5).unwrap());
        assert_eq!(modifiers.extra_contributions, 0);
    }
}
