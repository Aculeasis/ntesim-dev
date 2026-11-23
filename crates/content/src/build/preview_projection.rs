use super::*;

use ntesim_contracts::hypotheses::HypothesisId;

pub const PREVIEW_SINGLE_PACKET_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ResearchPreviewSinglePacketV1;
pub const PREVIEW_SINGLE_PACKET_HYPOTHESIS_DESCRIPTION: &str =
    PREVIEW_SINGLE_PACKET_HYPOTHESIS_ID.description();

#[derive(Clone)]
pub struct PreviewActionRequest<'a> {
    pub content_id: &'a str,
    pub character_id: &'a str,
    pub ability_id: &'a str,
    pub damage_effect_id: &'a str,
    pub purchased_skill_level: u32,
    pub awakening: ntesim_data::AwakeningSelection,
    pub model_assumptions: &'a BTreeMap<String, String>,
}

pub(crate) fn effective_skill_level(
    tables: &BTreeMap<String, Value>,
    character_id: &str,
    ability_id: &str,
    purchased_level: u32,
    breakthrough: u32,
    awakening: &ntesim_data::AwakeningSelection,
) -> Result<Value, String> {
    crate::registry::validate_awakening_selection(character_id, awakening)?;
    let purchase = ntesim_data::skill_levels::compile(
        tables,
        character_id,
        ability_id,
        purchased_level,
        breakthrough,
        awakening.count(),
    )?;
    let canonical_ability = purchase["ability_id"]
        .as_str()
        .ok_or("canonical skill ability missing")?;
    let (bonus, resonance) =
        crate::characters::resonance::bonus(tables, character_id, canonical_ability, awakening)?;
    let effective_level = purchased_level
        .checked_add(bonus)
        .ok_or("effective skill level overflow")?;
    let array_index = effective_level
        .checked_sub(1)
        .ok_or("effective skill level must be positive")?;

    Ok(json!({
        "ability_id": canonical_ability,
        "purchase": purchase,
        "resonance": resonance,
        "purchased_level": purchased_level,
        "effective_level": effective_level,
        "array_index": array_index,
        "model_assumption": "Native damage array coordinate equals effective skill level minus one."
    }))
}

