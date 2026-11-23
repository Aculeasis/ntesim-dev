use crate::simulation::ExactFraction;

pub(super) struct Parameters {
    pub(super) a1_evolved_skill_ratio_modify: ExactFraction,
    pub(super) a2_skill_ratio_modify: ExactFraction,
    pub(super) resonance_burst_ratio_modify: ExactFraction,
    pub(super) resonance_evolved_burst_ratio_modify: ExactFraction,
    pub(super) resonance_team_crit_damage: ExactFraction,
    pub(super) team_atk_percent: ExactFraction,
    pub(super) a4_team_damage_bonus: ExactFraction,
    pub(super) a6_psyche_damage_bonus: ExactFraction,
    pub(super) a6_psyche_duration_us: u64,
    pub(super) a2_form_def_percent: crate::formula::Ratio,
    pub(super) a5_resistance_reduction: ExactFraction,
    pub(super) a5_resistance_duration_us: u64,
    pub(super) team_duration_us: u64,
    pub(super) a4_team_duration_us: u64,
}

// Global 2026-10-04 DT_BlackBirdEffectFigure; only consumed owner parameters.
// Constant ratios/durations preserve the bounded A1/A2/R3/R6 models and recipients.
// Relative coefficient upgrades remain separate from additive damage bonuses; no Malice curve
// is retained without an executable consumer. Seconds are exact microseconds.
pub(super) const PARAMETERS: Parameters = Parameters {
    a6_psyche_duration_us: 25_000_000,
    a2_form_def_percent: crate::formula::Ratio {
        numerator: 1,
        denominator: 5,
    },
    a1_evolved_skill_ratio_modify: ExactFraction {
        numerator: 3,
        denominator: 2,
    },
    a2_skill_ratio_modify: ExactFraction {
        numerator: 1,
        denominator: 1,
    },
    resonance_burst_ratio_modify: ExactFraction {
        numerator: 1,
        denominator: 1,
    },
    resonance_evolved_burst_ratio_modify: ExactFraction {
        numerator: 1,
        denominator: 2,
    },
    resonance_team_crit_damage: ExactFraction {
        numerator: 3,
        denominator: 10,
    },
    team_atk_percent: ExactFraction {
        numerator: 1,
        denominator: 5,
    },
    a4_team_damage_bonus: ExactFraction {
        numerator: 3,
        denominator: 25,
    },
    a6_psyche_damage_bonus: ExactFraction {
        numerator: 1,
        denominator: 4,
    },
    a5_resistance_reduction: ExactFraction {
        numerator: 2,
        denominator: 25,
    },
    a5_resistance_duration_us: 25000000,
    team_duration_us: 15000000,
    a4_team_duration_us: 25000000,
};

pub(super) const K_MALICE_MILLIONTHS: [u128; 13] = [
    280_000, 302_400, 326_592, 352_719, 380_937, 411_412, 444_325, 479_871, 518_260, 559_721,
    604_499, 652_859, 705_088,
];

pub(super) const K3_MALICE_MILLIONTHS: [u128; 13] = [
    140_000, 151_200, 163_296, 176_360, 190_468, 205_706, 222_162, 239_935, 259_130, 279_861,
    302_249, 326_429, 352_544,
];

pub(super) fn malice_threshold(packet_id: &str) -> Option<ExactFraction> {
    let millionths = match packet_id {
        "GE_Player_BlackBird_Evolve_Melee1_Damage" => 541_591,
        "GE_Player_BlackBird_Evolve_Melee2_Damage" => 369_949,
        "GE_Player_BlackBird_Evolve_Melee3_Damage" => 349_951,
        "GE_Player_BlackBird_Evolve_Melee4_Damage" => 2_258_020,
        "GE_Player_BlackBird_Evolve_Melee5_Damage" => 784_891,
        "GE_Player_BlackBird_Evolve_Branch1_Damage" => 719_900,
        "GE_Player_BlackBird_Evolve_Branch2_Damage" => 673_240,
        "GE_Player_BlackBird_Evolve_AirAttack_Damage" => 933_204,
        "GE_Player_BlackBird_Evolve_PerfectEvade_Damage" => 2_099_708,
        "GE_Player_BlackBird_Evolve_QTE_Damage" => 1_666_435,
        "GE_Player_BlackBird_Evolve_Skill_Damage" => 3_250_000,
        _ => return None,
    };
    ExactFraction::new(millionths, 1_000_000).ok()
}

pub(super) fn extra_packet(primary_packet_id: &str) -> Option<&'static str> {
    match primary_packet_id {
        "GE_Player_BlackBird_Evolve_Melee1_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Melee1_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Melee2_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Melee2_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Melee3_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Melee3_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Melee4_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Melee4_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Melee5_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Melee5_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Branch1_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Branch1_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Branch2_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Branch2_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_AirAttack_Damage" => {
            Some("GE_Player_BlackBird_Evolve_AirAttack_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_PerfectEvade_Damage" => {
            Some("GE_Player_BlackBird_Evolve_PerfectEvade_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_QTE_Damage" => {
            Some("GE_Player_BlackBird_Evolve_QTE_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_Skill_Damage" => {
            Some("GE_Player_BlackBird_Evolve_Skill_Extra_Damage")
        }
        "GE_Player_BlackBird_Evolve_UltraSkill_Damage" => {
            Some("GE_Player_BlackBird_Evolve_UltraSkill_Extra_Damage")
        }
        _ => None,
    }
}

pub(super) fn extra_delay_us(primary_packet_id: &str) -> u64 {
    if primary_packet_id == "GE_Player_BlackBird_Evolve_Melee5_Damage" {
        100_000
    } else {
        67_000
    }
}

pub(super) fn extra_ability(primary_packet_id: &str) -> Option<&'static str> {
    match primary_packet_id {
        "GE_Player_BlackBird_Evolve_Melee1_Damage"
        | "GE_Player_BlackBird_Evolve_Melee2_Damage"
        | "GE_Player_BlackBird_Evolve_Melee3_Damage"
        | "GE_Player_BlackBird_Evolve_Melee4_Damage"
        | "GE_Player_BlackBird_Evolve_Melee5_Damage"
        | "GE_Player_BlackBird_Evolve_Branch1_Damage"
        | "GE_Player_BlackBird_Evolve_Branch2_Damage"
        | "GE_Player_BlackBird_Evolve_AirAttack_Damage"
        | "GE_Player_BlackBird_Evolve_PerfectEvade_Damage" => Some("GA_BlackBird_Evolve_Melee"),
        "GE_Player_BlackBird_Evolve_QTE_Damage" => Some("GA_BlackBird_Evolve_QTE"),
        "GE_Player_BlackBird_Evolve_Skill_Damage" => Some("GA_BlackBird_Evolve_Skill"),
        "GE_Player_BlackBird_Evolve_UltraSkill_Damage" => Some("GA_BlackBird_Evolve_UltraSkill"),
        _ => None,
    }
}
