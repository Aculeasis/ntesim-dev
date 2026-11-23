use crate::simulation::{GameActionCommand, PreparedSourceScenario, SourceTrialOutcome};

#[test]
fn source_curated_teams_complete_without_rotation_end_reserves() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../presets/studio");
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../presets/studio/index.json")).unwrap();
    for entry in manifest["scripts"].as_array().unwrap() {
        if entry["selectable"] != true {
            continue;
        }
        let source =
            std::fs::read_to_string(directory.join(entry["file"].as_str().unwrap())).unwrap();
        let prepared =
            PreparedSourceScenario::new(&source, Default::default(), &Default::default()).unwrap();
        let result = match prepared.run_batch(42).unwrap() {
            SourceTrialOutcome::Completed(trial) => trial,
            SourceTrialOutcome::Failed(result) => panic!(
                "{}: {:?}; {:?}",
                entry["id"], result.diagnostic, result.report.halt_diagnostic
            ),
        };
        assert_eq!(result.relative_elapsed_us, 120_000_000, "{}", entry["id"]);
    }
}

#[test]
fn source_insight_keeps_late_support_skills_and_completes_relative_horizon() {
    let source = include_str!("../../../../../presets/studio/zankou-insight.nte");
    let prepared =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let result = prepared.run(42).unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert!(result.diagnostic.is_none());
    assert_eq!(result.report.relative_elapsed_us, 120_000_000);
    let pauses = ntesim_core::kernel::timing::merge_intervals(
        result
            .report
            .timer_pauses
            .as_ref()
            .unwrap()
            .iter()
            .map(|pause| [pause.start_absolute_us, pause.end_absolute_us])
            .collect(),
    );
    // Completion alone passed with the old carry-only tail. Require actual
    // admitted Skills from all three supports in the final thirty seconds.
    for support in [1, 2, 3] {
        assert!(
            result.request.actions.iter().any(|command| {
                let GameActionCommand::InvokeAbility {
                    timestamp_us,
                    actor_index,
                    ability_id,
                    ..
                } = command
                else {
                    return false;
                };
                let paused: u64 = pauses
                    .iter()
                    .map(|[start, end]| timestamp_us.min(end).saturating_sub(*start))
                    .sum();
                *actor_index == support
                    && ability_id.ends_with("_Skill")
                    && timestamp_us - paused > 90_000_000
            }),
            "support {support} has no late Skill"
        );
    }
    super::assert_native_replay(&result);
}

#[test]
fn source_s5_executes_with_zero_defense_ignore() {
    let source = include_str!("../../../../../presets/studio/s5.nte")
        .replace("iteration=1000", "iteration=1")
        .replace(
            "zero add cartridge=speedyhedgehog count=4",
            "zero add cartridge=lostradiance count=4",
        );
    let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(result.report.success && result.report.execution_completed);
    assert!(result.diagnostic.is_none());
    // The later Skill reaches Zero's 75% execution while both 25% Lost Radiance
    // and 5% Iroi support apply. It must not fail the exact formula at 105%.
    assert!(result.report.hit_receipts.iter().any(|hit| {
        hit.actor_index == 2
            && hit.damage_effect_id == "GE_Player_Female046_Skill_Kill_Damage_lv1"
            && hit.time_us > 20_000_000
            && hit.time_us < 26_000_000
    }));
    super::assert_native_replay(&result);
}