pub fn project_preview_action(
    tables: &BTreeMap<String, Value>,
    request: &PreviewActionRequest<'_>,
) -> Result<Value, String> {
    crate::registry::validate_awakening_selection(request.character_id, &request.awakening)?;
    let PreviewActionRequest {
        content_id,
        character_id,
        ability_id,
        damage_effect_id,
        purchased_skill_level,
        awakening,
        model_assumptions,
    } = request.clone();
    if content_id != "ntesim-current" {
        return Err(format!(
            "unsupported preview package identity: {content_id}"
        ));
    }
    let progression = ntesim_data::skill_levels::progression(character_id, 80, Some(80))?;
    let breakthrough: u32 = progression["breakthrough"]
        .as_u64()
        .ok_or("native level-80 breakthrough missing")?
        .try_into()
        .map_err(|_| "native breakthrough overflow")?;
    let skill_level = effective_skill_level(
        tables,
        character_id,
        ability_id,
        purchased_skill_level,
        breakthrough,
        &awakening,
    )?;
    let effective_level = skill_level["effective_level"]
        .as_u64()
        .ok_or("effective skill level missing")?;
    let array_index: usize = skill_level["array_index"]
        .as_u64()
        .ok_or("native damage coordinate missing")?
        .try_into()
        .map_err(|_| "native damage coordinate overflow")?;

    let array_index: u32 = array_index
        .try_into()
        .map_err(|_| "native damage coordinate exceeds u32")?;
    let packet_projection = super::actions::project_native_packet_coefficient(
        tables,
        &super::actions::PacketCoefficientRequest {
            content_id,
            character_id,
            ability_id,
            damage_effect_id,
            array_index,
            research_metadata: None,
            model_assumptions,
        },
    )?;
    let report = &packet_projection["native_damage_report"];
    let effect = &report["effects"][0];
    let compiled_rate: Ratio =
        serde_json::from_value(packet_projection["compiled_rate"].clone())
            .map_err(|error| format!("compiled preview coefficient invalid: {error}"))?;
    let compiled_scaling_stat: crate::simulation::NativeScalingStat =
        serde_json::from_value(packet_projection["compiled_scaling_stat"].clone())
            .map_err(|error| format!("compiled preview scaling stat invalid: {error}"))?;

    Ok(json!({
        "content_id": content_id,
        "character_id": character_id,
        "ability_id": report["ability_id"],
        "damage_effect_id": effect["effect_id"],
        "compiled_rate": compiled_rate,
        "compiled_scaling_stat": compiled_scaling_stat,
        "packet_model_assumptions": if character_id == "character:1033" && ability_id == "GA_Adler_Skill" {
            BTreeMap::from([
                (crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS, crate::simulation::metadata::FIXED_CRIT_PROBABILITY_DESCRIPTION),
                (crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.as_str(), crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.description()),
            ])
        } else {
            BTreeMap::new()
        },
        "array_index": packet_projection["array_index"],
        "purchased_skill_level": purchased_skill_level,
        "effective_skill_level": effective_level,
        "skill_level_receipt": skill_level,
        "native_damage_report": report,
        "packet_metadata": packet_projection["packet_metadata"],
        "binding": packet_projection["binding"],
        "model_assumption_id": PREVIEW_SINGLE_PACKET_HYPOTHESIS_ID.as_str(),
        "model_assumption": PREVIEW_SINGLE_PACKET_HYPOTHESIS_DESCRIPTION
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_coefficients_are_source_projected_and_non_atk_profiles_fail_explicitly() {
        let tables = ntesim_data::skill_levels::frozen::tables();
        let assumptions = BTreeMap::from([(
            PREVIEW_SINGLE_PACKET_HYPOTHESIS_ID.into(),
            PREVIEW_SINGLE_PACKET_HYPOTHESIS_DESCRIPTION.into(),
        )]);
        let mut projected = 0;
        let mut unsupported = BTreeSet::new();

        for descriptor in crate::registry::character_descriptors().unwrap() {
            let profile = crate::registry::get_character_profile(descriptor.slug).unwrap();
            let result = project_preview_action(
                tables,
                &PreviewActionRequest {
                    content_id: "ntesim-current",
                    character_id: profile.character_id,
                    ability_id: profile.ability_id,
                    damage_effect_id: profile.damage_effect_id,
                    purchased_skill_level: 8,
                    awakening: profile.awakening,
                    model_assumptions: &assumptions,
                },
            );
            match result {
                Ok(receipt) => {
                    assert_ne!(descriptor.slug, "fadia");
                    let effect = &receipt["native_damage_report"]["effects"][0];
                    assert_eq!(receipt["ability_id"], profile.ability_id);
                    assert_eq!(receipt["damage_effect_id"], profile.damage_effect_id);
                    let scaling_stat = receipt["compiled_scaling_stat"].as_str().unwrap();
                    let selected_rate = &effect["arrays"][match scaling_stat {
                        "atk" => "AtkRateBaseArray",
                        "def" => "DefRateBaseArray",
                        other => panic!("unexpected preview scaling stat: {other}"),
                    }]["selected_value"];
                    let selected_numerator: u64 = selected_rate["numerator"]
                        .as_str()
                        .expect("exact native rate numerator")
                        .parse()
                        .unwrap();
                    let selected_denominator: u64 = selected_rate["denominator"]
                        .as_str()
                        .expect("exact native rate denominator")
                        .parse()
                        .unwrap();
                    assert_eq!(
                        receipt["compiled_rate"],
                        format!("{selected_numerator}/{selected_denominator}")
                    );
                    assert_eq!(
                        receipt["packet_metadata"]["source_evidence"]["table"],
                        ntesim_data::native_damage::TABLE_NAME
                    );
                    projected += 1;
                }
                Err(error) => {
                    assert_eq!(descriptor.slug, "fadia");
                    assert_eq!(
                        error,
                        "native action HP scaling is unsupported for this owner/action/packet"
                    );
                    unsupported.insert(descriptor.slug);
                }
            }
        }

        assert_eq!(projected, 22);
        assert_eq!(unsupported, BTreeSet::from(["fadia"]));
    }

    #[test]
    fn preview_boundaries_reject_selected_effects_for_unknown_owners() {
        let tables = ntesim_data::skill_levels::frozen::tables();
        let assumptions = BTreeMap::from([(
            PREVIEW_SINGLE_PACKET_HYPOTHESIS_ID.into(),
            PREVIEW_SINGLE_PACKET_HYPOTHESIS_DESCRIPTION.into(),
        )]);
        let awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
        let error = project_preview_action(
            tables,
            &PreviewActionRequest {
                content_id: "ntesim-current",
                character_id: "character:9999",
                ability_id: "GA_Mint019_Skill",
                damage_effect_id: "GE_Player_Mint_Skill_Damage",
                purchased_skill_level: 1,
                awakening: awakening.clone(),
                model_assumptions: &assumptions,
            },
        )
        .unwrap_err();
        assert!(error.contains("character:9999 does not support selected awakening effects"));
        assert!(
            effective_skill_level(
                tables,
                "character:9999",
                "GA_Mint019_Skill",
                1,
                0,
                &awakening,
            )
            .unwrap_err()
            .contains("character:9999 does not support selected awakening effects")
        );
    }
}
