use crate::simulation::ExactFraction;

pub(super) fn warrant_bonus(
    config: &crate::simulation::GameActorConfig,
) -> Result<ExactFraction, String> {
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    Ok(
        if progression["breakthrough"]
            .as_u64()
            .ok_or("Chaos breakthrough missing")?
            >= 4
        {
            PARAMETERS.b4_warrant_damage_bonus
        } else {
            PARAMETERS.warrant_damage_bonus
        },
    )
}

pub(super) struct Parameters {
    pub(super) warrant_damage_bonus: ExactFraction,
    pub(super) b4_warrant_damage_bonus: ExactFraction,
    pub(super) a5_single_target_damage_bonus: ExactFraction,
    pub(super) a2_defense_ignore: ExactFraction,
    pub(super) a3_crit_damage: ExactFraction,
    pub(super) a3_duration_us: u64,
    pub(super) warrant_duration_us: u64,
    pub(super) resonance_resistance_reduction: ExactFraction,
}

// Global 2026-10-04 DT_ChaosEffectFigure; only consumed owner parameters.
// HuntMarkDamageUp/Duration, Star2DefIgnore, Star3CritDamageUp/Duration,
// Star5DamageUp and StarExtra2ResDown are constant rows. Existing Warrant
// applicability remains the runtime model; seconds become exact microseconds.
pub(super) const PARAMETERS: Parameters = Parameters {
    warrant_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    b4_warrant_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a5_single_target_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a2_defense_ignore: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a3_crit_damage: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    a3_duration_us: 20000000,
    warrant_duration_us: 15000000,
    resonance_resistance_reduction: ExactFraction {
        numerator: 1,
        denominator: 10,
    },
};
