//! Compact source-bound build inputs for the synchronous studio compiler.
//! Growth is compiled by the native constant adapters, never interpolated in JS.
use super::*;
use crate::HypothesisId;

pub const TABLES: [&str; 0] = [];

fn attributes(report: &Value) -> Result<BTreeMap<String, f64>, String> {
    report["attributes"]
        .as_object()
        .ok_or("compiled attributes missing")?
        .iter()
        .map(|(k, v)| Ok((k.clone(), receipt(v)?.to_f64())))
        .collect()
}

fn progression(
    compile: impl Fn(u32, u32) -> Result<Value, String>,
    max_levels: &[u32],
) -> Result<Value, String> {
    let initial = attributes(&compile(1, 6)?)?;
    let mut levels = Vec::new();
    for level in 1..=80 {
        let attrs = attributes(&compile(level, 6)?)?;
        let growth: BTreeMap<_, _> = attrs
            .into_iter()
            .map(|(key, value)| {
                let delta = value - initial.get(&key).copied().unwrap_or(0.0);
                (key, delta)
            })
            .filter(|(_, value)| *value != 0.0)
            .collect();
        levels.push(growth);
    }
    let mut breakthroughs = Vec::new();
    for (breakthrough, max_level) in max_levels.iter().enumerate() {
        let report = compile(1, breakthrough as u32)?;
        breakthroughs.push(json!({"max_level":max_level,"attributes":attributes(&report)?,"evidence":report["evidence"],"model_assumption":report["steps"].as_array().unwrap().iter().find_map(|s| s.get("model_assumption"))}));
    }
    Ok(json!({"levels":levels,"breakthroughs":breakthroughs}))
}

