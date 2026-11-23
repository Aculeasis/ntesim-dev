use super::*;

pub fn documented_discrepancies() -> Vec<DocumentedDiscrepancy> {
    vec![
        DocumentedDiscrepancy {
            discrepancy_id: "negative_resistance_formula".into(),
            source: "hxwd94666/NTE-Drive-Calculator (docs/reference/damage/core-formula.md:187)".into(),
            standard_formula: "1.0 - R / (1.0 - R)".into(),
            community_formula: "1.0 - R / 1.10".into(),
            numerical_impact: "At R = -0.20, Drive Calculator yields 1.181818 (+1.30% higher) vs standard 1.166667. Positive resistance matches identically (1 - R = 0.80).".into(),
            engine_policy: "Simulator implements both via NegativeResistanceModel enum. Default is StandardRational; LinearScaled mode available for exact reproduction.".into(),
            status: "isolated_and_reproducible".into(),
        },
        DocumentedDiscrepancy {
            discrepancy_id: "nightmare_stack_expiry_policy".into(),
            source: "hxwd94666/NTE-Drive-Calculator (docs/reference/damage/special-mechanics.md:39)".into(),
            standard_formula: "GE_Player_Lacrimosa_BloodDam: RemoveSingleStackAndRefreshDuration".into(),
            community_formula: "Independent per-application duration expiration".into(),
            numerical_impact: "Independent timers lead to faster stack decay under low refresh rates. Native GameplayEffect uses refresh-on-stack.".into(),
            engine_policy: "Selected executable Nightmare model refreshes the shared 10s duration with a 10-stack cap; native owner/source tests retain continuing ticks. See docs/specs/characters/lacrimosa.md; this formula battery does not test lifecycle.".into(),
            status: "selected_model_checked_game_policy_unconfirmed".into(),
        },
        DocumentedDiscrepancy {
            discrepancy_id: "scorch_trigger_definition".into(),
            source: "hxwd94666/NTE-Drive-Calculator (docs/reference/damage/special-mechanics.md:185)".into(),
            standard_formula: "Native ZhouAnReactionSixGEComp: 0.99s period, 15s duration on application".into(),
            community_formula: "Observed non-Scorch DoT ticks activate and add Scorch layers".into(),
            numerical_impact: "Alters DoT layering frequency in multi-element comps.".into(),
            engine_policy: "Selected executable model admits Scorch through qualified Incantation/Chaos Support switches, preserving .99s tick phase across refresh and 15s expiry. See docs/mechanics/shared/cycles-and-break.md; no ordinary DoT-triggered layering is inferred.".into(),
            status: "selected_model_checked_game_activation_unconfirmed".into(),
        },
    ]
}
