//! The studio prepares declarations; this shared Rust path owns their stat math.
use super::*;

pub(crate) const STATIC_ADDITIVE_MODEL: (&str, &str) = (
    "build.static_additive_v1",
    "Native base, level and terminal breakthrough constants plus equipped weapon and unconditional set stats; add stats supplies additive cartridge/module contributions. HP and DEF use build.stat_quantization_v1.",
);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BaseOverrides {
    pub atk: Option<u64>,
    pub hp: Option<u64>,
    pub def: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(default)]
    pub slot: usize,
    pub content_id: String,
    pub character_id: String,
    pub level: u32,
    #[serde(default)]
    pub awakening: ntesim_data::AwakeningSelection,
    pub max_level: Option<u32>,
    pub weapon_id: Option<String>,
    pub weapon_level: Option<u32>,
    pub weapon_max_level: Option<u32>,
    pub refinement: u32,
    pub set_id: Option<String>,
    pub set_count: u32,
    #[serde(default)]
    pub base_overrides: BaseOverrides,
    #[serde(default)]
    pub additive_stats: BTreeMap<String, Ratio>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub decomposition: super::BuildDecomposition,
    pub static_stats: GameActorStaticStats,
    pub energy_capacity: u64,
    pub charge_efficiency: Ratio,
    pub model_assumptions: BTreeMap<String, String>,
    pub base: BTreeMap<String, u64>,
    pub evidence: Value,
    pub unmodeled_attributes: BTreeMap<String, Value>,
    /// Frozen per-ability native labels for studio presentation and diagnosis.
    /// GameplayTag is retained verbatim and does not define UI primary slots.
    pub native_action_metadata: Value,
    pub public_actions: BTreeMap<String, super::public_actions::PublicAction>,
    pub equipment_projection: super::studio_equipment::EquipmentProjection,
    pub stochastic_crit: crate::simulation::StochasticCritPolicy,
}

fn exact_receipt(value: Ratio) -> Result<Value, String> {
    let value = ExactFraction::from_ratio(value)?;
    Ok(
        json!({"numerator":value.numerator.to_string(), "denominator":value.denominator.to_string()}),
    )
}

fn native_action_metadata(
    content_id: &str,
    tables: &BTreeMap<String, Value>,
    character_id: &str,
) -> Result<Value, String> {
    if content_id != "ntesim-current" {
        return Err(format!("unsupported studio package identity: {content_id}"));
    }
    let native = character_id
        .strip_prefix("character:")
        .ok_or("canonical character ID required")?;
    let (config_table, config_rows) = ntesim_data::character_actions::rows(tables)?;
    let (config_id, config) = config_rows
        .iter()
        .find(|(id, _)| id.eq_ignore_ascii_case(native))
        .ok_or("native action character config missing")?;
    let configured = config["CharacterAbilityList"]
        .as_array()
        .ok_or("native character actions missing")?;
    let (_, damage_rows) = ntesim_data::native_damage::rows(tables)?;
    let mut actions = BTreeMap::new();
    for (index, item) in configured.iter().enumerate() {
        let ability_id = item["Key"].as_str().ok_or("native ability key missing")?;
        let ability_index = item["Value"]["AbilityIndex"].as_u64();
        let input_role = ntesim_data::native_action_metadata::resolve_primary_role(ability_index)?;
        let (input_role_name, primary_slot) = match input_role {
            ntesim_data::native_action_metadata::NativePrimaryRole::Normal => {
                ("normal", Some("attack"))
            }
            ntesim_data::native_action_metadata::NativePrimaryRole::RedirectSkill => {
                ("redirect_skill", Some("skill"))
            }
            ntesim_data::native_action_metadata::NativePrimaryRole::Ultimate => {
                ("ultimate", Some("burst"))
            }
            ntesim_data::native_action_metadata::NativePrimaryRole::Qte => ("qte", None),
        };
        let ability_metadata = ntesim_data::native_action_metadata::resolve_ability(
            content_id,
            tables,
            character_id,
            ability_id,
        )?;
        let mut effects = BTreeMap::new();
        for effect_id in damage_rows
            .iter()
            .filter(|(_, row)| {
                row["GAName"]
                    .as_str()
                    .is_some_and(|ga| ga.eq_ignore_ascii_case(ability_id))
            })
            .map(|(id, _)| id)
        {
            effects.insert(
                effect_id.clone(),
                ntesim_data::native_action_metadata::resolve_damage_effect(
                    content_id, tables, effect_id,
                )?,
            );
        }
        actions.insert(
            ability_id.to_owned(),
            json!({
                "input_role":input_role_name,
                "native_ability_index":ability_index,
                "ui_primary_slot":primary_slot,
                "ability_config_evidence":{
                    "content_id":content_id,
                    "table":config_table,
                    "row":config_id,
                    "pointer":format!("/rows/{config_id}/CharacterAbilityList/{index}/Value/AbilityIndex"),
                    "raw_action":item,
                },
                "ability_metadata":ability_metadata,
                "damage_effects":effects,
            }),
        );
    }
    Ok(json!({
        "content_id":content_id,
        "character_id":character_id,
        "input_role_mapping":{"1":"normal","2":"redirect_skill","3":"ultimate","4":"qte"},
        "input_role_mapping_hypothesis_id":ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
        "input_role_mapping_hypothesis":ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_DESCRIPTION,
        "input_role_source":format!("{config_table}.CharacterAbilityList[].Value.AbilityIndex"),
        "gameplay_tags_are_combat_metadata_only":true,
        "actions":actions,
    }))
}

