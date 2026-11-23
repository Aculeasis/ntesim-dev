use super::*;

pub(crate) fn effective_def(
    actor_config: &crate::simulation::GameActorConfig,
    buff_stats: &impl crate::runtime_api::BuffStatsView,
) -> Result<ExactFraction, String> {
    let buff_stat = |key: &str| -> Result<ExactFraction, String> {
        buff_stats
            .get(key)
            .copied()
            .map(ExactFraction::from_ratio)
            .unwrap_or_else(|| ExactFraction::new(0, 1))
    };
    let live_def_percent = buff_stat("def_percent")?;
    let base_def = match actor_config.static_stats.base_def {
        Some(base_def) => ExactFraction::new(base_def.into(), 1)?,
        None => {
            if !live_def_percent.is_zero() || actor_config.static_stats.def_percent.numerator != 0 {
                return Err("DEF percent requires a native base-DEF projection".into());
            }
            ExactFraction::new(actor_config.static_stats.def.into(), 1)?
        }
    };
    let static_def_percent = ExactFraction::from_ratio(actor_config.static_stats.def_percent)?;
    let def_percent = static_def_percent.add(live_def_percent)?;
    base_def
        .mul(ExactFraction::new(1, 1)?.add(def_percent)?)?
        .add(ExactFraction::from_ratio(
            actor_config.static_stats.def_flat,
        )?)?
        .add(buff_stat("def_add")?)
}

fn scaling_stats(
    actor_config: &crate::simulation::GameActorConfig,
    scaling_stat: crate::simulation::NativeScalingStat,
    buff_stats: &impl crate::runtime_api::BuffStatsView,
) -> Result<(ExactFraction, ExactFraction), String> {
    let buff_stat = |key: &str| -> Result<ExactFraction, String> {
        buff_stats
            .get(key)
            .copied()
            .map(ExactFraction::from_ratio)
            .unwrap_or_else(|| ExactFraction::new(0, 1))
    };
    let mut atk_exact = ExactFraction::new(0, 1)?;
    let mut def_exact = ExactFraction::new(0, 1)?;
    scaling_stat.validate()?;
    if scaling_stat == crate::simulation::NativeScalingStat::BaseAtk {
        atk_exact = ExactFraction::new(actor_config.static_stats.base_atk as u128, 1)?;
    } else if scaling_stat.uses_atk() {
        atk_exact =
            ExactFraction::from_ratio(actor_config.static_stats.atk)?.add(buff_stat("atk_add")?)?;
        let atk_percent = buff_stat("atk_percent")?;
        if !atk_percent.is_zero() {
            atk_exact = atk_exact.add(
                ExactFraction::new(actor_config.static_stats.base_atk as u128, 1)?
                    .mul(atk_percent)?,
            )?;
        }
    }
    if scaling_stat.uses_def() {
        def_exact = effective_def(actor_config, buff_stats)?;
    }
    if scaling_stat.uses_hp() {
        for key in ["hp_percent", "hp_flat", "hp_add"] {
            if !buff_stat(key)?.is_zero() {
                return Err(format!(
                    "HP-scaled native packet cannot resolve live {key} modifiers"
                ));
            }
        }
    }
    Ok((atk_exact, def_exact))
}

pub(super) fn packet_stat(
    ctx: &crate::runtime_api::DamageContext<'_>,
    scaling_stat: crate::simulation::NativeScalingStat,
    modifiers: &crate::runtime_api::HitModifiers,
) -> Result<ExactFraction, String> {
    let config = ctx.config;
    let (mut atk, mut def) = scaling_stats(config, scaling_stat, &ctx.buff_stats)?;
    if scaling_stat != crate::simulation::NativeScalingStat::BaseAtk && scaling_stat.uses_atk() {
        atk = atk.add(
            ExactFraction::new(config.static_stats.base_atk.into(), 1)?
                .mul(modifiers.atk_percent)?,
        )?;
    }
    if scaling_stat.uses_def() && !modifiers.def_percent.is_zero() {
        let base = config
            .static_stats
            .base_def
            .ok_or("DEF percent requires a native base-DEF projection")?;
        def = def.add(ExactFraction::new(base.into(), 1)?.mul(modifiers.def_percent)?)?;
    }
    let [atk_weight, def_weight, hp_weight] = scaling_stat.weights();
    let mut base = atk
        .mul(ExactFraction::from_ratio(atk_weight)?)?
        .add(def.mul(ExactFraction::from_ratio(def_weight)?)?)?;
    if scaling_stat.uses_hp() {
        let hp = crate::simulation::state::effective_hp_max(
            ctx.buffs,
            ctx.actor.hp_max_base,
            &ctx.actor.hp_max_static_bonus,
            ctx.actor_index,
            ctx.actor.hp_max,
        )?;
        base = base
            .add(ExactFraction::new(hp.into(), 1)?.mul(ExactFraction::from_ratio(hp_weight)?)?)?;
    }
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{formula::Ratio, simulation::NativeScalingStat};

    #[test]
    fn mixed_scaling_reads_live_atk_and_def_zones_from_their_own_base_stats() {
        let mut req = crate::simulation::tests::sample_s1_request();
        let config = &mut req.actors[0];
        config.static_stats.base_atk = 100;
        config.static_stats.atk = Ratio {
            numerator: 200,
            denominator: 1,
        };
        config.static_stats.base_def = Some(80);
        config.static_stats.def_percent = Ratio {
            numerator: 1,
            denominator: 2,
        };
        config.static_stats.def_flat = Ratio {
            numerator: 3,
            denominator: 1,
        };
        let stats = std::collections::BTreeMap::from([
            (
                "atk_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 5,
                },
            ),
            (
                "atk_add".into(),
                Ratio {
                    numerator: 7,
                    denominator: 1,
                },
            ),
            (
                "def_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 4,
                },
            ),
            (
                "def_add".into(),
                Ratio {
                    numerator: 11,
                    denominator: 1,
                },
            ),
        ]);
        let mixed = NativeScalingStat::Mixed {
            atk: Ratio::one(),
            def: Ratio::one(),
            hp: Ratio::zero(),
        };
        let (atk, def) = scaling_stats(config, mixed, &stats).unwrap();
        assert_eq!(atk, ExactFraction::new(227, 1).unwrap()); // 200 + 100*.2 + 7
        assert_eq!(def, ExactFraction::new(154, 1).unwrap()); // 80*(1 + .5 + .25) + 3 + 11
        config.static_stats.base_def = None;
        assert!(
            scaling_stats(config, mixed, &stats)
                .unwrap_err()
                .contains("base-DEF")
        );
        assert!(scaling_stats(config, NativeScalingStat::Atk, &stats).is_ok());
    }
}
