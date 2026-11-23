use super::*;
use crate::simulation::ResearchActionMetadata;

pub(crate) struct PacketCoefficientRequest<'a> {
    pub content_id: &'a str,
    pub character_id: &'a str,
    pub ability_id: &'a str,
    pub damage_effect_id: &'a str,
    pub array_index: u32,
    pub research_metadata: Option<&'a ResearchActionMetadata>,
    pub model_assumptions: &'a BTreeMap<String, String>,
}

pub(crate) fn supports_hp_packet(character_id: &str, ability_id: &str, effect_id: &str) -> bool {
    character_id == "character:1039"
        && ((ability_id == "GA_Fadia_Transfer" && effect_id == "GE_Player_Fadia_ZhouYin_Damage")
            || (ability_id == "GA_Fadia_Skill"
                && matches!(
                    effect_id,
                    "GE_Player_Fadia_Skill1_Damage"
                        | "GE_Player_Fadia_Skill2_Damage"
                        | "GE_Player_Fadia_Skill3_Damage"
                        | "GE_Player_Fadia_ZhouYin_Damage"
                ))
            || (matches!(ability_id, "GA_Fadia_Melee" | "GA_Fadia_UltraSkill")
                && matches!(
                    effect_id,
                    "GE_Player_Fadia_UltraSkill1_Damage"
                        | "GE_Player_Fadia_UltraSkill2_Damage"
                        | "GE_Player_Fadia_UltraSkill3_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee1_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee2_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee3_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee4_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee5_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee6_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee7_Damage"
                        | "GE_Player_Fadia_UltraSkillMelee8_Damage"
                )))
}

/// Compile one selected packet's coefficient from its frozen source row.
/// Both scenario action projection and historical preview profiling use this
/// function, so neither path can substitute a manual scalar.
pub(crate) fn project_native_packet_coefficient(
    tables: &BTreeMap<String, Value>,
    projection: &PacketCoefficientRequest<'_>,
) -> Result<Value, String> {
    let native_action = ntesim_data::native_action_metadata::resolve_ability(
        projection.content_id,
        tables,
        projection.character_id,
        projection.ability_id,
    )?;
    let native_packet = ntesim_data::native_action_metadata::resolve_damage_effect(
        projection.content_id,
        tables,
        projection.damage_effect_id,
    )?;
    let request = damage_values::Request {
        schema_version: 1,
        content_id: projection.content_id.to_owned(),
        character_id: projection.character_id.to_owned(),
        ability_id: projection.ability_id.to_owned(),
        effect_ids: vec![projection.damage_effect_id.to_owned()],
        array_index: projection.array_index,
    };
    let owner_binding = crate::characters::action_packet_bindings::resolve(
        projection.character_id,
        &native_action.ability_id,
        &native_packet.effect_id,
    );
    let report = if let Some(binding) = owner_binding {
        damage_values::compile_owner_packet(tables, &request, binding.packet_ga_name)?
    } else {
        damage_values::compile(tables, &request)?
    };
    let effect = &report["effects"][0];
    let scaling = &effect["arrays"];
    let selected_rate = |name: &str| -> Result<Option<ExactFraction>, String> {
        match scaling[name]["state"].as_str() {
            Some("selected_native_coordinate") => {
                receipt(&scaling[name]["selected_value"]).map(Some)
            }
            Some("declared_empty") => Ok(None),
            _ => Err(format!(
                "native action scaling coordinate unavailable: {name}"
            )),
        }
    };
    let atk_rate = selected_rate("AtkRateBaseArray")?;
    let def_rate = selected_rate("DefRateBaseArray")?;
    let hp_rate = selected_rate("HPRateBaseArray")?;
    let hp_action = supports_hp_packet(
        projection.character_id,
        &native_action.ability_id,
        &native_packet.effect_id,
    ) && (matches!(
        native_packet.effect_id.as_str(),
        "GE_Player_Fadia_UltraSkill1_Damage"
            | "GE_Player_Fadia_UltraSkill2_Damage"
            | "GE_Player_Fadia_UltraSkill3_Damage"
    ) || projection
        .model_assumptions
        .get(crate::characters::fadia::effects::MODEL.as_str())
        .map(String::as_str)
        == Some(crate::characters::fadia::effects::MODEL.description()));
    if hp_rate.is_some_and(|rate| !rate.is_zero()) && !hp_action {
        return Err("native action HP scaling is unsupported for this owner/action/packet".into());
    }
    let selected = [
        (crate::simulation::NativeScalingStat::Atk, atk_rate),
        (crate::simulation::NativeScalingStat::Def, def_rate),
        (crate::simulation::NativeScalingStat::Hp, hp_rate),
    ];
    let mut nonzero = selected
        .into_iter()
        .filter_map(|(stat, rate)| rate.filter(|rate| !rate.is_zero()).map(|rate| (stat, rate)));
    let nonzero_scaling = nonzero.next();
    // A selected literal zero is executable; an empty array supplies no stat.
    // Prefer the nonzero term when other arrays explicitly select zero.
    let zero = ExactFraction::new(0, 1)?;
    let (scaling_stat, compiled_rate) = if nonzero.next().is_some() {
        (
            crate::simulation::NativeScalingStat::Mixed {
                atk: ratio(atk_rate.unwrap_or(zero))?,
                def: ratio(def_rate.unwrap_or(zero))?,
                hp: ratio(hp_rate.unwrap_or(zero))?,
            },
            Ratio::one(),
        )
    } else {
        let (stat, rate) = nonzero_scaling
            .or_else(|| {
                selected.into_iter().find_map(|(stat, rate)| {
                    rate.filter(|_| stat != crate::simulation::NativeScalingStat::Hp || hp_action)
                        .map(|rate| (stat, rate))
                })
            })
            .ok_or("native action has no selected ATK or DEF scaling or supported HP scaling")?;
        (stat, ratio(rate)?)
    };
    if native_packet.damage_type_ex.as_deref() != effect["damage_type"].as_str() {
        return Err(
            "native action projection packet type disagrees with its frozen source row".into(),
        );
    }
    let action_packet_metadata = crate::simulation::metadata::resolve_requested_action_packet(
        projection.content_id,
        tables,
        projection.character_id,
        projection.ability_id,
        projection.damage_effect_id,
        projection.research_metadata,
        projection.model_assumptions,
    )?;
    if !action_packet_metadata
        .packet
        .damage_effect_id
        .eq_ignore_ascii_case(projection.damage_effect_id)
    {
        return Err("resolved action/packet binding changed the selected native row".into());
    }
    Ok(json!({
        "compiled_atk_rate": ratio(atk_rate.unwrap_or(zero))?,
        "compiled_scaling_stat": scaling_stat,
        "compiled_rate": compiled_rate,
        "array_index": projection.array_index,
        "native_damage_report": report,
        "packet_metadata": action_packet_metadata.packet,
        "action_metadata": action_packet_metadata.action,
        "binding": action_packet_metadata.binding,
    }))
}

