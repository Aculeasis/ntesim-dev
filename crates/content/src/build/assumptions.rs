use super::*;
use crate::HypothesisId;

pub(crate) const STAT_QUANTIZATION_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::BuildStatQuantizationV1;

const WEAPON_HYPOTHESES: [(&str, HypothesisId); 13] = [
    (
        "weapon:fork_BlastCandy",
        crate::weapons::blast_candy::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_prism",
        crate::weapons::far_side_moon::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_oulaquantao",
        crate::weapons::oraora::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_wuhuakuang",
        crate::weapons::forgotten::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_Arachne",
        crate::weapons::eternal_waltz::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_wushoutieyu",
        crate::weapons::raging_flames::HYPOTHESIS_ID,
    ),
    ("weapon:fork_bopu", crate::weapons::bopu::HYPOTHESIS_ID),
    (
        "weapon:fork_koinobori",
        crate::weapons::koinobori::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_GoldRecord",
        crate::weapons::gold_record::HYPOTHESIS_ID,
    ),
    (
        "weapon:fork_Time",
        crate::weapons::marching_beyond_time::HYPOTHESIS_ID,
    ),
    ("weapon:fork_vine", crate::weapons::be_happy::HYPOTHESIS_ID),
    ("weapon:fork_spider", crate::weapons::spider::HYPOTHESIS_ID),
    (
        "weapon:fork_BitGame",
        crate::weapons::bit_game::HYPOTHESIS_ID,
    ),
];

pub(crate) fn attach(scenario: &mut GameScenarioRequest, request: &Request) -> Result<(), String> {
    let mut insert = |id: &str, description: &str| -> Result<(), String> {
        if let Some(old) = scenario.model_assumptions.get(id) {
            if old != description {
                return Err(format!("conflicting compiled model assumption: {id}"));
            }
        } else {
            scenario
                .model_assumptions
                .insert(id.into(), description.into());
        }
        Ok(())
    };
    insert("build.static_additive_v1", &request.hypothesis)?;
    if scenario.actors.iter().any(|actor| {
        actor
            .weapon_id
            .eq_ignore_ascii_case("weapon:fork_mofeikesi")
    }) {
        let id = crate::weapons::good_boy::HYPOTHESIS_ID;
        insert(id.as_str(), id.description())?;
    }
    insert(
        STAT_QUANTIZATION_HYPOTHESIS_ID.as_str(),
        STAT_QUANTIZATION_HYPOTHESIS_ID.description(),
    )?;
    if scenario
        .actors
        .iter()
        .any(|actor| actor.set_id.as_deref() == Some("set:Suit2"))
    {
        insert(
            crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.as_str(),
            crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.description(),
        )?;
        insert(
            crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.as_str(),
            crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.description(),
        )?;
    }
    if let Some(projection) = &request.action_projection {
        insert("build.action_projection_v1", &projection.hypothesis)?;
        if matches!(
            projection.policy,
            ActionProjectionPolicy::NativeMintSkillLevelV1
        ) {
            insert(
                HypothesisId::MintSkillCoordinatesV1.as_str(),
                HypothesisId::MintSkillCoordinatesV1.description(),
            )?;
            insert(
                HypothesisId::MintResonance3V1.as_str(),
                HypothesisId::MintResonance3V1.description(),
            )?;
        } else if matches!(
            projection.policy,
            ActionProjectionPolicy::NativeSkillLevelV1
        ) {
            insert(
                HypothesisId::SkillLevelsCoordinatesV1.as_str(),
                HypothesisId::SkillLevelsCoordinatesV1.description(),
            )?;
            insert(
                HypothesisId::SkillLevelsResonance3V1.as_str(),
                HypothesisId::SkillLevelsResonance3V1.description(),
            )?;
        }
    }
    if let Some(projection) = &request.gear_projection {
        for (weapon, id) in WEAPON_HYPOTHESES {
            if scenario
                .actors
                .iter()
                .any(|actor| actor.weapon_id.eq_ignore_ascii_case(weapon))
            {
                insert(id.as_str(), id.description())?;
            }
        }
        insert("build.gear_projection_v1", &projection.hypothesis)?;
        if scenario.actors.iter().any(|actor| {
            actor
                .weapon_id
                .eq_ignore_ascii_case("weapon:fork_BitterCake")
        }) {
            insert(
                crate::weapons::bittercake::HYPOTHESIS_ID.as_str(),
                crate::weapons::bittercake::HYPOTHESIS_ID.description(),
            )?;
        }
        if !scenario.incoming_damage_events.is_empty() {
            insert(
                crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.as_str(),
                crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.description(),
            )?;
        }
        if scenario
            .actors
            .iter()
            .any(|actor| actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Castle"))
        {
            insert(
                crate::weapons::castle::HYPOTHESIS_ID.as_str(),
                crate::weapons::castle::HYPOTHESIS_ID.description(),
            )?;
            insert(
                crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str(),
                crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description(),
            )?;
        }
        if scenario
            .actors
            .iter()
            .any(|actor| actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Door"))
        {
            insert(
                crate::weapons::wrong_gate::HYPOTHESIS_ID.as_str(),
                crate::weapons::wrong_gate::HYPOTHESIS_ID.description(),
            )?;
        }
        if matches!(
            projection.policy,
            GearProjectionPolicy::NativeMindRoyaleFirefliesSoloV1
        ) {
            insert(
                "fireflies.solo_proc_v2",
                "Declared set count 4 activates Fireflies; selected scalar is fractional CRIT damage, stack count is integral, and CD is per-stack seconds. Solo Anima deliveries to declared all-in-range dummies count as nearby team damage. Stacks expire independently before equal-time hits; capped hits do not refresh. Native trigger and refresh semantics remain unresolved.",
            )?;
        } else if matches!(projection.policy, GearProjectionPolicy::NativeWeaponCurveV1) {
            insert(
                "weapons.native_curve_v1",
                "Equipped native weapon parameters select exact refinement values: Mind Royale grants energy on Skill; Good Boy grants team ATK percentage on Ultimate; Bittercake grants DEF on explicit incoming-damage triggers.",
            )?;
        }
    }
    crate::simulation::validate_model_assumptions(scenario)
}