fn project_static_weapon_hp(
    tables: &BTreeMap<String, Value>,
    request: &Request,
    additives: &mut BTreeMap<String, Value>,
) -> Result<Option<Value>, String> {
    let static_hp_weapon = match request.weapon_id.as_deref() {
        Some(weapon_id) => super::weapon_projection::compile_static_hp_weapon(
            tables,
            1,
            &request.content_id,
            &request.character_id,
            weapon_id,
            request.refinement,
        )?,
        None => None,
    };
    if let Some(weapon) = &static_hp_weapon {
        let previous = additives
            .get("HPMaxUp")
            .map(receipt)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?);
        additives.insert(
            "HPMaxUp".into(),
            exact_receipt(ratio(previous.add(ExactFraction::from_ratio(
                serde_json::from_value(weapon["hp_bonus"].clone()).map_err(|e| e.to_string())?,
            )?)?)?)?,
        );
    }
    Ok(static_hp_weapon)
}

fn record_static_weapon_hp(
    model_assumptions: &mut BTreeMap<String, String>,
    evidence: &mut Value,
    static_hp_weapon: Option<Value>,
    attributes: &Value,
    projected: &Value,
) -> Result<(), String> {
    if let Some(weapon) = &static_hp_weapon {
        let hypothesis_id = weapon["hypothesis_id"]
            .as_str()
            .ok_or("static HP weapon hypothesis_id must be a string")?;
        let hypothesis = weapon["hypothesis"]
            .as_str()
            .ok_or("static HP weapon hypothesis must be a string")?;
        model_assumptions.insert(hypothesis_id.into(), hypothesis.into());
    }
    if let Some(weapon) = static_hp_weapon {
        evidence
            .as_array_mut()
            .ok_or("native stat evidence must be an array")?
            .push(json!({
                "id":"weapon_static_hp_projection", "weapon":weapon,
                "native_base_hp":attributes["HPMaxBase"],"native_hp_percent":attributes["HPMaxUp"],
                "projected_hp":projected["derived"]["hp_receipt"]
            }));
    }
    Ok(())
}

pub(super) fn additive_property(name: &str) -> Result<&'static str, String> {
    Ok(match name {
        "healing_bonus" => "HealUp",
        "mental_damage_bonus" => "DamageUpPsychicallyBase",
        "shield_bonus" => "ShieldEfficiency",
        "atk" => "AtkAdd",
        "hp" => "HPMaxAdd",
        "def" => "DefAdd",
        "crit_rate" => "CritBase",
        "crit_damage_bonus" => "CritDamageBase",
        "damage_bonus" => "DamageUpGeneralBase",
        "anima_bonus" => "DamageUpNatureBase",
        "chaos_bonus" => "DamageUpChaosBase",
        "cosmos_bonus" => "DamageUpCosmosBase",
        "incantation_bonus" => "DamageUpIncantationBase",
        "lakshana_bonus" => "DamageUpLakshanaBase",
        "psyche_bonus" => "DamageUpPsycheBase",
        "atk_percent" => "AtkUp",
        "hp_percent" => "HPMaxUp",
        "def_percent" => "DefUp",
        "charge_efficiency" => "ChargeGetEfficiencyBase",
        "cycle_intensity" => "MagBase",
        "break_intensity" => "UnbalIntensityBase",
        _ => return Err(format!("unsupported studio additive stat {name}")),
    })
}

fn project_static_weapon_charge(
    tables: &BTreeMap<String, Value>,
    request: &Request,
    additives: &mut BTreeMap<String, Value>,
) -> Result<Option<Value>, String> {
    let good_boy_charge = crate::weapons::good_boy::projection::static_charge(
        tables,
        &request.content_id,
        request.weapon_id.as_deref(),
        request.refinement,
    )?;
    if let Some(weapon) = &good_boy_charge {
        let current = additives
            .get("ChargeGetEfficiencyBase")
            .map(receipt)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?);
        let bonus: Ratio = serde_json::from_value(weapon["charge_efficiency_bonus"].clone())
            .map_err(|e| e.to_string())?;
        additives.insert(
            "ChargeGetEfficiencyBase".into(),
            exact_receipt(ratio(current.add(ExactFraction::from_ratio(bonus)?)?)?)?,
        );
    }
    Ok(good_boy_charge)
}