#[cfg(test)]
pub(crate) fn project_actions(
    tables: &BTreeMap<String, Value>,
    scenario: &mut GameScenarioRequest,
    projection: &ActionProjection,
) -> Result<Vec<Value>, String> {
    project_actions_with_levels(tables, scenario, projection, &BTreeMap::new())
}

pub(crate) fn project_actions_with_levels(
    tables: &BTreeMap<String, Value>,
    scenario: &mut GameScenarioRequest,
    projection: &ActionProjection,
    skill_levels: &BTreeMap<usize, BTreeMap<String, Value>>,
) -> Result<Vec<Value>, String> {
    if projection.hypothesis.trim().is_empty() {
        return Err("native action projection requires an explicit hypothesis".into());
    }
    let mut receipts = Vec::new();
    let mut actions = scenario.actions.clone();
    for (index, command) in actions.iter_mut().enumerate() {
        let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            array_index,
            atk_rate,
            scaling_stat,
            ..
        } = command
        else {
            continue;
        };
        let actor = scenario
            .actors
            .get(*actor_index)
            .ok_or("native action actor index out of bounds")?;
        let replaced_array_index = *array_index;
        let level = if matches!(
            projection.policy,
            ActionProjectionPolicy::NativeMintSkillLevelV1
                | ActionProjectionPolicy::NativeSkillLevelV1
        ) {
            let levels = skill_levels
                .get(&actor.slot)
                .ok_or("compiled actor skill levels missing")?;
            let matching = levels
                .iter()
                .filter(|(id, _)| id.eq_ignore_ascii_case(ability_id))
                .collect::<Vec<_>>();
            if matching.len() != 1 {
                return Err("invoked ability requires exactly one purchased skill level".into());
            }
            let level = matching[0].1;
            *array_index = level["array_index"]
                .as_u64()
                .ok_or("compiled skill coordinate missing")?
                .try_into()
                .map_err(|_| "skill coordinate overflow")?;
            Some(level)
        } else {
            None
        };
        let packet_projection = project_native_packet_coefficient(
            tables,
            &PacketCoefficientRequest {
                content_id: &scenario.content_id,
                character_id: &actor.character_id,
                ability_id,
                damage_effect_id,
                array_index: (*array_index)
                    .try_into()
                    .map_err(|_| "native action array index exceeds u32")?,
                research_metadata: scenario.action_metadata.get(&index),
                model_assumptions: &scenario.model_assumptions,
            },
        )?;
        let report = &packet_projection["native_damage_report"];
        let effect = &report["effects"][0];
        let compiled: Ratio = serde_json::from_value(packet_projection["compiled_rate"].clone())
            .map_err(|error| format!("compiled action coefficient invalid: {error}"))?;
        let compiled_scaling_stat: crate::simulation::NativeScalingStat =
            serde_json::from_value(packet_projection["compiled_scaling_stat"].clone())
                .map_err(|error| format!("compiled action scaling stat invalid: {error}"))?;
        receipts.push(json!({"command_index":index, "replaced_atk_rate":*atk_rate,
            "compiled_atk_rate":packet_projection["compiled_atk_rate"],
            "compiled_scaling_stat":compiled_scaling_stat, "compiled_rate":compiled, "native_damage_report":report,
            "packet_metadata":packet_projection["packet_metadata"],
            "binding":packet_projection["binding"],
            "replaced_array_index":replaced_array_index,"compiled_array_index":*array_index,"skill_level":level}));
        *atk_rate = compiled;
        *scaling_stat = compiled_scaling_stat;
        *ability_id = report["ability_id"]
            .as_str()
            .ok_or("canonical ability ID missing")?
            .into();
        *damage_effect_id = effect["effect_id"]
            .as_str()
            .ok_or("canonical effect ID missing")?
            .into();
    }
    // A legal party may have only autonomous effects. Their packets project at
    // live admission; purchased talent levels still belong to the compiled build.
    scenario.actions = actions;
    Ok(receipts)
}
