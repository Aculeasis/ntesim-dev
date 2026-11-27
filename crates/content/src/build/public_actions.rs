//! Existing bounded authoring policy, shared by native projection and WASM.
//! These values are research inputs, not recovered native action timing.
use super::*;
use crate::HypothesisId;

pub(crate) const TIMING_ID: HypothesisId = HypothesisId::ActionsCallerTimingV1;

/// These owners observe state before their first explicit source input,
/// including when they begin off field.
pub(super) fn initial_assumptions(character: &str) -> BTreeMap<String, String> {
    let models: &[HypothesisId] = match character {
        "character:1033" => &[crate::characters::adler::righteous_heart::HYPOTHESIS_ID],
        "character:1070" => &[
            crate::characters::aurelia::harmonics::HYPOTHESIS_ID,
            crate::characters::aurelia::cadenza::A2_HYPOTHESIS_ID,
            crate::characters::aurelia::cadenza::R6_HYPOTHESIS_ID,
        ],
        _ => &[],
    };
    let mut assumptions = match character {
        "character:1004" => BTreeMap::from([(
            crate::characters::lacrimosa::sleep_in::MODEL.into(),
            crate::characters::lacrimosa::sleep_in::MODEL
                .description()
                .into(),
        )]),
        "character:1025" => crate::characters::hathor::source_assumptions(),
        "character:1042" => crate::characters::blackbird::source::assumptions(),
        "character:1054" => crate::characters::daffodil::source_assumptions(),
        "character:1036" => crate::characters::zankou::source_assumptions(),
        "character:1003" => crate::characters::sakiri::source_assumptions(),
        "character:1020" => crate::characters::haniel::source_assumptions(),
        "character:1057" => crate::characters::akane::source_assumptions(),
        "character:1090" => crate::characters::elyms::source_assumptions(),
        "character:1071" => crate::characters::chaos::source_assumptions(),
        "character:1075" => crate::characters::iroi::source_assumptions(),
        _ => BTreeMap::new(),
    };
    assumptions.extend(
        models
            .iter()
            .map(|&id| (id.into(), id.description().into())),
    );
    assumptions
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicAction {
    pub command: GameActionCommand,
    pub advance_us: u64,
    pub model_assumptions: BTreeMap<String, String>,
}

pub(super) fn arc(id: HypothesisId) -> PublicAction {
    PublicAction {
        command: GameActionCommand::PlayerInput {
            timestamp_us: 0,
            actor_index: 0,
            input: crate::simulation::PlayerInput::Arc,
            critical_success: false,
        },
        advance_us: 0,
        model_assumptions: BTreeMap::from([(id.into(), id.description().into())]),
    }
}

fn entry_assumptions(character: &str) -> BTreeMap<String, String> {
    use crate::characters::{baicang, chiz};
    let models: &[HypothesisId] = match character {
        "character:1023" => &[
            baicang::etd::HYPOTHESIS_ID,
            baicang::support::HYPOTHESIS_ID,
            baicang::words::HYPOTHESIS_ID,
        ],
        "character:1073" => &[
            chiz::clock::MODEL,
            chiz::pressed::MODEL,
            chiz::finance::MODEL,
            chiz::cancel::MODEL,
            chiz::secondary::MODEL,
        ],
        _ => &[],
    };
    models
        .iter()
        .copied()
        .map(|id| (id.into(), id.description().into()))
        .collect()
}

fn assumptions(character: &str, kind: &str) -> BTreeMap<String, String> {
    use crate::characters::packet_semantics as semantics;
    let mut values = BTreeMap::from([
        (TIMING_ID.as_str().into(), TIMING_ID.description().into()),
        (
            HypothesisId::ActionsPrimaryPacketV1.as_str().into(),
            HypothesisId::ActionsPrimaryPacketV1.description().into(),
        ),
    ]);
    let extra: &[(HypothesisId, &str)] = match (character, kind) {
        ("character:1073", "skill") => &[
            (
                crate::characters::chiz::pressed::held::MODEL,
                crate::characters::chiz::pressed::held::MODEL.description(),
            ),
            (
                crate::characters::chiz::pressed::held::finish::MODEL,
                crate::characters::chiz::pressed::held::finish::MODEL.description(),
            ),
        ],
        ("character:1008", "skill") => &[(
            crate::characters::skia::entry_state::HYPOTHESIS_ID,
            crate::characters::skia::entry_state::HYPOTHESIS_ID.description(),
        )],
        ("character:1072", "skill") => &[(
            crate::characters::linko::skill::PET_HYPOTHESIS_ID,
            crate::characters::linko::skill::PET_HYPOTHESIS_ID.description(),
        )],
        ("character:1070", "attack") => &[
            (
                HypothesisId::AureliaCadenzaPowerSelectV1,
                HypothesisId::AureliaCadenzaPowerSelectV1.description(),
            ),
            (
                crate::characters::aurelia::crescendo::HYPOTHESIS_ID,
                crate::characters::aurelia::crescendo::HYPOTHESIS_ID.description(),
            ),
        ],
        ("character:1010", "attack" | "burst") => &[(
            crate::characters::nanally::clock::MODEL,
            crate::characters::nanally::clock::MODEL.description(),
        )],
        ("character:1010", "skill") => &[
            (
                crate::characters::nanally::removal::MODEL,
                crate::characters::nanally::removal::MODEL.description(),
            ),
            (
                crate::characters::nanally::clock::MODEL,
                crate::characters::nanally::clock::MODEL.description(),
            ),
        ],
        ("character:1075", "attack" | "skill") => &[(
            crate::characters::iroi::clock::MODEL,
            crate::characters::iroi::clock::MODEL.description(),
        )],
        ("character:1075", "burst") => &[
            (
                crate::characters::iroi::clock::MODEL,
                crate::characters::iroi::clock::MODEL.description(),
            ),
            (
                crate::characters::iroi::dream::HYPOTHESIS_ID,
                crate::characters::iroi::dream::HYPOTHESIS_ID.description(),
            ),
        ],
        ("character:1039", "skill") => &[(
            crate::characters::fadia::effects::MODEL,
            crate::characters::fadia::effects::MODEL.description(),
        )],
        ("character:1055", "attack" | "skill") => &[(
            crate::characters::jiuyuan::pact::HYPOTHESIS_ID,
            crate::characters::jiuyuan::pact::HYPOTHESIS_ID.description(),
        )],
        ("character:1004", "skill" | "burst") => &[
            (
                crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID,
                crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID.description(),
            ),
            (
                crate::characters::lacrimosa::dot::ALL_TARGETS_HYPOTHESIS_ID,
                crate::characters::lacrimosa::dot::ALL_TARGETS_HYPOTHESIS_ID.description(),
            ),
        ],
        ("character:1033", "skill") => &[
            (
                semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID,
                semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.description(),
            ),
            (
                HypothesisId::AdlerBlessingPartyInterceptionV1,
                HypothesisId::AdlerBlessingPartyInterceptionV1.description(),
            ),
        ],
        ("character:1033", "burst") => &[(
            HypothesisId::AdlerBurstIntervalsV1,
            HypothesisId::AdlerBurstIntervalsV1.description(),
        )],
        ("character:1036", "skill" | "attack") => &[
            (
                semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID,
                semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID.description(),
            ),
            (
                crate::characters::zankou::skill::HEARTWRENCH_HYPOTHESIS_ID,
                crate::characters::zankou::skill::HEARTWRENCH_HYPOTHESIS_ID.description(),
            ),
        ],
        ("character:1036", "burst") => &[(
            semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID,
            semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID.description(),
        )],
        ("character:1023", "burst") => &[
            (
                semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID,
                semantics::BAICANG_DOT_PACKET_HYPOTHESIS_ID.description(),
            ),
            (
                crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID,
                crate::characters::baicang::skill::DOMAIN_HYPOTHESIS_ID.description(),
            ),
        ],
        _ => &[],
    };
    let financial = character == "character:1073" && matches!(kind, "skill" | "burst");
    if !extra.is_empty() || financial {
        values.insert(
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS.into(),
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_DESCRIPTION.into(),
        );
    }
    values.extend(
        extra
            .iter()
            .map(|(id, text)| ((*id).into(), (*text).into())),
    );
    if financial {
        values.extend(crate::characters::chiz::financial_source_assumptions());
    }
    values
}

pub(super) fn critical_policy(
    character: &str,
    slot: usize,
) -> Result<crate::simulation::ResearchCritPolicy, String> {
    Ok(crate::simulation::ResearchCritPolicy {
        hypothesis: "Explicit script hypothesis: independent critical rolls per target hit; native eligibility and shared-roll rules are unresolved.".into(),
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        actor_sequences: BTreeMap::from([(character.into(), (slot as u64).checked_add(54).ok_or("critical stream identity overflow")?)]),
        eligible_effects: BTreeMap::from([(character.into(), BTreeSet::from(["*".into()]))]),
    })
}

pub(super) fn project(
    character: &str,
    metadata: &Value,
) -> Result<BTreeMap<String, PublicAction>, String> {
    let mut actions = BTreeMap::new();
    for (ability, descriptor) in metadata["actions"]
        .as_object()
        .ok_or("native action metadata missing")?
    {
        let Some(kind) = descriptor["ui_primary_slot"].as_str() else {
            continue;
        };
        let Some(packet) = crate::registry::primary_packet(character, ability) else {
            continue;
        };
        if descriptor["damage_effects"].get(packet).is_none() {
            return Err(format!(
                "{character} primary packet {packet} is not owned by {ability}"
            ));
        }
        let (mut recovery_us, mut cooldown_us, mut energy_cost) = match kind {
            "attack" => (800_000, 0, 0),
            "skill" => (1_200_000, 6_000_000, 0),
            "burst" => (
                crate::characters::cinematics::FALLBACK_ACTION_RECOVERY_US,
                20_000_000,
                ntesim_data::character::definition(character)
                    .map(|d| d.base_energy() as u64)
                    .unwrap_or(100),
            ),
            _ => return Err(format!("unsupported native primary input role {kind}")),
        };
        let mut hit_delay_us = 0;
        if character == "character:1004" && kind == "skill" {
            cooldown_us = crate::characters::lacrimosa::sleep_in::COOLDOWN_US;
        }
        if character == "character:1023" && kind == "skill" {
            cooldown_us = 14_000_000;
        }
        if character == "character:1010" && kind == "skill" {
            (recovery_us, cooldown_us, hit_delay_us) =
                crate::characters::nanally::clock::skill_timing();
        }
        if character == "character:1010" && kind == "burst" {
            (recovery_us, cooldown_us, hit_delay_us) =
                crate::characters::nanally::clock::burst_timing();
        }
        if character == "character:1075" && kind == "attack" {
            recovery_us = 955_811;
        }
        if character == "character:1075" && kind == "burst" {
            recovery_us = 4_015_812;
            cooldown_us = crate::characters::iroi::skill::BURST_COOLDOWN_US;
        }
        if character == "character:1075" && kind == "skill" {
            recovery_us = 1_449_905;
            cooldown_us = 12_000_000;
        }
        if character == "character:1036" && kind == "skill" {
            (recovery_us, cooldown_us, hit_delay_us) =
                crate::characters::zankou::skill::public_skill_timing();
        }
        if character == "character:1008" && kind == "skill" {
            (recovery_us, cooldown_us, hit_delay_us) =
                crate::characters::skia::entry_state::public_skill_timing();
        }
        if character == "character:1052" {
            match kind {
                "skill" => {
                    (recovery_us, cooldown_us, hit_delay_us) =
                        crate::characters::hotori::skill::public_skill_timing();
                }
                "burst" => {
                    (recovery_us, cooldown_us, hit_delay_us) =
                        crate::characters::hotori::skill::public_burst_timing();
                    energy_cost = 0;
                }
                _ => {}
            }
        }
        if character == "character:1076" {
            match kind {
                "attack" => {
                    recovery_us =
                        crate::characters::shinku::parameters::PARAMETERS.normal_recoveries_us[0];
                }
                "skill" => {
                    recovery_us =
                        crate::characters::shinku::parameters::PARAMETERS.skill_recovery_us;
                    cooldown_us =
                        crate::characters::shinku::parameters::PARAMETERS.skill_cooldown_us;
                }
                "burst" => {
                    recovery_us =
                        crate::characters::shinku::parameters::PARAMETERS.burst_default_recovery_us;
                }
                _ => {}
            }
        }
        if character == "character:1057" {
            match kind {
                "skill" => {
                    cooldown_us =
                        crate::characters::akane::parameters::PARAMETERS.skill_cooldown_us;
                    recovery_us = 1_498_994;
                    hit_delay_us = 345_126;
                }
                "burst" => {
                    cooldown_us =
                        crate::characters::akane::parameters::PARAMETERS.burst_cooldown_us;
                    recovery_us = 4_115_238;
                    hit_delay_us = 2_856_699;
                }
                _ => {}
            }
        }
        if character == "character:1090" {
            match kind {
                "attack" => {
                    recovery_us = crate::characters::elyms::clock::profile::NORMAL_RECOVERY[0];
                    hit_delay_us = crate::characters::elyms::clock::profile::MELEE1_HITS[0].1;
                }
                "skill" => {
                    cooldown_us =
                        crate::characters::elyms::parameters::PARAMETERS.skill_cooldown_us;
                    recovery_us = crate::characters::elyms::clock::profile::SKILL1_RECOVERY;
                    hit_delay_us = crate::characters::elyms::clock::profile::SKILL1_HITS[0].1;
                }
                "burst" => {
                    cooldown_us =
                        crate::characters::elyms::parameters::PARAMETERS.burst_cooldown_us;
                    recovery_us = crate::characters::elyms::clock::profile::BURSTA_RECOVERY;
                    hit_delay_us = crate::characters::elyms::clock::profile::BURSTA_HITS[0].1;
                }
                _ => {}
            }
        }
        actions.insert(
            kind.into(),
            PublicAction {
                command: GameActionCommand::InvokeAbility {
                    timestamp_us: 0,
                    actor_index: 0,
                    ability_id: ability.clone(),
                    array_index: 0,
                    damage_effect_id: packet.into(),
                    atk_rate: Ratio::one(),
                    scaling_stat: Default::default(),
                    energy_cost,
                    recovery_us,
                    cooldown_us,
                    hit_delay_us,
                },
                advance_us: recovery_us,
                model_assumptions: assumptions(character, kind),
            },
        );
    }
    if character == "character:1023" {
        actions.insert(
            "plunge".into(),
            crate::characters::baicang::plunge::public_action(),
        );
    }
    if character == "character:1057" {
        actions.insert(
            "plunge".into(),
            crate::characters::akane::secondary::public_plunge(),
        );
    }
    if character == "character:1090" {
        actions.insert(
            "plunge".into(),
            crate::characters::elyms::secondary::public_plunge(),
        );
    }
    if character == "character:1071" {
        actions.insert(
            "plunge".into(),
            crate::characters::chaos::secondary::public_plunge(),
        );
    }
    if character == "character:1010" {
        actions.insert(
            "plunge".into(),
            crate::characters::nanally::plunge::public_action(),
        );
    }
    if let Some(normal) = actions.get("attack")
        && let Some(plunge) = crate::characters::source_choices::plunge(character, normal)?
    {
        actions.insert("plunge".into(), plunge);
    }
    for (kind, input, advance_us) in [
        ("special", crate::simulation::PlayerInput::Special, 800_000),
        ("dash", crate::simulation::PlayerInput::Dash, 500_000),
        ("jump", crate::simulation::PlayerInput::Jump, 0),
    ] {
        if crate::characters::player_inputs::validate(character, input).is_ok() {
            actions.insert(
                kind.into(),
                PublicAction {
                    command: GameActionCommand::PlayerInput {
                        timestamp_us: 0,
                        actor_index: 0,
                        input,
                        critical_success: false,
                    },
                    advance_us,
                    model_assumptions: if character == "character:1004" && kind == "dash" {
                        [
                            crate::characters::lacrimosa::bat_exit::HYPOTHESIS_ID,
                            crate::characters::baicang::entry_jump::HYPOTHESIS_ID,
                            crate::characters::baicang::support::HYPOTHESIS_ID,
                        ]
                        .into_iter()
                        .map(|id| (id.into(), id.description().into()))
                        .collect()
                    } else {
                        BTreeMap::new()
                    },
                },
            );
        }
    }
    actions.insert(
        "swap".into(),
        PublicAction {
            command: GameActionCommand::SwitchActor {
                timestamp_us: 0,
                target_actor_index: 0,
                cooldown_us: 250_000,
            },
            advance_us: 250_000,
            model_assumptions: entry_assumptions(character),
        },
    );
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_projection_keeps_research_inputs_and_canonical_packets_together() {
        let request = serde_json::from_value(json!({
            "content_id":"ntesim-current","character_id":"character:1033",
            "level":80,"max_level":80,"refinement":1,"set_count":0
        }))
        .unwrap();
        let projection =
            studio_projection::project(ntesim_data::stat_projection::frozen_tables(), &request)
                .unwrap();
        assert_eq!(projection.energy_capacity, 120);
        let skill = &projection.public_actions["skill"];
        let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            recovery_us,
            cooldown_us,
            energy_cost,
            ..
        } = &skill.command
        else {
            panic!("expected ability")
        };
        assert_eq!(ability_id, "GA_Adler_Skill");
        assert_eq!(damage_effect_id, "GE_Player_Adler_Skill2_Damage");
        assert_eq!(
            (*recovery_us, *cooldown_us, *energy_cost),
            (1_200_000, 6_000_000, 0)
        );
        assert_eq!(skill.advance_us, 1_200_000);
        assert_eq!(
            skill.model_assumptions[TIMING_ID.as_str()],
            TIMING_ID.description()
        );
        assert!(projection.public_actions.contains_key("special"));
        let mut mint = request;
        mint.character_id = "character:1019".into();
        mint.slot = 2;
        let projection =
            studio_projection::project(ntesim_data::stat_projection::frozen_tables(), &mint)
                .unwrap();
        assert!(!projection.public_actions.contains_key("special"));
        assert!(projection.public_actions.contains_key("dash"));
        assert_eq!(
            projection.stochastic_crit.actor_sequences["character:1019"],
            56
        );
        assert_eq!(
            projection.stochastic_crit.eligible_effects["character:1019"],
            BTreeSet::from(["*".into()])
        );
        mint.set_id = Some("set:Suit1".into());
        mint.set_count = 4;
        let partial =
            studio_projection::project(ntesim_data::stat_projection::frozen_tables(), &mint)
                .unwrap();
        assert!(partial.static_stats.hp > 0);
        assert!(matches!(
            &partial.equipment_projection,
            super::super::studio_equipment::EquipmentProjection::Supported { gear_procs, .. }
                if gear_procs
                    .iter()
                    .any(|proc| matches!(proc, GearProcConfig::DiabolosChaosIgnore { .. }))
        ));
    }

    #[test]
    fn burst_energy_cost_matches_character_capacity() {
        for (id, expected_energy) in [
            ("character:1003", 120), // Sakiri
            ("character:1004", 120), // Lacrimosa
            ("character:1020", 120), // Haniel
            ("character:1023", 120), // Baicang
            ("character:1033", 120), // Adler
            ("character:1036", 120), // Zankou
            ("character:1039", 120), // Fadia
            ("character:1054", 120), // Daffodil
            ("character:1070", 120), // Aurelia
            ("character:1008", 100), // Skia
            ("character:1010", 100), // Nanally
            ("character:1019", 100), // Mint
            ("character:1021", 100), // Edgar
            ("character:1025", 100), // Hathor
            ("character:1042", 100), // Blackbird
            ("character:1046", 100), // Zero
            ("character:1052", 100), // Hotori (capacity 100, burst cost 0 via Timepiece units)
            ("character:1055", 100), // Jiuyuan
            ("character:1057", 100), // Akane Rin
            ("character:1071", 100), // Chaos
            ("character:1072", 100), // Linko
            ("character:1073", 100), // Chiz
            ("character:1075", 100), // Iroi
            ("character:1076", 60),  // Shinku (Defiant Spirit capacity 60)
            ("character:1090", 100), // Elyms
        ] {
            let request = serde_json::from_value(json!({
                "content_id":"ntesim-current","character_id":id,
                "level":80,"max_level":80,"refinement":1,"set_count":0
            }))
            .unwrap();
            let projection =
                studio_projection::project(ntesim_data::stat_projection::frozen_tables(), &request)
                    .unwrap();
            let expected_burst_cost = if id == "character:1052" {
                0
            } else {
                expected_energy
            };
            assert_eq!(
                projection.energy_capacity, expected_energy,
                "capacity for {id}"
            );
            if let Some(burst) = projection.public_actions.get("burst") {
                let GameActionCommand::InvokeAbility { energy_cost, .. } = &burst.command else {
                    panic!("expected ability command for burst");
                };
                assert_eq!(*energy_cost, expected_burst_cost, "burst cost for {id}");
            }
        }
    }
}