pub fn project(tables: &BTreeMap<String, Value>, request: &Request) -> Result<Response, String> {
    crate::registry::validate_awakening_selection(&request.character_id, &request.awakening)?;
    if request.content_id != "ntesim-current" {
        return Err(format!(
            "unsupported studio package identity: {}",
            request.content_id
        ));
    }
    let element = ntesim_data::character::definition(&request.character_id)?
        .element
        .native_key();
    element_attribute(element)?;
    let mut additives = BTreeMap::new();
    for (name, value) in &request.additive_stats {
        let property = additive_property(name)?;
        let value = exact_receipt(*value).map_err(|error| {
            if name == "charge_efficiency" {
                format!("invalid charge efficiency addition: {error}")
            } else {
                error
            }
        })?;
        additives.insert(property.into(), value);
    }
    let edgar_hp_bonus = crate::characters::edgar::awakenings::owned_max_hp_bonus(
        &request.character_id,
        &request.awakening,
    );
    if edgar_hp_bonus.numerator != 0 {
        let existing = additives
            .get("HPMaxUp")
            .map(receipt)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?);
        additives.insert(
            "HPMaxUp".into(),
            exact_receipt(ratio(
                existing.add(ExactFraction::from_ratio(edgar_hp_bonus)?)?,
            )?)?,
        );
    }
    let mut eternal_waltz_receipt = None;
    if request
        .weapon_id
        .as_deref()
        .is_some_and(|weapon| weapon.eq_ignore_ascii_case("weapon:fork_Arachne"))
    {
        let native = crate::weapons::eternal_waltz::projection::compile_native_weapon(
            tables,
            1,
            &request.content_id,
            request.weapon_id.as_deref().unwrap_or_default(),
            request.refinement,
        )?;
        let parameters = native["parameters"]
            .as_array()
            .ok_or("Eternal Waltz parameters missing")?;
        let hp_bonus = receipt(&parameters[0]["selected"]["value"])?;
        let hp_percent_before = additives
            .get("HPMaxUp")
            .map(receipt)
            .transpose()?
            .unwrap_or(ExactFraction::new(0, 1)?);
        let hp_percent_after = hp_percent_before.add(hp_bonus)?;
        additives.insert("HPMaxUp".into(), exact_receipt(ratio(hp_percent_after)?)?);
        eternal_waltz_receipt = Some(json!({
            "weapon_id":request.weapon_id,
            "refinement":request.refinement,
            "native_parameter":parameters[0],
            "native_parameter_report":native,
            "hp_percent_before":exact_receipt(ratio(hp_percent_before)?)?,
            "hp_percent_after":exact_receipt(ratio(hp_percent_after)?)?,
            "settlement":"Arachne HP ratio is added to the existing HPMaxUp domain before native max-HP projection"
        }));
    }
    let static_hp_weapon = project_static_weapon_hp(tables, request, &mut additives)?;
    let good_boy_charge = project_static_weapon_charge(tables, request, &mut additives)?;
    let mut overrides = BTreeMap::new();
    for (name, value) in [
        ("AtkBase", request.base_overrides.atk),
        ("HPMaxBase", request.base_overrides.hp),
        ("DefBase", request.base_overrides.def),
    ] {
        if let Some(value) = value {
            overrides.insert(
                name.into(),
                json!({"numerator":value.to_string(),"denominator":"1"}),
            );
        }
    }
    let projected = ntesim_data::stat_projection::compile(
        tables,
        &ntesim_data::stat_projection::Request {
            character_id: request.character_id.clone(),
            level: request.level,
            max_level: request.max_level,
            weapon_id: request.weapon_id.clone(),
            weapon_level: request.weapon_level,
            weapon_max_level: request.weapon_max_level,
            refinement: request.refinement,
            set_id: request.set_id.clone(),
            set_count: request.set_count,
            base_overrides: overrides,
            additive_attributes: additives,
        },
    )?;
    let attributes = &projected["attributes"];
    let attribute = |name: &str| receipt(&attributes[name]);
    let optional = |name: &str| match attributes.get(name) {
        Some(value) => receipt(value),
        None => ExactFraction::new(0, 1),
    };
    if !optional("DamageUpNormalBase")?.is_zero() {
        return Err("unsupported studio typed bonus DamageUpNormalBase".into());
    }
    let base = BTreeMap::from([
        ("atk".into(), integer(&attributes["AtkBase"], "base ATK")?),
        ("hp".into(), integer(&attributes["HPMaxBase"], "base HP")?),
        ("def".into(), integer(&attributes["DefBase"], "base DEF")?),
    ]);
    let static_stats = GameActorStaticStats {
        base_atk: base["atk"],
        atk: ratio(receipt(&projected["derived"]["atk_receipt"])?)?,
        hp: nearest_integer(&projected["derived"]["hp_receipt"], "HP")?,
        base_hp: Some(base["hp"]),
        hp_percent: ratio(optional("HPMaxUp")?)?,
        hp_flat: ratio(optional("HPMaxAdd")?)?,
        def: nearest_integer(&projected["derived"]["def_receipt"], "DEF")?,
        base_def: Some(base["def"]),
        def_percent: ratio(optional("DefUp")?)?,
        def_flat: ratio(optional("DefAdd")?)?,
        crit_rate: ratio(attribute("CritBase")?)?,
        crit_damage_bonus: ratio(attribute("CritDamageBase")?)?,
        mental_damage_bonus: ratio(optional("DamageUpPsychicallyBase")?)?,
        healing_bonus: ratio(optional("HealUp")?)?,
        shield_bonus: ratio(optional("ShieldEfficiency")?)?,
        break_intensity: ratio(attribute("UnbalIntensityBase")?)?,
        cycle_intensity: ratio(optional("MagBase")?)?,
        break_efficiency_multiplier: ratio(attribute("UnbalAccrueEfficiencyBase")?)?,
        damage_bonus: ratio(optional("DamageUpGeneralBase")?)?,
        elemental_bonuses: super::stats::ELEMENT_ATTRIBUTES
            .iter()
            .map(|(element, property)| Ok(((*element).into(), ratio(optional(property)?)?)))
            .collect::<Result<_, String>>()?,
        elemental_bonus: Ratio::zero(),
    };
    let mut model_assumptions: BTreeMap<String, String> =
        serde_json::from_value(projected["model_assumptions"].clone())
            .map_err(|e| e.to_string())?;
    model_assumptions.insert(
        STATIC_ADDITIVE_MODEL.0.into(),
        STATIC_ADDITIVE_MODEL.1.into(),
    );
    let mut add_assumption = |id: crate::HypothesisId| {
        model_assumptions.insert(id.as_str().into(), id.description().into());
    };
    add_assumption(super::assumptions::STAT_QUANTIZATION_HYPOTHESIS_ID);
    // Edgar's off-field Support observer and key/field lifecycles also need
    // the selected model when the script never issues an Edgar action.
    if request.character_id == "character:1021" {
        add_assumption(crate::characters::edgar::clock::MODEL);
    }
    if request.character_id == "character:1023" {
        add_assumption(crate::characters::baicang::words::HYPOTHESIS_ID);
        add_assumption(crate::characters::baicang::cancel::HYPOTHESIS_ID);
        add_assumption(crate::characters::baicang::clock::HYPOTHESIS_ID);
    }
    if request.character_id == "character:1039" {
        add_assumption(crate::characters::fadia::effects::MODEL);
    }
    if good_boy_charge.is_some() {
        add_assumption(crate::weapons::good_boy::HYPOTHESIS_ID);
    }
    if eternal_waltz_receipt.is_some() {
        add_assumption(crate::weapons::eternal_waltz::HYPOTHESIS_ID);
    }
    model_assumptions.extend(public_actions::initial_assumptions(&request.character_id));
    if request.base_overrides.atk.is_some()
        || request.base_overrides.hp.is_some()
        || request.base_overrides.def.is_some()
    {
        model_assumptions.insert("build.base_overrides_v1".into(), "Explicit caller base overrides replace source-bound native base stats in this research projection.".into());
    }
    let consumed = super::stats::STATIC_ATTRIBUTES;
    let mut unmodeled_attributes = BTreeMap::new();
    for (name, value) in attributes
        .as_object()
        .ok_or("projected attributes missing")?
    {
        if !consumed.contains(&name.as_str()) && !receipt(value)?.is_zero() {
            unmodeled_attributes.insert(name.clone(), value.clone());
        }
    }
    if !unmodeled_attributes.is_empty() {
        model_assumptions.insert("build.unmodeled_native_attributes_v1".into(),
            "Nonzero native attributes outside static ATK/HP/DEF/crit/element bonus/energy projection are retained in unmodeled_attributes; their combat application is not implemented by this adapter.".into());
    }
    let mut evidence = projected["evidence"].clone();
    if let Some(weapon) = good_boy_charge {
        evidence
            .as_array_mut()
            .ok_or("native stat evidence must be an array")?
            .push(json!({"id":"good_boy_static_charge_projection","weapon":weapon}));
    }
    record_static_weapon_hp(
        &mut model_assumptions,
        &mut evidence,
        static_hp_weapon,
        attributes,
        &projected,
    )?;
    if let Some(receipt) = eternal_waltz_receipt {
        let weapon_evidence = json!({
            "id":"eternal_waltz_static_hp_projection",
            "native_base_hp":attributes["HPMaxBase"],
            "native_hp_percent":attributes["HPMaxUp"],
            "projected_hp":projected["derived"]["hp_receipt"],
            "weapon":receipt,
        });
        if let Some(items) = evidence.as_array_mut() {
            items.push(weapon_evidence);
        } else {
            return Err("native stat evidence must be an array".into());
        }
    }
    if edgar_hp_bonus.numerator != 0 {
        if let Some(items) = evidence.as_array_mut() {
            items.push(json!({
                "id":"edgar_a5_max_hp_projection",
                "awakening":request.awakening,
                "base_hp":base["hp"],
                "hp_percent_excluding_edgar_a5":exact_receipt(ratio(optional("HPMaxUp")?.sub(ExactFraction::from_ratio(edgar_hp_bonus)?)?)?)?,
                "hp_flat":exact_receipt(ratio(optional("HPMaxAdd")?)?)?,
                "edgar_a5_hp_percent":edgar_hp_bonus,
                "final_hp":static_stats.hp,
                "source":"typed Rust Edgar owned_max_hp_bonus; Global Effect5 / Buff_Edgar_Star5_HPMaxUP"
            }));
        } else {
            return Err("native stat evidence must be an array".into());
        }
    }
    let charge_efficiency = ratio(attribute("ChargeGetEfficiencyBase")?)?;
    let decomposition = super::decomposition::project(
        &projected,
        &request.additive_stats,
        edgar_hp_bonus,
        consumed,
    )?;
    decomposition.validate_static_stats(&static_stats)?;
    crate::gear::validate_charge_multiplier(charge_efficiency)?;
    let native_action_metadata =
        native_action_metadata(&request.content_id, tables, &request.character_id)?;
    let mut public_actions =
        super::public_actions::project(&request.character_id, &native_action_metadata)?;
    if let Some(id) = request
        .weapon_id
        .as_deref()
        .and_then(crate::gear::arc::hypothesis)
    {
        public_actions.insert("arc".into(), super::public_actions::arc(id));
    }
    let equipment_projection =
        super::studio_equipment::describe(tables, request, charge_efficiency);
    Ok(Response {
        decomposition,
        static_stats,
        energy_capacity: integer(&attributes["ChargeMax"], "energy capacity")?,
        charge_efficiency,
        model_assumptions,
        base,
        evidence,
        unmodeled_attributes,
        native_action_metadata,
        public_actions,
        equipment_projection,
        stochastic_crit: super::public_actions::critical_policy(
            &request.character_id,
            request.slot,
        )?,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn static_hp_evidence_requires_both_strings_before_recording() {
        use super::*;
        for weapon in [
            json!({"hypothesis":"rationale"}),
            json!({"hypothesis_id":42,"hypothesis":"rationale"}),
            json!({"hypothesis_id":"id"}),
            json!({"hypothesis_id":"id","hypothesis":false}),
        ] {
            let mut assumptions = BTreeMap::new();
            let mut evidence = json!([]);
            let error = record_static_weapon_hp(
                &mut assumptions,
                &mut evidence,
                Some(weapon),
                &json!({}),
                &json!({}),
            )
            .unwrap_err();
            assert!(error.contains("must be a string"));
            assert!(assumptions.is_empty());
            assert_eq!(evidence, json!([]));
        }
    }
    use super::*;

    fn request() -> Request {
        Request {
            slot: 0,
            content_id: "ntesim-current".into(),
            character_id: "character:1003".into(),
            level: 80,
            awakening: ntesim_data::AwakeningSelection::default(),
            max_level: Some(80),
            weapon_id: None,
            weapon_level: None,
            weapon_max_level: None,
            refinement: 1,
            set_id: None,
            set_count: 0,
            base_overrides: BaseOverrides {
                atk: Some(100),
                hp: Some(200),
                def: Some(50),
            },
            additive_stats: BTreeMap::from([
                (
                    "atk_percent".into(),
                    Ratio {
                        numerator: 1,
                        denominator: 3,
                    },
                ),
                (
                    "atk".into(),
                    Ratio {
                        numerator: 1,
                        denominator: 8,
                    },
                ),
                (
                    "hp".into(),
                    Ratio {
                        numerator: 1,
                        denominator: 2,
                    },
                ),
                (
                    "def".into(),
                    Ratio {
                        numerator: 1,
                        denominator: 3,
                    },
                ),
            ]),
        }
    }

    #[test]
    fn studio_projection_keeps_exact_atk_and_explicit_rounding_and_provenance() {
        let report = project(ntesim_data::stat_projection::frozen_tables(), &request()).unwrap();
        assert_eq!(
            report.static_stats.atk,
            Ratio {
                numerator: 3203,
                denominator: 24
            }
        );
        assert_eq!(report.static_stats.hp, 201);
        assert_eq!(report.static_stats.def, 50);
        // Sakiri's native base pool is 120, independent of demonstration pools.
        assert_eq!(report.energy_capacity, 120);
        assert!(
            report
                .model_assumptions
                .contains_key("build.base_overrides_v1")
        );
        let evidence = report.evidence.as_array().unwrap();
        assert_eq!(evidence.len(), 2);
        assert!(
            evidence
                .iter()
                .any(|row| row["kind"] == "typed_character_definition")
        );
        assert!(
            evidence
                .iter()
                .any(|row| row["kind"] == "typed_character_stats")
        );
        assert_eq!(
            report.native_action_metadata["content_id"],
            "ntesim-current"
        );
        assert_eq!(
            report.native_action_metadata["actions"]["GA_Sagiri_Melee"]["ui_primary_slot"],
            "attack"
        );
        assert_eq!(
            report.native_action_metadata["actions"]["GA_Sagiri_Melee"]["ability_metadata"]["gameplay_tag_names"]
                [0]["value"],
            "Ability.Melee"
        );
        assert_eq!(
            report.native_action_metadata["actions"]["GA_Sagiri_Melee"]["ability_metadata"]["evidence"]
                ["table"],
            ntesim_data::ability_tags::EVIDENCE_TABLE
        );
        assert_eq!(
            report.native_action_metadata["actions"]["GA_Sagiri_Melee"]["damage_effects"]["GE_Player_Sagiri_Melee1_Damage"]
                ["damage_type_ex"],
            "EExecutionDamageTypeEX::DAMAGE_TYPE_INCANTATION"
        );
    }

    #[test]
    fn charge_efficiency_addition_projects_into_native_multiplier_and_enforces_final_cap() {
        let mut input = request();
        let baseline = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(baseline.charge_efficiency, Ratio::one());
        input.additive_stats.insert(
            "charge_efficiency".into(),
            Ratio {
                numerator: 3,
                denominator: 25,
            },
        );
        let report = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(
            report.charge_efficiency,
            Ratio {
                numerator: 28,
                denominator: 25,
            }
        );
        input.additive_stats.insert(
            "charge_efficiency".into(),
            Ratio {
                numerator: 99,
                denominator: 1,
            },
        );
        assert!(project(ntesim_data::stat_projection::frozen_tables(), &input).is_ok());
        input.additive_stats.insert(
            "charge_efficiency".into(),
            Ratio {
                numerator: 100,
                denominator: 1,
            },
        );
        assert!(
            project(ntesim_data::stat_projection::frozen_tables(), &input)
                .unwrap_err()
                .contains("positive multiplier <= 100")
        );
        input
            .additive_stats
            .get_mut("charge_efficiency")
            .unwrap()
            .denominator = 0;
        assert!(
            project(ntesim_data::stat_projection::frozen_tables(), &input)
                .unwrap_err()
                .contains("invalid charge efficiency addition")
        );
    }

    #[test]
    fn edgar_a5_hp_uses_base_percent_and_flat_before_one_settlement() {
        let tables = ntesim_data::stat_projection::frozen_tables();
        let mut input = request();
        input.character_id = "character:1021".into();
        input.awakening = ntesim_data::AwakeningSelection::selected([1, 2, 3, 4, 6]).unwrap();
        input.base_overrides.hp = Some(1_000);
        input.additive_stats = BTreeMap::from([
            (
                "hp_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 10,
                },
            ),
            (
                "hp".into(),
                Ratio {
                    numerator: 50,
                    denominator: 1,
                },
            ),
        ]);
        let a4 = project(tables, &input).unwrap();
        input.awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
        let a5 = project(tables, &input).unwrap();
        let repeated = project(tables, &input).unwrap();
        assert_eq!(a4.static_stats.hp, 1_150);
        assert_eq!(a5.static_stats.hp, 1_350);
        assert_eq!(a5.static_stats.base_hp, Some(1_000));
        assert_eq!(
            a5.static_stats.hp_percent,
            Ratio {
                numerator: 3,
                denominator: 10
            }
        );
        assert_eq!(
            a5.static_stats.hp_flat,
            Ratio {
                numerator: 50,
                denominator: 1
            }
        );
        assert_eq!(repeated.static_stats.hp, a5.static_stats.hp);

        input.base_overrides.hp = Some(1_001);
        input.additive_stats = BTreeMap::from([
            (
                "hp_percent".into(),
                Ratio {
                    numerator: 1,
                    denominator: 3,
                },
            ),
            (
                "hp".into(),
                Ratio {
                    numerator: 1,
                    denominator: 3,
                },
            ),
        ]);
        let fractional = project(tables, &input).unwrap();
        assert_eq!(fractional.static_stats.hp, 1_535);
        assert_eq!(
            fractional.static_stats.hp_percent,
            Ratio {
                numerator: 8,
                denominator: 15
            }
        );
        assert_eq!(
            fractional.static_stats.hp_flat,
            Ratio {
                numerator: 1,
                denominator: 3
            }
        );
    }

    #[test]
    fn native_primary_slots_do_not_reinterpret_gameplay_tags_as_ui_categories() {
        let mut request = request();
        request.character_id = "character:1071".into();
        let report = project(ntesim_data::stat_projection::frozen_tables(), &request).unwrap();
        let skill = &report.native_action_metadata["actions"]["GA_Chaos071_Skill"];
        let ultimate = &report.native_action_metadata["actions"]["GA_Chaos071_UltraSkill"];
        assert_eq!(skill["ui_primary_slot"], "skill");
        assert_eq!(ultimate["ui_primary_slot"], "burst");
        assert_eq!(
            skill["ability_metadata"]["gameplay_tag_names"][0]["value"],
            "Ability.Melee"
        );
        assert_eq!(
            ultimate["ability_metadata"]["gameplay_tag_names"][0]["value"],
            "Ability.Melee"
        );
    }

    #[test]
    fn adler_native_build_keeps_base_and_static_def_contributions_separate() {
        let mut input = request();
        input.character_id = "character:1033".into();
        input.base_overrides.def = Some(1000);
        input.additive_stats.clear();
        let base = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(base.static_stats.base_def, Some(1000));
        assert_eq!(base.static_stats.def, 1000);

        input.additive_stats.insert(
            "def_percent".into(),
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        );
        input.additive_stats.insert(
            "def".into(),
            Ratio {
                numerator: 40,
                denominator: 1,
            },
        );
        let built = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(built.static_stats.base_def, Some(1000));
        assert_eq!(built.static_stats.def, 1240);
        assert_eq!(
            built.static_stats.def_percent,
            Ratio {
                numerator: 1,
                denominator: 5,
            }
        );
        assert_eq!(
            built.static_stats.def_flat,
            Ratio {
                numerator: 40,
                denominator: 1,
            }
        );

        input.base_overrides.def = Some(0);
        let zero_base = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(zero_base.static_stats.base_def, Some(0));
        assert_eq!(zero_base.static_stats.def, 40);
    }

    #[test]
    fn general_and_all_six_element_additions_keep_native_identity() {
        let mut input = request();
        for key in [
            "damage_bonus",
            "anima_bonus",
            "chaos_bonus",
            "cosmos_bonus",
            "incantation_bonus",
            "lakshana_bonus",
            "psyche_bonus",
        ] {
            input.additive_stats.insert(
                key.into(),
                Ratio {
                    numerator: 1,
                    denominator: 10,
                },
            );
        }
        let report = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(report.static_stats.elemental_bonus, Ratio::zero());
        assert_eq!(
            report.static_stats.damage_bonus,
            Ratio {
                numerator: 1,
                denominator: 10
            }
        );
        for element in [
            "CHAOS",
            "COSMOS",
            "INCANTATION",
            "LAKSHANA",
            "NATURE",
            "PSYCHE",
        ] {
            assert_eq!(
                report.static_stats.elemental_bonuses[element],
                Ratio {
                    numerator: 1,
                    denominator: 10
                }
            );
        }
        for (_, property) in super::stats::ELEMENT_ATTRIBUTES {
            assert!(!report.unmodeled_attributes.contains_key(property));
        }
    }

    #[test]
    fn quiet_manor_two_piece_native_mental_bonus_is_consumed_with_evidence() {
        let tables = ntesim_data::stat_projection::frozen_tables();
        for count in 0..=3 {
            let mut input = request();
            input.set_id = Some("set:Suit12".into());
            input.set_count = count;
            let report = project(tables, &input).unwrap();
            assert_eq!(
                report.static_stats.mental_damage_bonus.numerator,
                u64::from(count >= 2)
            );
            assert_eq!(
                report.static_stats.mental_damage_bonus.denominator,
                if count >= 2 { 10 } else { 1 }
            );
            assert!(
                !report
                    .unmodeled_attributes
                    .contains_key("DamageUpPsychicallyBase")
            );
            if count >= 2 {
                assert!(
                    report
                        .evidence
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(
                            |entry| entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                                && entry["row"] == "Psychically_2"
                        )
                );
            }
        }
    }

    #[test]
    fn hp_suit_two_piece_native_hp_bonus_is_consumed_with_evidence() {
        let tables = ntesim_data::stat_projection::frozen_tables();
        for count in 0..=3 {
            let mut input = request();
            input.set_id = Some("set:Suit10".into());
            input.set_count = count;
            let report = project(tables, &input).unwrap();
            // Independent oracle: Mag_2 grants HPMaxUp +0.1 at 2 pieces.
            let expected = if count >= 2 { 1 } else { 0 };
            assert_eq!(report.static_stats.hp_percent.numerator, expected as u64);
            assert_eq!(
                report.static_stats.hp_percent.denominator,
                if count >= 2 { 10 } else { 1 }
            );
            if count >= 2 {
                assert!(
                    report
                        .evidence
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(
                            |entry| entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                                && entry["row"] == "Mag_2"
                        )
                );
            }
        }
    }

    #[test]
    fn native_and_manual_healing_bonus_share_the_exact_additive_attribute() {
        let mut input = request();
        input.additive_stats.insert(
            "healing_bonus".into(),
            Ratio {
                numerator: 1,
                denominator: 4,
            },
        );
        let report = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(report.static_stats.healing_bonus.numerator, 1);
        assert_eq!(report.static_stats.healing_bonus.denominator, 4);
        assert!(!report.unmodeled_attributes.contains_key("healing_bonus"));
        assert!(!report.unmodeled_attributes.contains_key("HealUp"));
        for count in 0..=4 {
            input.set_id = Some("set:Suit9".into());
            input.set_count = count;
            let report = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
            let expected = if count == 4 { (9, 20) } else { (1, 4) };
            assert_eq!(report.static_stats.healing_bonus.numerator, expected.0);
            assert_eq!(report.static_stats.healing_bonus.denominator, expected.1);
            assert!(!report.unmodeled_attributes.contains_key("HealUp"));
            let evidence = serde_json::to_value(report.evidence).unwrap();
            if count == 4 {
                assert!(evidence.as_array().unwrap().iter().any(|entry| {
                    entry["source_id"] == ntesim_data::set_modifiers::SOURCE_ID
                        && entry["row"] == "Heal_4"
                }));
            }
        }
    }

    #[test]
    fn native_break_efficiency_rejects_manual_build_additions() {
        let mut input = request();
        let baseline = project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
        assert_eq!(
            baseline.static_stats.break_efficiency_multiplier,
            Ratio::one()
        );
        input.additive_stats.insert(
            "break_efficiency".into(),
            Ratio {
                numerator: 1,
                denominator: 2,
            },
        );
        assert_eq!(
            project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap_err(),
            "unsupported studio additive stat break_efficiency"
        );
    }

    #[test]
    fn manual_mental_and_shield_bonuses_add_to_native_set_attributes() {
        for (stat, property, set_id, threshold, total) in [
            (
                "mental_damage_bonus",
                "DamageUpPsychicallyBase",
                "set:Suit12",
                2,
                (7, 20),
            ),
            ("shield_bonus", "ShieldEfficiency", "set:Suit7", 4, (9, 20)),
        ] {
            for count in 0..=4 {
                let mut input = request();
                input.set_id = Some(set_id.into());
                input.set_count = count;
                input.additive_stats.insert(
                    stat.into(),
                    Ratio {
                        numerator: 1,
                        denominator: 4,
                    },
                );
                let report =
                    project(ntesim_data::stat_projection::frozen_tables(), &input).unwrap();
                let actual = if stat == "mental_damage_bonus" {
                    report.static_stats.mental_damage_bonus
                } else {
                    report.static_stats.shield_bonus
                };
                let expected = if count >= threshold { total } else { (1, 4) };
                assert_eq!(
                    actual,
                    Ratio {
                        numerator: expected.0,
                        denominator: expected.1
                    }
                );
                assert!(!report.unmodeled_attributes.contains_key(property));
                assert_eq!(
                    report.static_stats.elemental_bonuses["PSYCHE"],
                    Ratio::zero()
                );
            }
        }
    }

    #[test]
    fn unknown_stats_invalid_units_and_progression_are_rejected() {
        let mut request = request();
        request.additive_stats.insert(
            "damage_replacement".into(),
            Ratio {
                numerator: 1,
                denominator: 1,
            },
        );
        assert!(
            project(ntesim_data::stat_projection::frozen_tables(), &request)
                .unwrap_err()
                .contains("unsupported")
        );
        request.additive_stats.remove("damage_replacement");
        request.additive_stats.get_mut("atk").unwrap().denominator = 0;
        assert!(project(ntesim_data::stat_projection::frozen_tables(), &request).is_err());
        request.additive_stats.remove("atk");
        request.additive_stats.insert(
            "healing_bonus".into(),
            Ratio {
                numerator: 1,
                denominator: 0,
            },
        );
        assert!(project(ntesim_data::stat_projection::frozen_tables(), &request).is_err());
        request.additive_stats.remove("healing_bonus");
        request.max_level = Some(20);
        assert!(project(ntesim_data::stat_projection::frozen_tables(), &request).is_err());
    }

    #[test]
    fn studio_projection_applies_arachne_hp_in_native_max_hp_percent_domain() {
        let tables = ntesim_data::stat_projection::frozen_tables();
        let make_request = |refinement, additive_stats| Request {
            slot: 0,
            content_id: "ntesim-current".into(),
            character_id: "character:1039".into(),
            level: 80,
            awakening: ntesim_data::AwakeningSelection::default(),
            max_level: Some(80),
            weapon_id: Some("weapon:fork_Arachne".into()),
            weapon_level: Some(80),
            weapon_max_level: Some(80),
            refinement,
            set_id: None,
            set_count: 0,
            base_overrides: BaseOverrides {
                hp: Some(10_000),
                ..BaseOverrides::default()
            },
            additive_stats,
        };

        let mut projected_by_refinement = BTreeMap::new();
        for refinement in [1, 5] {
            let report = project(tables, &make_request(refinement, BTreeMap::new())).unwrap();
            assert!(report.static_stats.hp > 10_000);
            assert!(
                report
                    .model_assumptions
                    .contains_key(crate::weapons::eternal_waltz::HYPOTHESIS_ID.as_str())
            );
            let weapon_evidence = report
                .evidence
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["id"] == "eternal_waltz_static_hp_projection")
                .unwrap();
            let parameter = &weapon_evidence["weapon"]["native_parameter_report"]["parameters"][0];
            assert_eq!(
                parameter["reference_evidence"]["source_id"],
                "table:typed_weapon_refinements"
            );
            assert_eq!(
                parameter["selected"]["evidence"]["source_id"],
                "table:typed_weapon_curves"
            );
            let selected_pointer = parameter["selected"]["evidence"]["pointer"]
                .as_str()
                .unwrap();
            assert!(selected_pointer.contains("/Keys/") && selected_pointer.ends_with("/Value"));
            projected_by_refinement.insert(refinement, report.static_stats.hp);
        }
        assert_eq!(
            projected_by_refinement[&5] - projected_by_refinement[&1],
            1_200
        );
        let additional_hp_percent = BTreeMap::from([(
            "hp_percent".into(),
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        )]);
        let report = project(tables, &make_request(1, additional_hp_percent)).unwrap();
        assert_eq!(report.static_stats.hp - projected_by_refinement[&1], 2_000);
    }
}