pub fn catalog(tables: &BTreeMap<String, Value>, coverage: &Value) -> Result<Value, String> {
    let package = "ntesim-current";
    let mut characters = BTreeMap::new();
    for entity in coverage["entities"]
        .as_array()
        .ok_or("coverage entities missing")?
    {
        let Some(id) = entity["id"]
            .as_str()
            .filter(|id| id.starts_with("character:"))
        else {
            continue;
        };
        if entity["availability"] != "released" {
            continue;
        }
        let character = ntesim_data::character::definition(id)?;
        let caps = ntesim_data::character::progression::LEVEL_CAPS;
        let mut entry = progression(
            |level, breakthrough| {
                ntesim_data::character::compile(&ntesim_data::character::Request {
                    schema_version: 1,
                    content_id: package.into(),
                    character_id: id.into(),
                    level,
                    breakthrough,
                })
            },
            &caps,
        )?;
        entry["group"] = json!(character.weapon_group.native_key());
        entry["script_alias"] = json!(character.script_alias);
        characters.insert(id, entry);
    }
    let mut weapons = BTreeMap::new();
    let breakthrough_rows = ntesim_data::weapon_breakthrough::rows(tables)?;
    for (native, row) in ntesim_data::weapon_definitions::rows(tables)?.1 {
        if row["ElementData"]["MaxBreakthrough"] != 6 {
            continue;
        }
        let id = format!("weapon:{}", native.to_lowercase());
        let pack = row["ElementData"]["BreakthroughPackId"]
            .as_str()
            .ok_or("weapon breakthrough pack missing")?;
        let caps = (0..=6)
            .map(|b| {
                let id = format!("{pack}_{b}");
                breakthrough_rows
                    .iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case(&id))
                    .ok_or_else(|| format!("weapon breakthrough missing: {id}"))?
                    .1["MaxForkLevel"]
                    .as_u64()
                    .map(|v| v as u32)
                    .ok_or_else(|| "weapon cap missing".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut entry = progression(
            |level, breakthrough| {
                ntesim_data::weapon::compile(
                    tables,
                    &ntesim_data::weapon::Request {
                        schema_version: 1,
                        content_id: package.into(),
                        weapon_id: id.clone(),
                        level,
                        breakthrough,
                        refinement: 1,
                    },
                )
            },
            &caps,
        )?;
        entry["group"] = row["ElementData"]["ApplyGroupType"].clone();
        entry["weapon_id"] = json!(format!("weapon:{native}"));
        entry["max_refinement"] = row["ElementData"]["MaxUpgradeStar"].clone();
        weapons.insert(id, entry);
    }
    let gear = gear_catalog(tables, package, None)?;
    let (actions, damage_effects, native_actions) = action_catalog(tables, coverage, package)?;
    let ability_names = ntesim_data::ability_titles::TITLES
        .iter()
        .map(|row| (row.ability_id, row.title))
        .collect::<BTreeMap<_, _>>();
    Ok(
        json!({"schema_version":1,"content_id":package,"characters":characters,"weapons":weapons,
        "weapon_procs":gear["weapon_procs"],"sets":gear["sets"],
        "set_definitions":ntesim_data::set_definitions::rows(tables)?.1,
        "weapon_definitions":ntesim_data::weapon_definitions::rows(tables)?.1,
        "fireflies_proc":gear["fireflies_proc"],"fireflies_evidence":gear["fireflies_evidence"],
        "speedy_hedgehog_proc":gear["speedy_hedgehog_proc"],"speedy_hedgehog_evidence":gear["speedy_hedgehog_evidence"],
        "actions":actions,"damage_effects":damage_effects,"native_actions":native_actions,"ability_names":ability_names,
        "set_procs":gear["set_procs"],"gear_assumptions":gear["gear_assumptions"],
        "scope":"Native base/growth constants; gear main/substats are supplied by add stats. Set count declares 2/4-piece effects; console stats are authored explicitly."}),
    )
}

/// Shared source descriptors for catalog export and selected equipped builds.
pub(super) fn gear_catalog(
    tables: &BTreeMap<String, Value>,
    package: &str,
    weapon_filter: Option<&str>,
) -> Result<Value, String> {
    let mut weapon_procs = BTreeMap::new();
    for adapter in super::gear::WEAPON_ADAPTERS {
        if weapon_filter.is_some_and(|weapon| !weapon.eq_ignore_ascii_case(adapter.weapon_id)) {
            continue;
        }
        let id = adapter.weapon_id;
        let mut procs = Vec::new();
        for tier in 1..=5 {
            let mut actor: crate::simulation::GameActorConfig = serde_json::from_value(json!({
                "slot":0,"character_id":"character:1019","character_level":80,"awakening":[],
                "purchased_skill_level":8,"initial_hp":1,"initial_hp_max":1,"initial_energy":0,
                "energy_capacity":100,"weapon_id":id,"weapon_tier":tier,"set_id":null,
                "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
                "crit_rate":"0/1","crit_damage_bonus":"0/1"}
            }))
            .map_err(|e| e.to_string())?;
            actor.character_id = adapter.preview_character_id.into();
            adapter.project(tables, 1, package, 20_000_000, &mut actor)?;
            procs.push(actor.gear_procs.remove(0));
        }
        weapon_procs.insert(id.to_ascii_lowercase(), procs);
    }
    let shadow = ntesim_data::set_passive_values::shadow_creed(tables)?;
    let shadow_proc = GearProcConfig::ShadowCreedAtk {
        atk_bonus: ratio(receipt(
            &shadow["parameters"]["Attack2_4_AtkUp"]["selected"],
        )?)?,
        duration_us: integer(
            &shadow["parameters"]["Attack2_4_CD"]["selected"],
            "Shadow Creed duration",
        )?
        .checked_mul(1_000_000)
        .ok_or("Shadow Creed duration overflow")?,
    };
    let (set_proc, fireflies) = crate::sets::fireflies::projection::compile(tables)?;
    let mut sets = BTreeMap::new();
    let speedy = ntesim_data::set_passive_values::speedy_hedgehog(tables)?;
    let speedy_parameters = &speedy["parameters"];
    let speedy_proc = GearProcConfig::SpeedyHedgehogTeamAtk {
        team_atk_bonus: ratio(receipt(
            &speedy_parameters["GetEfficiency2_4_Atk"]["selected"],
        )?)?,
        duration_us: integer(
            &speedy_parameters["GetEfficiency2_4_CD"]["selected"],
            "Speedy Hedgehog duration",
        )?
        .checked_mul(1_000_000)
        .ok_or("Speedy Hedgehog duration overflow")?,
    };
    let lost_radiance = ntesim_data::set_passive_values::lost_radiance(tables)?;
    let lost_radiance_parameters = &lost_radiance["parameters"];
    let lost_radiance_proc = GearProcConfig::LostRadianceDefIgnore {
        def_ignore: ratio(receipt(
            &lost_radiance_parameters["Cosmos2_4_DefIgnore"]["selected"],
        )?)?,
        duration_us: integer(
            &lost_radiance_parameters["Cosmos2_4_CD"]["selected"],
            "Lost Radiance duration",
        )?
        .checked_mul(1_000_000)
        .ok_or("Lost Radiance duration overflow")?,
    };
    let (quiet_proc, quiet_manor) = crate::sets::quiet_manor::projection::compile(tables)?;
    let (hp_suit_proc, hp_suit) = crate::sets::hp_suit::projection::compile(tables)?;
    let (crimson_proc, crimson) = crate::sets::crimson::projection::compile(tables)?;
    let (devils_proc, devils) = crate::sets::devils_blood::projection::compile(tables)?;
    let (diabolos_proc, diabolos) = crate::sets::diabolos::projection::compile(tables)?;
    let (street_boxer_proc, street_boxer) =
        crate::sets::street_boxer::projection::compile(tables, true)?;
    let (set_table, set_rows) = ntesim_data::set_definitions::rows(tables)?;
    for (id, row) in set_rows {
        let mut effects = Vec::new();
        for effect in row["SuitStructList"]
            .as_array()
            .ok_or("set effects missing")?
        {
            let pack_id = effect["SuitModifyPackID"]
                .as_str()
                .ok_or("set pack missing")?;
            let mut attrs = BTreeMap::<String, f64>::new();
            if pack_id != "None" {
                let pack = ntesim_data::set_modifiers::compile(tables, pack_id)?;
                for (prop, value) in pack["attributes"]
                    .as_object()
                    .ok_or("set modifier attributes missing")?
                {
                    attrs.insert(prop.clone(), receipt(value)?.to_f64());
                }
            }
            effects.push(json!({"threshold":effect["SuitCondition"],"attributes":attrs,"has_buff":!effect["SuitBuff"].is_null(),"evidence":{"source_id":format!("table:{set_table}"),"row":id,"modifier_row":pack_id}}));
        }
        sets.insert(format!("set:{id}"), effects);
    }
    let set_procs = json!({
        "set:Suit8":{"threshold":4,"procs":[shadow_proc],"evidence":shadow},
        "set:Suit2":{"threshold":4,"procs":[set_proc],"evidence":fireflies},
        "set:Suit11":{"threshold":4,"procs":[speedy_proc],"evidence":speedy},
        "set:Suit6":{"threshold":4,"procs":[lost_radiance_proc],"evidence":lost_radiance},
        "set:Suit12":{"threshold":4,"procs":[quiet_proc],"evidence":quiet_manor},
        "set:Suit10":{"threshold":4,"procs":[hp_suit_proc],"evidence":hp_suit},
        "set:Suit4":{"threshold":4,"procs":[crimson_proc],"evidence":crimson},
        "set:Suit3":{"threshold":4,"procs":[devils_proc],"evidence":devils},
        "set:Suit1":{"threshold":4,"procs":[diabolos_proc],"evidence":diabolos},
        "set:Suit5":{"threshold":4,"procs":[street_boxer_proc],"evidence":street_boxer}
    });
    let gear_assumptions = gear_assumptions();
    Ok(
        json!({"weapon_procs":weapon_procs,"sets":sets,"set_procs":set_procs,
        "gear_assumptions":gear_assumptions,"fireflies_proc":set_proc,"fireflies_evidence":fireflies,
        "speedy_hedgehog_proc":speedy_proc,"speedy_hedgehog_evidence":speedy}),
    )
}

fn base_gear_assumptions() -> Value {
    json!({
          "devils_blood_damage":{(crate::sets::devils_blood::HYPOTHESIS_ID.as_str()):crate::sets::devils_blood::HYPOTHESIS_ID.description()},
          "diabolos_chaos_ignore":{(crate::sets::diabolos::HYPOTHESIS_ID.as_str()):crate::sets::diabolos::HYPOTHESIS_ID.description()},
          "street_boxer_crit":{(crate::sets::street_boxer::HYPOTHESIS_ID.as_str()):crate::sets::street_boxer::HYPOTHESIS_ID.description()},
          "quiet_manor_mental":{(crate::sets::quiet_manor::HYPOTHESIS_ID.as_str()):crate::sets::quiet_manor::HYPOTHESIS_ID.description()},
          "hp_suit_max_hp":{(crate::sets::hp_suit::HYPOTHESIS_ID.as_str()):crate::sets::hp_suit::HYPOTHESIS_ID.description()},
          "crimson_atk":{(crate::sets::crimson::HYPOTHESIS_ID.as_str()):crate::sets::crimson::HYPOTHESIS_ID.description()},
          "castle_healing_bonus":{
            (crate::weapons::castle::HYPOTHESIS_ID.as_str()):crate::weapons::castle::HYPOTHESIS_ID.description(),
            (crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()):crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
          },
          "noros_atk":{crate::weapons::noros::HYPOTHESIS_ID.as_str():crate::weapons::noros::HYPOTHESIS_ID.description()},
          "gold_wool":{crate::weapons::gold_wool::HYPOTHESIS_ID.as_str():crate::weapons::gold_wool::HYPOTHESIS_ID.description()},
          "motor_candy_atk":{crate::weapons::motor_candy::HYPOTHESIS_ID.as_str():crate::weapons::motor_candy::HYPOTHESIS_ID.description()},
          "knight_candy_crit_damage":{crate::weapons::knight_candy::HYPOTHESIS_ID.as_str():crate::weapons::knight_candy::HYPOTHESIS_ID.description()},
          "boxing_candy_damage":{crate::weapons::boxing_candy::HYPOTHESIS_ID.as_str():crate::weapons::boxing_candy::HYPOTHESIS_ID.description()},
          "appliance_damage":{crate::weapons::appliance::HYPOTHESIS_ID.as_str():crate::weapons::appliance::HYPOTHESIS_ID.description()},
          "prokaryon_damage":{crate::weapons::prokaryon::HYPOTHESIS_ID.as_str():crate::weapons::prokaryon::HYPOTHESIS_ID.description()},
          "paper_plane_damage":{crate::weapons::paper_plane::HYPOTHESIS_ID.as_str():crate::weapons::paper_plane::HYPOTHESIS_ID.description()},
          "worldrain_damage":{crate::weapons::worldrain::HYPOTHESIS_ID.as_str():crate::weapons::worldrain::HYPOTHESIS_ID.description()},
          "shadow_creed_atk":{crate::sets::shadow_creed::HYPOTHESIS_ID.as_str():crate::sets::shadow_creed::HYPOTHESIS_ID.description(), crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID.as_str():crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID.description()},
          "cast_buff_order":{crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str():crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()},
          "blast_candy_atk":{crate::weapons::blast_candy::HYPOTHESIS_ID.as_str():crate::weapons::blast_candy::HYPOTHESIS_ID.description()},
          "mind_royale_energy":{HypothesisId::WeaponsMindRoyaleV1.as_str():HypothesisId::WeaponsMindRoyaleV1.description()},
          "good_boy_team_atk":{HypothesisId::WeaponsGoodBoyV1.as_str():HypothesisId::WeaponsGoodBoyV1.description()},
          "fireflies_crit_damage":{
            HypothesisId::FirefliesProcV1.as_str():HypothesisId::FirefliesProcV1.description(),
            (crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.as_str()):crate::sets::fireflies::hit::TEAM_HIT_HYPOTHESIS_ID.description(),
            (crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.as_str()):crate::sets::fireflies::hit::ALL_IN_RANGE_HYPOTHESIS_ID.description()
          },
          "speedy_hedgehog_team_atk":{HypothesisId::SpeedyHedgehogProcV1.as_str():HypothesisId::SpeedyHedgehogProcV1.description()},
          "lost_radiance_def_ignore":{
            (crate::sets::lost_radiance::HYPOTHESIS_ID.as_str()):crate::sets::lost_radiance::HYPOTHESIS_ID.description(),
            (crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "twinbirds":{
            (crate::weapons::twinbirds::HYPOTHESIS_ID.as_str()):crate::weapons::twinbirds::HYPOTHESIS_ID.description(),
            (crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "eternal_waltz":{
            (crate::weapons::eternal_waltz::HYPOTHESIS_ID.as_str()):crate::weapons::eternal_waltz::HYPOTHESIS_ID.description(),
            (crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "raging_flames":{
            (crate::weapons::raging_flames::HYPOTHESIS_ID.as_str()):crate::weapons::raging_flames::HYPOTHESIS_ID.description(),
            (crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::CAST_BUFF_HYPOTHESIS_ID.description(),
            (crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "lunar_phase":{
            (crate::weapons::lunarphase::HYPOTHESIS_ID.as_str()):crate::weapons::lunarphase::HYPOTHESIS_ID.description(),
            (crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "demon_blade":{
            (crate::weapons::demon_blade::HYPOTHESIS_ID.as_str()):crate::weapons::demon_blade::HYPOTHESIS_ID.description()
          },
          "moon":{
            (crate::weapons::moon::HYPOTHESIS_ID.as_str()):crate::weapons::moon::HYPOTHESIS_ID.description()
          },
          "far_side_moon":{
            (crate::weapons::far_side_moon::HYPOTHESIS_ID.as_str()):crate::weapons::far_side_moon::HYPOTHESIS_ID.description(),
            (crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::SKILL_CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "oraora":{
            (crate::weapons::oraora::HYPOTHESIS_ID.as_str()):crate::weapons::oraora::HYPOTHESIS_ID.description()
          },
          "forgotten_hp_stats":{
            (crate::weapons::forgotten::HYPOTHESIS_ID.as_str()):crate::weapons::forgotten::HYPOTHESIS_ID.description()
          },
          "bopu_damage":{
            (crate::weapons::bopu::HYPOTHESIS_ID.as_str()):crate::weapons::bopu::HYPOTHESIS_ID.description(),
            (crate::gear::QTE_CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::QTE_CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "koinobori_stats":{
            (crate::weapons::koinobori::HYPOTHESIS_ID.as_str()):crate::weapons::koinobori::HYPOTHESIS_ID.description()
          },
          "gold_record":{
            (crate::weapons::gold_record::HYPOTHESIS_ID.as_str()):crate::weapons::gold_record::HYPOTHESIS_ID.description()
          },
          "spider_team_atk":{
            (crate::weapons::spider::HYPOTHESIS_ID.as_str()):crate::weapons::spider::HYPOTHESIS_ID.description(),
            (crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()):crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
          },
          "bit_game":{
            (crate::weapons::bit_game::HYPOTHESIS_ID.as_str()):crate::weapons::bit_game::HYPOTHESIS_ID.description()
          },
          "bitter_cake":{
            (crate::weapons::bittercake::HYPOTHESIS_ID.as_str()):crate::weapons::bittercake::HYPOTHESIS_ID.description()
          }
    })
}

pub(crate) const BATCH_TWO_FOUR_WEAPON_ASSUMPTIONS: &[(&str, HypothesisId)] = &[
    ("jiaojuan_break", crate::weapons::jiaojuan::HYPOTHESIS_ID),
    ("snowman_shield_atk", crate::weapons::snowman::HYPOTHESIS_ID),
    ("yuren_hp", crate::weapons::yuren::HYPOTHESIS_ID),
    (
        "thief_candy_dodge",
        crate::weapons::thief_candy::HYPOTHESIS_ID,
    ),
    ("dustbin_break", crate::weapons::dustbin::HYPOTHESIS_ID),
    ("police_rat_boss", crate::weapons::police_rat::HYPOTHESIS_ID),
    (
        "tiger_tally_atk",
        crate::weapons::tiger_tally::HYPOTHESIS_ID,
    ),
    ("nakupeda", crate::weapons::nakupeda::HYPOTHESIS_ID),
    (
        "black_book_break",
        crate::weapons::black_book::HYPOTHESIS_ID,
    ),
    (
        "jingmotingyuan_crit",
        crate::weapons::jingmotingyuan::HYPOTHESIS_ID,
    ),
    ("rishi_atk", crate::weapons::rishi::HYPOTHESIS_ID),
    (
        "tuansanlang_team",
        crate::weapons::tuansanlang::HYPOTHESIS_ID,
    ),
    ("yaodao_parry", crate::weapons::yaodao::HYPOTHESIS_ID),
    ("rose_thorn", crate::weapons::rose::HYPOTHESIS_ID),
    ("mamen_fons", crate::weapons::mamen::HYPOTHESIS_ID),
];

fn gear_assumptions() -> Value {
    let mut gear_assumptions = base_gear_assumptions();
    for &(kind, id) in &[
        ("butterfly_damage", crate::weapons::butterfly::HYPOTHESIS_ID),
        ("kite_atk", crate::weapons::kite::HYPOTHESIS_ID),
        ("whale_atk", crate::weapons::whale::HYPOTHESIS_ID),
        ("crowbar_break", crate::weapons::crowbar::HYPOTHESIS_ID),
        ("nest_bird_gaze", crate::weapons::nest_bird::HYPOTHESIS_ID),
    ] {
        gear_assumptions[kind] = json!({(id.as_str()): id.description()});
    }
    for &(kind, id) in BATCH_TWO_FOUR_WEAPON_ASSUMPTIONS {
        gear_assumptions[kind] = json!({(id.as_str()): id.description()});
    }
    gear_assumptions
}

/// Native membership describes candidates, not packet reachability or caller timing.
/// Multiple candidates require an explicit effect selection at the input boundary.
fn action_catalog(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    content_id: &str,
) -> Result<(Value, Value, Value), String> {
    let (config_table, configs) = ntesim_data::character_actions::rows(tables)?;
    let (_, damage) = ntesim_data::native_damage::rows(tables)?;
    let mut actions = BTreeMap::new();
    let mut effects = BTreeMap::new();
    let mut native_actions = BTreeMap::new();
    for owner in coverage["entities"]
        .as_array()
        .ok_or("coverage entities missing")?
    {
        let Some(id) = owner["id"]
            .as_str()
            .filter(|id| id.starts_with("character:"))
        else {
            continue;
        };
        if owner["availability"] != "released" {
            continue;
        }
        let native = id.strip_prefix("character:").unwrap();
        let configured = configs
            .get(native)
            .and_then(|v| v["CharacterAbilityList"].as_array())
            .ok_or("configured character abilities missing")?;
        let ability_ids = configured
            .iter()
            .map(|v| v["Key"].as_str().ok_or("configured ability ID missing"))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let owned = damage
            .iter()
            .filter(|(_, row)| {
                row["GAName"]
                    .as_str()
                    .is_some_and(|ga| ability_ids.contains(ga))
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let mut descriptors = BTreeMap::new();
        let mut owner_native_actions = BTreeMap::new();
        let mut primaries = BTreeMap::<&'static str, (String, usize)>::new();
        for (index, configured_action) in configured.iter().enumerate() {
            let ability = configured_action["Key"]
                .as_str()
                .ok_or("configured ability ID missing")?;
            let primary_index = configured_action["Value"]["AbilityIndex"].as_u64();
            let role = ntesim_data::native_action_metadata::resolve_primary_role(primary_index)?;
            let slot = match role {
                ntesim_data::native_action_metadata::NativePrimaryRole::Normal => Some("attack"),
                ntesim_data::native_action_metadata::NativePrimaryRole::RedirectSkill => {
                    Some("skill")
                }
                ntesim_data::native_action_metadata::NativePrimaryRole::Ultimate => Some("burst"),
                ntesim_data::native_action_metadata::NativePrimaryRole::Qte => None,
            };
            if let Some(slot) = slot.filter(|slot| {
                primaries
                    .insert(*slot, (ability.to_owned(), index))
                    .is_some()
            }) {
                return Err(format!(
                    "{id} has duplicate native primary input role {slot}"
                ));
            }
            let ability_metadata = ntesim_data::native_action_metadata::resolve_ability(
                content_id, tables, id, ability,
            )?;
            let mut effect_metadata = BTreeMap::new();
            for (effect_id, row) in damage.iter().filter(|(_, row)| {
                row["GAName"]
                    .as_str()
                    .is_some_and(|ga| ga.eq_ignore_ascii_case(ability))
            }) {
                effect_metadata.insert(
                    effect_id.clone(),
                    json!({
                        "native_metadata":ntesim_data::native_action_metadata::resolve_damage_effect(content_id, tables, effect_id)?,
                        "native_row":row,
                    }),
                );
            }
            owner_native_actions.insert(
                ability.to_owned(),
                json!({
                    "native_ability_index":primary_index,
                    "ability_metadata":ability_metadata,
                    "damage_effects":effect_metadata,
                }),
            );
        }
        for kind in ["attack", "skill", "burst"] {
            let (ability, config_index) = primaries
                .get(kind)
                .cloned()
                .ok_or_else(|| format!("{id} lacks configured native primary input role {kind}"))?;
            let candidates = damage
                .iter()
                .filter(|(_, row)| {
                    row["GAName"]
                        .as_str()
                        .is_some_and(|ga| ga.eq_ignore_ascii_case(&ability))
                })
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            let native_metadata = owner_native_actions
                .get(&ability)
                .ok_or("native primary action metadata missing")?;
            let mut descriptor = json!({"ability_id":ability,"damage_effect_ids":candidates,
                "native_ability_index":native_metadata["native_ability_index"],
                "evidence":{"source_id":format!("table:{config_table}"),"row":native,
                    "pointer":format!("/rows/{native}/CharacterAbilityList/{config_index}/Value/AbilityIndex")},
                "native_metadata":native_metadata,
                "primary_role_hypothesis_id":ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
                "primary_role_hypothesis":ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_DESCRIPTION,
                "scope":"UI aliases use the shared native primary-role interpretation. Native GameplayTag and DamageTypeEX remain separate source metadata; effect membership follows GAName only. Reachability, timing and cost require explicit modeled input."});
            if candidates.len() == 1 {
                descriptor["damage_effect_id"] = json!(candidates[0]);
            }
            let primary_packet = crate::registry::primary_packet(id, &ability);
            if let Some(packet) = primary_packet {
                if !candidates.iter().any(|candidate| candidate == packet) {
                    return Err(format!(
                        "{id} primary packet {packet} is not owned by {ability}"
                    ));
                }
                descriptor["public_primary_packet"] = json!(packet);
            }
            descriptors.insert(kind, descriptor);
        }
        actions.insert(id, descriptors);
        effects.insert(id, owned);
        native_actions.insert(id, owner_native_actions);
    }
    Ok((json!(actions), json!(effects), json!(native_actions)))
}

#[cfg(test)]
mod action_catalog_tests {
    use super::*;
    #[test]
    fn an_additional_owner_uses_membership_without_registry_changes() {
        let wrap = |name: &str, rows: Value| json!({"schema_version":1,"source_id":format!("table:{name}"),"rows":rows});
        let tables = BTreeMap::from([
            (
                "DT_CharacterAbilityConfig".into(),
                wrap(
                    "DT_CharacterAbilityConfig",
                    json!({"9999":{"CharacterAbilityList":[
                        {"Key":"GA_Primary_A","Value":{"AbilityIndex":1}},
                        {"Key":"GA_Primary_B","Value":{"AbilityIndex":2}},
                        {"Key":"GA_Primary_C","Value":{"AbilityIndex":3}},
                    ]}}),
                ),
            ),
            (
                "DT_CharacterAbilityEffectConfig".into(),
                wrap(
                    "DT_CharacterAbilityEffectConfig",
                    json!({
                        "GA_Primary_A":{"AbilityGameplayTag":{"TagName":"Ability.Melee"}},
                        "GA_Primary_B":{"AbilityGameplayTag":{"TagName":"Ability.Melee"}},
                        "GA_Primary_C":{"AbilityGameplayTag":{"TagName":"Ability.Melee"}},
                    }),
                ),
            ),
            (
                "DT_SkillDamageData".into(),
                wrap(
                    "DT_SkillDamageData",
                    json!({
                        "GE_First":{"GAName":"GA_Primary_A","DamageTypeEX":"native:TYPE_A","DamageSourceCategory":"native:SOURCE_A"},
                        "GE_Second":{"GAName":"GA_Primary_B","DamageTypeEX":"native:TYPE_B","DamageSourceCategory":"native:SOURCE_B"},
                        "GE_SecondChild":{"GAName":"GA_Primary_B","DamageTypeEX":"native:TYPE_CHILD"},
                        "GE_Third":{"GAName":"GA_Primary_C","DamageTypeEX":"native:TYPE_C","DamageSourceCategory":"native:SOURCE_C"},
                        "GE_Other":{"GAName":"GA_Other_Skill"},
                    }),
                ),
            ),
        ]);
        let (actions, effects, native_actions) = action_catalog(
            &tables,
            &json!({"entities":[{"id":"character:9999","availability":"released"}]}),
            "ntesim-current",
        )
        .unwrap();
        assert_eq!(
            actions["character:9999"]["attack"]["damage_effect_id"],
            "GE_First"
        );
        assert_eq!(
            actions["character:9999"]["skill"]["ability_id"],
            "GA_Primary_B"
        );
        assert_eq!(
            actions["character:9999"]["burst"]["ability_id"],
            "GA_Primary_C"
        );
        assert!(actions["character:9999"]["skill"]["damage_effect_id"].is_null());
        assert_eq!(
            actions["character:9999"]["skill"]["native_metadata"]["ability_metadata"]["gameplay_tag_names"]
                [0]["value"],
            "Ability.Melee"
        );
        assert_eq!(
            native_actions["character:9999"]["GA_Primary_B"]["damage_effects"]["GE_Second"]["native_metadata"]
                ["damage_type_ex"],
            "native:TYPE_B"
        );
        assert_eq!(effects["character:9999"].as_array().unwrap().len(), 4);
    }
}
