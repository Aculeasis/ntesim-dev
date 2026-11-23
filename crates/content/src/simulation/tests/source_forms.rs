use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_zankou_live_permissions_and_readiness_drive_two_complete_legal_rotations() {
    let source="options duration=120 crit=random;
        zankou char lvl=80/80 awakening=[4] talent=1,1,1 qte=1 start_energy=120;
        zankou add stats ce=1;
        target hp=1000000000 def=0 resist=0;active zankou;
        if !.zankou.bloodfeast.active || !.zankou.burst.ready {let assertion_failure=1/0;}
        repeat 2 {
            zankou attack;zankou attack[hold=200ms];
            if !.zankou.enhanced_entry.active || !.zankou.skill.ready {let assertion_failure=1/0;}
            zankou skill;
            if !.zankou.illusion.active || .zankou.mods.illusion<=0 {let assertion_failure=1/0;}
            while !.zankou.enhanced_soulcross.active {zankou attack;}
            if !.zankou.skill.ready {let assertion_failure=1/0;}
            zankou skill;
            if !.zankou.bloodfeast.active || !.zankou.burst.ready {let assertion_failure=1/0;}
            zankou burst;
            if !.zankou.free_inferno.active || !.zankou.burst.ready || .zankou.mods.free_inferno<=0 {let assertion_failure=1/0;}
            zankou burst;
            if .zankou.free_inferno.active || .zankou.burst.ready {let assertion_failure=1/0;}
            zankou attack:20;
        }";
    let result = run(source);
    assert!(
        result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic, result.report.halt_diagnostic
    );
    assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    assert_eq!(
        result
            .report
            .trace
            .iter()
            .filter(|e| e.event_kind == "zankou_form_transition")
            .count(),
        4
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_zankou_free_finisher_queries_expire_on_the_same_real_microsecond_as_admission() {
    let result=run("options duration=25;zankou char lvl=80/80 awakening=[4] talent=1,1,1 qte=1 start_energy=120;target hp=100000000 def=0 resist=0;active zankou;
        zankou burst;let end=.time+.zankou.mods.free_inferno;
        if end!=5 {let assertion_failure=1/0;}
        wait(end-.time-0.000001);
        if !.zankou.free_inferno.active || !.zankou.burst.ready || .zankou.mods.free_inferno!=0.000001 {let assertion_failure=1/0;}
        wait(0.000001);
        if .zankou.free_inferno.active || .zankou.burst.ready || .zankou.mods.free_inferno {let assertion_failure=1/0;}");
    assert!(
        result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic, result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_chaos_native_periods_preserve_paid_verdicts_across_form_expiry() {
    let source = include_str!("../../../../../examples/chaos-native-cycles.nte");
    for (periods, horizon, energy_carry, energy_overflow, crime_overflow) in
        [(1, 75, 229, 16, 2574), (2, 150, 208, 90, 5530)]
    {
        let source = source
            .replace("repeat 2 {", &format!("repeat {periods} {{"))
            .replace("duration=150", &format!("duration={horizon}"));
        let result = run(&source);
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}; {:?}",
            result.diagnostic,
            result.report.halt_diagnostic
        );
        assert_eq!(result.request.actors[0].initial_energy, 0);
        assert_eq!(result.request.hold_inputs.len(), 2 * periods);
        assert_eq!(result.report.hits_settled, 111 * periods);
        let actor = serde_json::to_value(&result.report.final_actor_states[0]).unwrap();
        assert_eq!(actor["spent_energy"], 100 * periods);
        assert_eq!(actor["energy"], 57);
        assert_eq!(
            actor["charge_remainder"],
            serde_json::to_value(crate::simulation::ExactFraction::new(energy_carry, 250).unwrap())
                .unwrap()
        );
        assert_eq!(actor["overflow_energy"], energy_overflow);
        let crime = &actor["fractional_resources"]["chaos.crime"];
        for (field, value) in [
            ("current", 382),
            ("spent", 2000 * periods),
            ("overflow", crime_overflow),
        ] {
            assert_eq!(
                crime[field],
                serde_json::json!(format!("{}/{}", value, 1)),
                "{field}"
            );
        }
        assert_eq!(crime["conserved"], true);
        // Level8 .312 ATK * compiled2530.5 * (1 + set .10 + weapon .20) * DEF .5.
        assert_eq!(
            result.report.hit_receipts[0].non_crit_damage,
            crate::simulation::ExactFraction::new(2_565_927, 5000).unwrap()
        );
        assert_eq!(result.report.hit_receipts[0].time_us, 214_573);
        let full: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Chaos_Branch2_Damage_Ultra")
            .collect();
        assert_eq!(full.len(), 4 * periods);
        for period in 0..periods {
            // The second captured transformed Verdict outlives the form's
            // World-frozen form's51.802493s end, without resetting Crime or cancelling stage2.
            let times: Vec<_> = full
                .iter()
                .filter(|hit| {
                    (period as u64 * 75_000_000 + 51_802_493
                        ..period as u64 * 75_000_000 + 54_000_000)
                        .contains(&hit.time_us)
                })
                .map(|hit| hit.time_us - period as u64 * 75_000_000)
                .collect();
            assert_eq!(times, [52_615_298, 53_165_886]);
        }
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_blackbird_selects_transformed_plunge_skill_and_finisher_over_two_cycles() {
    let result = run("options duration=110 crit=random;
        blackbird char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=100;
        blackbird add stats ce=1.1;
        target hp=100000000 def=0 resist=0; active blackbird;
        repeat 2 {
            if !.blackbird.burst.ready {let assertion_failure=1/0;}
            blackbird burst;
            if !.blackbird.transformed.active || .blackbird.skill.ready {let assertion_failure=1/0;}
            blackbird plunge[airtime=600ms,height_bonus=1/2];
            blackbird attack:5;
            if !.blackbird.skill.ready {let assertion_failure=1/0;}
            blackbird skill;
            if .blackbird.skill.ready {let assertion_failure=1/0;}
            blackbird attack:5;
            if !.blackbird.burst.ready {let assertion_failure=1/0;}
            blackbird burst;
            if .blackbird.transformed.active {let assertion_failure=1/0;}
            repeat 4 {blackbird attack:5;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}; energy={}; inputs={:?}",
        result.diagnostic,
        result.report.halt_diagnostic,
        result.report.final_actor_states[0].energy,
        result
            .request
            .actions
            .iter()
            .map(|command| command.timestamp_us())
            .collect::<Vec<_>>()
    );
    for ability in [
        "GA_BlackBird_UltraSkill",
        "GA_BlackBird_Evolve_UltraSkill",
        "GA_BlackBird_Evolve_Skill",
    ] {
        assert_eq!(result.request.actions.iter().filter(|command| matches!(command,
            crate::simulation::GameActionCommand::InvokeAbility{ability_id,..} if ability_id == ability)).count(), 2, "{ability}");
    }
    let plunges: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_BlackBird_Evolve_AirAttack_Damage")
        .collect();
    assert_eq!(plunges.len(), 2);
    for hit in plunges {
        assert_eq!(
            hit.time_us,
            result.request.actions[hit.command_index.unwrap()].timestamp_us() + 700_643
        );
    }
    assert_eq!(result.report.final_actor_states[0].spent_energy, 200);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_BlackBird_Evolve_UltraSkill_Damage")
            .count(),
        14
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_shinku_crimson_refreshes_and_expires_across_two_continuing_cycles() {
    let result = run("options duration=100 crit=random;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        shinku add stats ce=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active shinku;
        if .shinku.surging_crimson.active || .shinku.mods.surging_crimson {let assertion_failure=1/0;}
        repeat 2 {
            let burst_start=.time;
            shinku burst;
            if !.shinku.surging_crimson.active || .shinku.mods.surging_crimson!=burst_start+13-.time {let assertion_failure=1/0;}
            mint swap;
            if .shinku.onfield || !.shinku.surging_crimson.active {let assertion_failure=1/0;}
            wait(.shinku.mods.surging_crimson-0.000001);
            if .shinku.mods.surging_crimson!=0.000001 {let assertion_failure=1/0;}
            wait(0.000001);
            if .shinku.surging_crimson.active || .shinku.mods.surging_crimson {let assertion_failure=1/0;}
            shinku swap;
            repeat 4 {shinku attack:5;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.report.final_actor_states[0].spent_energy, 200);
    super::assert_native_replay(&result);
}

#[test]
fn source_shinku_queries_preserve_rng_and_unknown_names_diagnose_at_the_source() {
    let prefix = "options duration=15 crit=random;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        target hp=100000000 def=0 resist=0; active shinku;";
    let baseline = run(&format!("{prefix} shinku burst; shinku attack:5;"));
    let observed = run(&format!(
        "{prefix}
        if .shinku.mods.surging_crimson {{let assertion_failure=1/0;}}
        shinku burst;
        if !.shinku.surging_crimson.active {{let assertion_failure=1/0;}}
        shinku attack:5;"
    ));
    assert!(
        observed.report.execution_completed,
        "{:?}",
        observed.diagnostic
    );
    assert_eq!(observed.report.rng, baseline.report.rng);
    assert_eq!(
        serde_json::to_value(&observed.report.hit_receipts).unwrap(),
        serde_json::to_value(&baseline.report.hit_receipts).unwrap()
    );
    for (field, message) in [
        ("mods.crimson", "Unknown timed modifier"),
        ("crimson.active", "Unknown state"),
        ("resources.surging_crimson", "Unknown resource"),
    ] {
        let result = run(&format!("{prefix}\nwait(.shinku.{field});"));
        let diagnostic = result.diagnostic.unwrap();
        assert!(diagnostic.message.contains(message), "{diagnostic:?}");
        assert!(diagnostic.message.contains(&format!(".shinku.{field}")));
        assert_eq!(diagnostic.loc.line, 4);
        assert!(diagnostic.loc.length > 0);
        assert!(!result.report.execution_completed);
    }
}

#[test]
fn source_skia_gnaw_observes_actual_entry_and_clears_on_field_exit() {
    let result = run("options duration=25 crit=random;
        skia char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active skia;
        if .skia.gnaw.active || .skia.mods.gnaw {let assertion_failure=1/0;}
        for let i=0; i<2; i=i+1 {
            wait_until(i*13);
            skia skill;
            if !.skia.gnaw.active || .skia.mods.gnaw!=i*13+6.048984-.time {let assertion_failure=1/0;}
            mint swap;
            if .skia.gnaw.active || .skia.mods.gnaw || .skia.onfield {let assertion_failure=1/0;}
            wait(2);
            skia swap;
            if .skia.gnaw.active || .skia.mods.gnaw {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    let entries = result
        .report
        .trace
        .iter()
        .filter(|event| event.event_kind == "skia_gnaw_enter")
        .map(|event| event.time_us)
        .collect::<Vec<_>>();
    assert_eq!(entries, vec![1_048_984, 14_048_984]);
    super::assert_native_replay(&result);
}

#[test]
fn source_zankou_kindling_is_generated_by_qualified_switch_and_spent_by_burst() {
    let result = run("options duration=120 crit=random;
        zankou char lvl=80/80 awakening=[2] talent=1,1,1 qte=1 start_energy=120;
        zankou add stats ce=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0; active zankou;
        if .zankou.resources.kindling || .zankou.hunt.active || .zankou.mods.scorch_enhancement
            || .zankou.scorch_enhancement.active {let assertion_failure=1/0;}
        mint swap; zankou attack:20; mint swap;
        if .zankou.resources.kindling!=1 {let assertion_failure=1/0;}
        repeat 2 {
            wait(2); zankou burst;
            if .zankou.resources.kindling {let assertion_failure=1/0;}
            zankou attack:20; mint swap;
            if .zankou.resources.kindling!=1 {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    super::assert_native_replay(&result);
}

#[test]
fn source_mint_riposte_guard_reads_its_native_cooldown_reset_gate() {
    let result = run(
        "options duration=20 crit=random stamina_capacity=100 initial_stamina=100
        stamina_use_efficiency=1 stamina_recovery_rate=20;
        mint char lvl=80/80 awakening=1 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        if .mint.mods.riposte_guard || .mint.riposte_guard.active {let assertion_failure=1/0;}
        repeat 2 {
            mint skill[window=dash];
            mint dash[critical=true]; mint attack;
            if !.mint.riposte_guard.active || .mint.mods.riposte_guard<=0 {let assertion_failure=1/0;}
            wait(.mint.mods.riposte_guard);
            if .mint.riposte_guard.active || .mint.mods.riposte_guard {let assertion_failure=1/0;}
        }",
    );
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(
        result
            .report
            .trace
            .iter()
            .filter(|event| event.event_kind == "mint_a1_skill_cooldown_reset")
            .count(),
        2
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_skia_natural_gnaw_expiry_uses_the_actual_grant_deadline() {
    let result = run("options duration=20 crit=random;
        skia char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active skia;
        skia skill;
        wait_until(6.048983);
        if !.skia.gnaw.active || .skia.mods.gnaw!=0.000001 {let assertion_failure=1/0;}
        wait(0.000001);
        if .skia.gnaw.active || .skia.mods.gnaw {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_owner_defeat_clears_shinku_and_skia_timed_states_immediately() {
    for (owner, state, action) in [
        ("shinku", "surging_crimson", "shinku burst;"),
        ("skia", "gnaw", "skia skill;"),
    ] {
        let result = run(&format!(
            "options duration=20 crit=random;
            {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active {owner};
            incoming {owner} at=4 damage=100000000;
            {action}
            if !.{owner}.{state}.active {{let assertion_failure=1/0;}}
            wait_until(6);
            if .{owner}.hp || .{owner}.{state}.active || .{owner}.mods.{state} {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{owner}: {:?}; {:?}",
            result.diagnostic,
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_zankou_scorch_kill_exposes_the_native_r6_interval_and_kindling() {
    let result = run("options duration=65 crit=random;
        zankou char lvl=80/80 awakening=6 talent=1,1,1 qte=1;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=2000 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active zankou;
        lacrimosa swap;
        wait_until(2);
        if .target.hp || !.zankou.scorch_enhancement.active || .zankou.resources.kindling!=1 {let assertion_failure=1/0;}
        let end=.time+.zankou.mods.scorch_enhancement;
        if end!=60.99 {let assertion_failure=1/0;}
        wait(end-.time-0.000001);
        if .zankou.mods.scorch_enhancement!=0.000001 {let assertion_failure=1/0;}
        wait(0.000001);
        if .zankou.scorch_enhancement.active || .zankou.mods.scorch_enhancement
            || .zankou.resources.kindling!=1 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_zankou_r6_refreshes_on_a_second_scorched_target_defeat_without_extra_kindling() {
    let result=run("options duration=70 crit=fixed;
        zankou char lvl=80/80 awakening=6 talent=1,1,1 qte=1;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;target hp=5000 def=0 resist=0;target hp=20000 def=0 resist=0;
        active zankou;lacrimosa swap;zankou skill;zankou attack;wait_until(6);
        let end=.time+.zankou.mods.scorch_enhancement;
        if end<=60.99 || end>66 || .zankou.resources.kindling!=1 {let assertion_failure=1/0;}
        wait(end-.time-0.000001);
        if .zankou.mods.scorch_enhancement!=0.000001 {let assertion_failure=1/0;}
        wait(0.000001);
        if .zankou.scorch_enhancement.active || .zankou.resources.kindling!=1 {let assertion_failure=1/0;}");
    assert!(
        result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic, result.report.halt_diagnostic
    );
    let events = result
        .report
        .trace
        .iter()
        .filter(|e| e.event_kind == "zankou_r6_scorch_kill_enhancement")
        .collect::<Vec<_>>();
    assert_eq!(
        events.len(),
        2,
        "events={events:?}; targets={:?}",
        result.report.final_target_states
    );
    assert!(events[1].time_us > events[0].time_us);
    assert!(
        events
            .iter()
            .all(|e| e.data.as_ref().unwrap().duration_us == Some(60_000_000))
    );
    super::assert_native_replay(&result);
}