#[test]
fn source_baselines_execute_every_a0_owner_twice_with_saved_native_replay() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut failures = Vec::new();
    for owner in crate::registry::character_descriptors().unwrap() {
        let path = match owner.slug {
            "mint" => directory.join("mint-native-cycles.nte"),
            "chaos" => directory.join("chaos-native-cycles.nte"),
            "nanally" => directory.join("../presets/native/nanally-full-kit.nte"),
            name => directory.join("baselines").join(format!("{name}.nte")),
        };
        let source = std::fs::read_to_string(&path).unwrap();
        let prepared =
            match PreparedSourceScenario::new(&source, Default::default(), &Default::default()) {
                Ok(prepared) => prepared,
                Err(error) => {
                    failures.push(format!("{}: {error:?}", owner.slug));
                    continue;
                }
            };
        let saved =
            serde_json::from_slice(&serde_json::to_vec(&prepared.export()).unwrap()).unwrap();
        let restored = PreparedSourceScenario::from_export(&saved, &Default::default()).unwrap();
        let result = restored.run(42).unwrap();
        if !result.report.success || !result.report.execution_completed {
            failures.push(format!(
                "{}: {:?}; {:?}",
                owner.slug, result.diagnostic, result.report.halt_diagnostic
            ));
            continue;
        }
        assert_eq!(result.source, source);
        assert_eq!(result.request.actors[0].character_id, owner.character_id);
        assert!(
            !result.request.actors[0].weapon_id.is_empty(),
            "{} equipped build",
            owner.slug
        );
        assert_eq!(result.request.actors[0].awakening.count(), 0);
        assert_eq!(
            result.report.relative_elapsed_us,
            result.request.relative_duration_us
        );
        assert!(result.report.hits_settled > 0);
        if result.report.quality != ntesim_contracts::Quality::BoundedHypothesis {
            failures.push(format!(
                "{}: quality {:?}",
                owner.slug, result.report.quality,
            ));
        }
        assert!(result.report.game_dps_computed);
        for action in ["Skill", "UltraSkill"] {
            let count = result.request.actions.iter().filter(|command| matches!(command,
                GameActionCommand::InvokeAbility {actor_index: 0, ability_id, ..}
                if ability_id.contains(action) && (action == "UltraSkill" || !ability_id.contains("UltraSkill"))
            )).count();
            assert!(count >= 2, "{} continuing {action}: {count}", owner.slug);
        }
        // Full report equality includes costs, targets, state/expiry timelines,
        // pending effects, owner resources, seeded draws and actual inputs.
        super::assert_native_replay(&result);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn source_build_access_reaches_every_weapon_refinement_and_set_threshold() {
    let owners = crate::registry::character_descriptors().unwrap();
    for weapon in ntesim_data::weapon_definitions::definitions() {
        let owner = owners
            .iter()
            .find(|owner| {
                ntesim_data::character::definition(owner.character_id)
                    .unwrap()
                    .weapon_group
                    == weapon.group
            })
            .unwrap();
        for refinement in 1..=5 {
            let source = format!(
                "options duration=5 crit=random start_fons=0;
                {} char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
                {} add weapon={} refine={refinement};
                target hp=100000000 def=1080 resist=0.2; active {}; {} attack;",
                owner.slug, owner.slug, weapon.native_id, owner.slug, owner.slug
            );
            let result =
                PreparedSourceScenario::new(&source, Default::default(), &Default::default())
                    .unwrap_or_else(|error| panic!("{} R{refinement}: {error:?}", weapon.native_id))
                    .run(42)
                    .unwrap();
            assert!(
                result.report.success && result.report.execution_completed,
                "{} R{refinement}: {:?}",
                weapon.native_id,
                result.diagnostic
            );
            assert_eq!(
                result.request.actors[0].weapon_id,
                format!("weapon:{}", weapon.native_id.to_ascii_lowercase())
            );
            assert_eq!(result.request.actors[0].weapon_tier, refinement);
            assert_eq!(
                result.report.quality,
                ntesim_contracts::Quality::BoundedHypothesis,
                "{} R{refinement}",
                weapon.native_id
            );
        }
    }
    for set in ntesim_data::set_definitions::SETS {
        for count in [2, 4] {
            let source = format!(
                "options duration=5 crit=random;
                mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
                mint add weapon=fork_appliance refine=1;
                mint add cartridge={} count={count};
                target hp=100000000 def=1080 resist=0.2; active mint; mint skill, attack;",
                set.native_id
            );
            let result =
                PreparedSourceScenario::new(&source, Default::default(), &Default::default())
                    .unwrap_or_else(|error| panic!("{} {count}pc: {error:?}", set.native_id))
                    .run(42)
                    .unwrap();
            assert!(
                result.report.success && result.report.execution_completed,
                "{} {count}pc: {:?}",
                set.native_id,
                result.diagnostic
            );
            assert_eq!(
                result.request.actors[0].set_id,
                Some(format!("set:{}", set.native_id))
            );
            assert_eq!(
                result.report.quality,
                ntesim_contracts::Quality::BoundedHypothesis,
                "{} {count}pc",
                set.native_id
            );
            super::assert_native_replay(&result);
        }
    }
}
