use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

const PREFIX: &str = "options duration=6 iteration=1 crit=random;
    mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_hp=1000;
    zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=7 start_hp=1000;
    target hp=100000000 def=0 resist=0; active mint;";

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_zero_skill_funds_two_real_blossoms_with_authored_intensity() {
    for initial in [0, 100] {
        let source = format!(
            "options duration=24 iteration=1 crit=random;
            zero char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle={initial};
            zero add stats ci=100.25; zero add stats ci=247.75;
            mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
            target lvl=80 hp=100000000 def=0 resist=0;
            active zero;
            repeat 2 {{
                wait_until(.time + .zero.skill.cd);
                zero skill;
                if .zero.resources.cycle!=100 {{let assertion_failure=1/0;}}
                mint swap;
                if .zero.resources.cycle {{let assertion_failure=1/0;}}
                zero swap;
            }}
            wait_until(24);"
        );
        let result = run(&source);
        assert!(result.report.execution_completed, "{:?}", result.diagnostic);
        let report = serde_json::to_value(&result.report).unwrap();
        let cycle = &report["final_actor_states"][0]["cycle"];
        assert_eq!(cycle["spent"], 200);
        assert_eq!(cycle["granted"], 200 - initial);
        assert_eq!(cycle["current"], 0);
        assert_eq!(cycle["overflow"], 0);
        assert_eq!(cycle["conserved"], true);
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "reaction_blossom_created")
                .count(),
            2
        );
        // Level80 Blossom base9000 * (1+348/600), with no DEF/RES mitigation.
        let pistils: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.blossom")
            .collect();
        assert_eq!(pistils.len(), 10);
        assert!(
            pistils
                .iter()
                .all(|hit| hit["damage"] == "14220/1" && hit["actor_index"] == 0)
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_zero_r6_buffs_party_for_paid_cosmos_cycle_and_expires_on_time() {
    let simulate = |nodes| {
        run(&format!(
            "options duration=25 iteration=1 crit=fixed;
        zero char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1 start_cycle=100;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        mint add stats ci=600;
        target hp=100000000 def=0 resist=0; active zero;
        mint swap; mint attack;
        wait_until(10); zero swap;
        wait_until(15); mint swap; mint attack;
        wait_until(20); mint attack;"
        ))
    };
    let active = simulate("6");
    let locked = simulate("[6]");
    for result in [&active, &locked] {
        assert!(result.report.execution_completed, "{:?}", result.diagnostic);
        assert_eq!(
            result.report.final_actor_states[0]
                .cycle
                .as_ref()
                .unwrap()
                .spent,
            100
        );
        assert_eq!(
            result.report.final_actor_states[1]
                .cycle
                .as_ref()
                .unwrap()
                .spent,
            100
        );
        super::assert_native_replay(result);
    }
    let grants: Vec<_> = active
        .report
        .trace
        .iter()
        .filter(|event| event.event_kind == "zero_cosmos_cycle_atk")
        .collect();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].time_us, 0);
    assert_eq!(grants[0].from_actor, Some(0));
    assert!(
        locked
            .report
            .trace
            .iter()
            .all(|event| event.event_kind != "zero_cosmos_cycle_atk")
    );
    let hits = |result: &SourceScenarioReport| {
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| {
                hit.actor_index == 1 && hit.damage_effect_id.starts_with("GE_Player_Mint_Melee")
            })
            .map(|hit| (hit.time_us, hit.non_crit_damage))
            .collect::<Vec<_>>()
    };
    let boosted = hits(&active);
    let baseline = hits(&locked);
    assert_eq!(boosted.len(), 6);
    assert_eq!(baseline.len(), 6);
    let mint = &active.request.actors[1].static_stats;
    let baseline_atk = crate::simulation::ExactFraction::from_ratio(mint.atk).unwrap();
    let multiplier = baseline_atk
        .add(crate::simulation::ExactFraction::new(u128::from(mint.base_atk) * 3, 25).unwrap())
        .unwrap()
        .try_div(baseline_atk)
        .unwrap();
    assert_eq!(
        boosted
            .iter()
            .filter(|(time, _)| *time < 20_000_000)
            .count(),
        4
    );
    for ((time, damage), (control_time, control_damage)) in boosted.into_iter().zip(baseline) {
        assert_eq!(time, control_time);
        assert_eq!(
            damage,
            if time < 20_000_000 {
                control_damage.mul(multiplier).unwrap()
            } else {
                control_damage
            }
        );
    }
}

#[test]
fn source_named_vitals_energy_and_field_queries_return_exact_native_values() {
    let source = format!(
        "{PREFIX}
        wait(.mint.hp/1000000);
        wait(.mint.hpmax/1000000);
        wait(.mint.hpratio*.mint.hpmax/1000000);
        wait(.zero.energy/1000000);
        wait(.zero.energymax/1000000);
        wait(.mint.onfield);
        wait(.zero.onfield);"
    );
    let result = run(&source);
    assert!(result.report.execution_completed, "{:?}", result.diagnostic);
    let durations = result
        .occurrences
        .iter()
        .map(|occurrence| occurrence.completed_us.unwrap() - occurrence.started_us)
        .collect::<Vec<_>>();
    assert_eq!(
        durations,
        vec![
            1000,
            result.request.actors[0].initial_hp_max,
            1000,
            7,
            result.request.actors[1].energy_capacity,
            1_000_000,
            0
        ]
    );
    assert_eq!(result.report.total_events_processed, 0);
}

#[test]
fn source_named_queries_see_equal_time_damage_off_field_and_implicit_switches() {
    let source = format!(
        "{PREFIX}
        incoming mint at=1 damage=100;
        incoming zero at=1 damage=200;
        wait_until(1);
        if .mint.hp==900 && .zero.hp==800 && .mint.onfield && !.zero.onfield {{wait(0.1);}}
        else {{let assertion_failure=1/0;}}
        zero attack;
        if .zero.onfield && !.mint.onfield && .zero.energy==.actor.energy
            && .zero.energy>7 && .mint.hp==900 {{wait(0);}}
        else {{let assertion_failure=1/0;}}"
    );
    let result = run(&source);
    assert!(result.report.execution_completed, "{:?}", result.diagnostic);
    assert_eq!(result.occurrences[1].completed_us, Some(1_100_000));
    assert_eq!(
        result
            .request
            .actions
            .iter()
            .filter(|action| matches!(
                action,
                crate::simulation::GameActionCommand::InvokeAbility { .. }
            ))
            .count(),
        1
    );
    assert_eq!(
        result
            .request
            .actions
            .iter()
            .filter(|action| matches!(
                action,
                crate::simulation::GameActionCommand::SwitchActor { .. }
            ))
            .count(),
        1
    );
}

#[test]
fn source_named_queries_do_not_draw_rng_and_unknown_visited_names_diagnose() {
    let baseline = run(&format!("{PREFIX} mint attack; zero attack;"));
    let observed = run(&format!(
        "{PREFIX}
        if .mint.hp==1000 && .zero.energy==7 {{wait(0);}}
        mint attack;
        if .mint.hpratio*.mint.hpmax==1000 {{wait(0);}}
        zero attack;"
    ));
    assert!(
        observed.report.execution_completed,
        "{:?}",
        observed.diagnostic
    );
    assert_eq!(observed.report.rng, baseline.report.rng);
    let outcomes = |result: &SourceScenarioReport| {
        serde_json::to_value(
            result
                .report
                .hit_receipts
                .iter()
                .map(|hit| {
                    (
                        hit.time_us,
                        hit.actor_index,
                        hit.target_index,
                        hit.damage,
                        hit.critical,
                        &hit.crit_sample,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    };
    assert_eq!(outcomes(&observed), outcomes(&baseline));
    for field in [".ghost.hp", ".mint.unknown"] {
        let result = run(&format!("{PREFIX} wait({field}); mint attack;"));
        assert!(!result.report.success);
        assert!(result.request.actions.is_empty());
        let diagnostic = result.diagnostic.unwrap();
        assert!(diagnostic.message.contains(field));
        assert!(diagnostic.loc.length > 0);
    }
    let skipped = run(&format!(
        "{PREFIX} if 0 && .ghost.hp {{let assertion_failure=1/0;}} mint attack;"
    ));
    assert!(skipped.report.execution_completed);
}

#[test]
fn source_hpmax_query_reads_dynamic_set_stacks_and_equal_time_expiry() {
    let result = run("options duration=12 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp=1000;
        mint add cartridge=Suit10 count=4;
        target hp=100000000 def=0 resist=0;
        incoming mint at=1 damage=100; active mint;
        let initial=.mint.hpmax;
        wait_until(1);
        wait(.mint.hpmax/1000000);
        wait(.mint.hp/1000000);
        wait_until(11);
        wait(.mint.hpmax/1000000);
        if .mint.hpmax!=initial {let assertion_failure=1/0;}");
    assert!(result.report.execution_completed, "{:?}", result.diagnostic);
    let base = result.request.actors[0].static_stats.base_hp.unwrap();
    // Static 2pc +10% and one dynamic +4% share the base-HP zone; each
    // effective maximum rounds once. It is not initial MaxHP * 1.04.
    let initial = (base * 110 + 50) / 100;
    let stacked = (base * 114 + 50) / 100;
    assert_eq!(result.request.actors[0].initial_hp_max, initial);
    let duration = |index: usize| {
        let occurrence = &result.occurrences[index];
        occurrence.completed_us.unwrap() - occurrence.started_us
    };
    assert_eq!(duration(1), stacked);
    assert_eq!(duration(2), 900);
    assert_eq!(duration(4), initial);
}

#[test]
fn source_team_stamina_reads_exact_continuing_account_and_recovery_boundaries() {
    let prefix = "options duration=14 crit=random stamina_capacity=40 initial_stamina=30
        stamina_use_efficiency=1 stamina_recovery_rate=5;
        nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active nanally;";
    let actions = "nanally dash; wait(0.4); nanally dash;
        wait_until(2.9); wait_until(3); mint swap;
        wait_until(6); mint dash; wait_until(8); wait_until(8.1);
        wait_until(12); mint attack;";
    let baseline = run(&format!("{prefix} {actions}"));
    let observed = run(&format!(
        "{prefix}
        if .team.stamina!=30 || .team.staminamax!=40 {{let assertion_failure=1/0;}}
        nanally dash; if .team.stamina!=15 {{let assertion_failure=1/0;}}
        wait(0.4); nanally dash; if .team.stamina!=0 {{let assertion_failure=1/0;}}
        wait_until(2.9); if .team.stamina!=0 {{let assertion_failure=1/0;}}
        wait_until(3); if .team.stamina!=0.5 {{let assertion_failure=1/0;}}
        mint swap; wait_until(6);
        if .team.stamina!=15.5 || .team.staminamax!=40 {{let assertion_failure=1/0;}}
        mint dash; if .team.stamina!=0.5 {{let assertion_failure=1/0;}}
        wait_until(8); if .team.stamina!=0.5 {{let assertion_failure=1/0;}}
        wait_until(8.1); if .team.stamina!=1 {{let assertion_failure=1/0;}}
        wait_until(12); if .team.stamina!=20.5 {{let assertion_failure=1/0;}}
        mint attack;"
    ));
    assert!(
        observed.report.success && observed.report.execution_completed,
        "{:?}",
        observed.diagnostic
    );
    // Queries introduce neither RNG draws nor resource settlement/events.
    super::assert_same_combat(&observed.report, &baseline.report);
    super::assert_native_replay(&observed);
}

#[test]
fn source_team_stamina_requires_profile_and_preserves_fractional_or_zero_accounts() {
    for field in ["stamina", "staminamax"] {
        let result = run(&format!("{PREFIX} wait(.team.{field}); mint attack;"));
        assert!(!result.report.success);
        assert!(result.request.actions.is_empty());
        let diagnostic = result.diagnostic.unwrap();
        assert!(
            diagnostic
                .message
                .contains("Missing combat stamina profile")
        );
        assert!(diagnostic.message.contains(field));
        assert!(diagnostic.loc.length > 0);
    }
    for initial in ["0", "0.125"] {
        let result = run(&format!(
            "options duration=600 crit=fixed stamina_capacity=40.25
            initial_stamina={initial} stamina_use_efficiency=0.5 stamina_recovery_rate=0;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active mint;
            if .team.stamina!={initial} || .team.staminamax!=40.25 {{let assertion_failure=1/0;}}
            wait_until(600);
            if .team.stamina!={initial} || .team.staminamax!=40.25 {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(result.report.total_events_processed, 0);
        super::assert_native_replay(&result);
    }
    let skipped = run(&format!(
        "{PREFIX} if 0 && .team.stamina {{let assertion_failure=1/0;}} mint attack;"
    ));
    assert!(skipped.report.success && skipped.report.execution_completed);
}

#[test]
fn source_aiming_queries_follow_native_toggles_dash_skill_and_field_exit() {
    for owner in ["adler", "nanally", "jiuyuan"] {
        let result = run(&format!(
            "options duration=45 crit=random;
            {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active {owner};
            if .{owner}.aiming.active {{let assertion_failure=1/0;}}
            {owner} special;
            if !.{owner}.aiming.active {{let assertion_failure=1/0;}}
            {owner} attack; {owner} swap;
            if !.{owner}.aiming.active {{let assertion_failure=1/0;}}
            {owner} special;
            if .{owner}.aiming.active {{let assertion_failure=1/0;}}
            {owner} special; {owner} dash;
            if .{owner}.aiming.active {{let assertion_failure=1/0;}}
            {owner} special; {owner} skill;
            if .{owner}.aiming.active {{let assertion_failure=1/0;}}
            wait_until(20); {owner} special; mint attack;
            if .{owner}.aiming.active || .{owner}.onfield {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{owner}: {:?}; {:?}",
            result.diagnostic,
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
    }
    let unknown = run(&format!("{PREFIX} wait(.mint.aiming.active);"));
    assert!(!unknown.report.success);
    assert!(
        unknown
            .diagnostic
            .unwrap()
            .message
            .contains("Unknown state 'aiming'")
    );
}

#[test]
fn source_lacrimosa_form_queries_preserve_native_mode_across_switches_and_two_cycles() {
    let result = run("options duration=30 crit=random;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active lacrimosa;
        for let i=0; i<2; i=i+1 {
            if !.lacrimosa.ranged.active || .lacrimosa.melee.active {let assertion_failure=1/0;}
            lacrimosa special;
            if .lacrimosa.ranged.active || !.lacrimosa.melee.active {let assertion_failure=1/0;}
            mint attack;
            if .lacrimosa.ranged.active || !.lacrimosa.melee.active || .lacrimosa.onfield {let assertion_failure=1/0;}
            lacrimosa attack;
            if !.lacrimosa.melee.active {let assertion_failure=1/0;}
            lacrimosa special;
            if !.lacrimosa.ranged.active || .lacrimosa.melee.active {let assertion_failure=1/0;}
            wait(2);
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_nanally_states_follow_two_continuing_cast_cycles_without_free_burst_damage() {
    let result = run("options duration=50 crit=random;
        nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        nanally add stats ce=1;
        target hp=100000000 def=0 resist=0; active nanally;
        if .nanally.authority.active || .nanally.underboss.active
            || .nanally.mods.authority!=0 || .nanally.mods.underboss!=0 {let assertion_failure=1/0;}
        for let cycle=0; cycle<2; cycle=cycle+1 {
            while .time<cycle*22 {wait(cycle*22-.time);} nanally skill;
            if !.nanally.authority.active || .nanally.mods.authority!=cycle*22+12.429917-.time {let assertion_failure=1/0;}
            while .time<cycle*22+2 {wait(cycle*22+2-.time);} nanally burst;
            if !.nanally.underboss.active || .nanally.mods.underboss!=cycle*22+8-.time {let assertion_failure=1/0;}
            for let chain=0; chain<3; chain=chain+1 {nanally attack:5;} while .time<cycle*22+21 {wait(cycle*22+21-.time);}
            if .nanally.authority.active || .nanally.underboss.active
                || .nanally.mods.authority!=0 || .nanally.mods.underboss!=0 {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    // Per cycle:42 Normal,5 Skill,7 Burst and10 Cat packets before the six-second deadline.
    assert_eq!(result.report.hits_settled, 128);
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .all(|hit| hit.damage_effect_id != "GE_Player_Nanally_Cat_AirAttack_Damage")
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_adler_karma_is_persistent_and_skill_consumption_continues_across_cycles() {
    let result = run("options duration=60 crit=random;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active adler;
        if .adler.resources.karma!=0 {let assertion_failure=1/0;}
        repeat 2 {
            adler attack;
            if .adler.resources.karma!=1 {let assertion_failure=1/0;}
            wait(16);
            if .adler.resources.karma!=1 {let assertion_failure=1/0;}
            adler skill;
            if .adler.resources.karma!=0 {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_aurelia_reads_native_cadenza_and_crescendo_at_exact_expiry() {
    let result = run("options duration=40 crit=random;
        aurelia char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active aurelia;
        if .aurelia.mods.cadenza || .aurelia.cadenza.active
            || .aurelia.mods.crescendo || .aurelia.resources.crescendo {let assertion_failure=1/0;}
        for let cycle=0; cycle<2; cycle=cycle+1 {
            wait_until(cycle*18);
            aurelia skill;
            if !.aurelia.cadenza.active || .aurelia.mods.cadenza<=0 {let assertion_failure=1/0;}
            aurelia attack:3;
            if .aurelia.resources.crescendo!=3 || !.aurelia.crescendo.active {let assertion_failure=1/0;}
            wait(.aurelia.mods.crescendo);
            if .aurelia.resources.crescendo || .aurelia.crescendo.active
                || .aurelia.mods.crescendo {let assertion_failure=1/0;}
            wait(.aurelia.mods.cadenza);
            if .aurelia.cadenza.active || .aurelia.mods.cadenza {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    // Current native Skill grants twelve real seconds from each admission.
    // The final zero-time check therefore completes at 18 + 12 seconds.
    assert_eq!(
        result.occurrences.last().unwrap().completed_us,
        Some(30_000_000)
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_aurelia_burst_leaves_cadenza_inactive_and_projects_harmonics_at_b2() {
    for (level, expected_harmonics) in [("30/30", 0), ("40/40", 3)] {
        let result = run(&format!(
            "options duration=16 crit=fixed;
            aurelia char lvl={level} awakening=[2] talent=1,1,1 qte=1 start_energy=120 start_cycle=100;
            lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active aurelia;
            aurelia burst;
            if .aurelia.cadenza.active {{let assertion_failure=1/0;}}
            lacrimosa swap; wait(8);"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(
            result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id == crate::characters::aurelia::harmonics::PACKET)
                .count(),
            expected_harmonics
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_adler_projects_righteous_heart_and_burst_callbacks() {
    let result = run("options duration=8 crit=fixed;
        adler char lvl=80/80 awakening=[1,5] talent=1,1,1 qte=1 start_energy=120;
        target hp=100000000 def=0 resist=0; active adler; adler burst; wait(4);");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let trace = &result.report.trace;
    for kind in ["adler_a1_karma_grant", "adler_a5_energy_refund"] {
        let times: Vec<_> = trace
            .iter()
            .filter(|event| event.event_kind == kind)
            .map(|event| event.time_us)
            .collect();
        assert_eq!(times, [2_343_074]);
    }
    assert!(
        result
            .report
            .resolved_model_assumptions
            .contains_key(crate::characters::adler::righteous_heart::HYPOTHESIS_ID.as_str())
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_jiuyuan_pact_uses_resonance_count_instead_of_node_six() {
    for (nodes, reductions) in [("[6]", 0), ("6", 1)] {
        let result = run(&format!(
            "options duration=4 crit=fixed;
            jiuyuan char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active jiuyuan; jiuyuan skill; wait(2);"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        let pact = result.report.final_target_states[0]
            .statuses
            .iter()
            .find(|status| status.id == "jiuyuan.rose_pact")
            .expect("Skill must apply Pact");
        assert_eq!(pact.resistance_reductions.len(), reductions);
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_owner_query_names_distinguish_absent_modifiers_from_unknown_fields() {
    for (owner, field, message) in [
        ("aurelia", "mods.unknown", "Unknown timed modifier"),
        ("aurelia", "resources.unknown", "Unknown resource"),
        ("aurelia", "unknown.active", "Unknown state"),
        ("adler", "mods.karma", "Unknown timed modifier"),
        ("linko", "mods.xiaozhen", "Unknown timed modifier"),
        ("linko", "resources.xiaozhen_energy", "Unknown resource"),
    ] {
        let result = run(&format!(
            "options duration=1 crit=random;
            {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active {owner};
            wait(.{owner}.{field});"
        ));
        let diagnostic = result.diagnostic.as_ref().unwrap();
        assert!(diagnostic.message.contains(message), "{diagnostic:?}");
        assert!(diagnostic.message.contains(&format!(".{owner}.{field}")));
        assert_eq!(diagnostic.loc.line, 4);
        assert!(!result.report.execution_completed);
    }
}

#[test]
fn source_adler_blessing_follows_its_living_carrier_and_expires_without_refresh() {
    for (awakening, lifetime) in [(0, 8), (6, 12)] {
        let result = run(&format!("options duration=45 crit=random;
            adler char lvl=80/80 awakening={awakening} talent=1,1,1 qte=1;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active adler;
            if .adler.blessing.active || .adler.mods.blessing {{let assertion_failure=1/0;}}
            for let cycle=0; cycle<2; cycle=cycle+1 {{
                wait_until(cycle*20); adler skill;
                if !.adler.blessing.active || .adler.mods.blessing!=cycle*20+{lifetime}-.time {{let assertion_failure=1/0;}}
                mint swap;
                if !.adler.blessing.active || .adler.onfield {{let assertion_failure=1/0;}}
                wait(.adler.mods.blessing);
                if .adler.blessing.active || .adler.mods.blessing {{let assertion_failure=1/0;}}
            }}"));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        assert_eq!(
            result.occurrences.last().unwrap().completed_us,
            Some((20 + lifetime) * 1_000_000)
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_adler_blessing_depletion_is_immediately_visible_off_field() {
    let result = run("options duration=10 crit=random;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint add stats hp=100000;
        target hp=100000000 def=0 resist=0; active adler;
        incoming mint at=3 damage=50000;
        adler skill; mint swap;
        if !.adler.blessing.active {let assertion_failure=1/0;}
        wait_until(3);
        if .adler.blessing.active || .adler.mods.blessing || .mint.hp==0 {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_adler_death_removes_the_off_field_blessing_query() {
    let result = run("options duration=10 crit=random;
        adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active adler;
        incoming adler at=3 damage=50000;
        adler skill; mint swap;
        if !.adler.blessing.active {let assertion_failure=1/0;}
        wait_until(3);
        if .adler.hp || .adler.blessing.active || .adler.mods.blessing {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_edgar_keys_capture_at_field_spawn_and_wake_expires_in_two_cycles() {
    let result = run("options duration=100 crit=random;
        edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        edgar add stats ce=1;
        target hp=100000000 def=0 resist=0; active edgar;
        if .edgar.resources.keys || .edgar.mods.finnegans_wake
            || .edgar.finnegans_wake.active || .edgar.out_of_combat.active {let assertion_failure=1/0;}
        for let cycle=0; cycle<2; cycle=cycle+1 {
            edgar skill;
            if .edgar.resources.keys!=1 {let assertion_failure=1/0;}
            let burst_start=.time;
            edgar burst;
            if .edgar.resources.keys || !.edgar.finnegans_wake.active
                || .edgar.mods.finnegans_wake!=burst_start+11-.time {let assertion_failure=1/0;}
            while .edgar.mods.finnegans_wake>0 {wait(.edgar.mods.finnegans_wake);}
            if .edgar.finnegans_wake.active || .edgar.mods.finnegans_wake {let assertion_failure=1/0;}
            repeat 4 {edgar attack:5;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_edgar_off_field_key_observer_is_initialized_without_an_owner_action() {
    let result = run("options duration=10 crit=random;
        edgar char lvl=80/80 awakening=1 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active mint;
        zero swap;
        if .edgar.resources.keys!=1 || .edgar.onfield {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert!(
        result
            .request
            .model_assumptions
            .contains_key(crate::characters::edgar::clock::MODEL.as_str())
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_baicang_stored_words_activate_in_two_continuing_domains() {
    let result = run(include_str!(
        "../../../../../presets/native/baicang-words.nte"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.request.actors[0].energy_capacity, 120);
    assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    super::assert_native_replay(&result);
}

#[test]
fn source_baicang_word_caps_and_exact_individual_expiry_remain_owner_defined() {
    for (awakening, cap) in [(0, 3), (3, 4)] {
        let result = run(&format!(
            "options duration=35 crit=random;
            baicang char lvl=80/80 awakening={awakening} talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active baicang;
            repeat 3 {{baicang attack:5;}}
            if .baicang.resources.words!={cap} {{let assertion_failure=1/0;}}
            wait(15);
            if .baicang.resources.words {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{:?}",
            result.diagnostic
        );
        super::assert_native_replay(&result);
    }
    let result = run("options duration=50 crit=random;
        baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active baicang;
        repeat 2 {
            baicang attack:2;
            let third_start=.time;
            baicang attack;
            if .baicang.resources.words!=1 {let assertion_failure=1/0;}
            wait_until(third_start+15.166254);
            if .baicang.resources.words!=1 {let assertion_failure=1/0;}
            wait(0.000001);
            if .baicang.resources.words {let assertion_failure=1/0;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn baicang_domain_query_does_not_expose_an_unborn_field_as_active() {
    let owner =
        crate::characters::RuntimeState::Baicang(crate::characters::baicang::awakenings::State {
            domain_start_us: 10_000_000,
            domain_until_us: 22_000_000,
            ..Default::default()
        });
    for (now, duration) in [(9_999_999, 0), (10_000_000, 12_000_000), (22_000_000, 0)] {
        assert_eq!(owner.live_modifier_us("domain", now).unwrap(), duration);
        assert_eq!(
            owner.live_state_active("domain", now).unwrap(),
            duration > 0
        );
    }
    assert!(owner.live_modifier_us("unknown", 0).is_err());
}

#[test]
fn source_fadia_lilith_permission_expires_and_clears_on_field_exit_across_cycles() {
    let result = run("options duration=100 crit=random;
        fadia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
        fadia add stats ce=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active fadia;
        if .fadia.lilith.active || .fadia.mods.lilith || .fadia.out_of_combat.active {let assertion_failure=1/0;}
        for let cycle=0; cycle<2; cycle=cycle+1 {
            fadia skill;
            let burst_start=.time;
            fadia burst;
            if !.fadia.lilith.active || .fadia.mods.lilith!=burst_start+9.42-.time {let assertion_failure=1/0;}
            if cycle==0 {while .fadia.mods.lilith>0 {wait(.fadia.mods.lilith);}}
            else {mint swap; fadia swap;}
            if .fadia.lilith.active || .fadia.mods.lilith {let assertion_failure=1/0;}
            repeat 3 {fadia attack:5;}
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
fn source_fadia_burst_only_uses_the_same_native_form_permission() {
    let result = run("options duration=15 crit=random;
        fadia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
        target hp=100000000 def=0 resist=0; active fadia;
        fadia burst;
        if !.fadia.lilith.active || .fadia.mods.lilith!=9.42-.time {let assertion_failure=1/0;}
        while .fadia.mods.lilith>0 {wait(.fadia.mods.lilith);}
        if .fadia.lilith.active || .fadia.mods.lilith {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_linko_oversync_and_off_field_companion_continue_across_two_cycles() {
    let result = run("options duration=100 crit=random;
        linko char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        linko add stats ce=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active linko;
        if .linko.mods.oversync || .linko.oversync.active || .linko.xiaozhen.active {let assertion_failure=1/0;}
        repeat 2 {
            linko skill;
            if !.linko.xiaozhen.active {let assertion_failure=1/0;}
            let burst_start=.time;
            linko burst;
            if !.linko.oversync.active || .linko.mods.oversync!=burst_start+15-.time {let assertion_failure=1/0;}
            mint swap;
            if .linko.onfield || !.linko.xiaozhen.active || !.linko.oversync.active {let assertion_failure=1/0;}
            wait(.linko.mods.oversync);
            if .linko.oversync.active || .linko.mods.oversync || !.linko.xiaozhen.active {let assertion_failure=1/0;}
            linko swap;
            repeat 4 {linko attack:5;}
        }");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.report.final_actor_states[0].spent_energy, 200);
    let pulses = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Xiaozhen_Skill1_Damage")
        .map(|hit| hit.time_us)
        .collect::<Vec<_>>();
    assert_eq!(&pulses[..2], &[10_000_000, 18_000_000]);
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .all(|hit| !hit.damage_effect_id.contains("ThrowXiaozhen"))
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_linko_defeat_clears_live_permission_and_cancels_pending_companion_pulses() {
    let result = run("options duration=20 crit=random;
        linko char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active linko;
        incoming linko at=4 damage=100000000;
        linko skill; linko burst; mint swap;
        if !.linko.xiaozhen.active || !.linko.oversync.active {let assertion_failure=1/0;}
        wait_until(6);
        if .linko.hp || .linko.xiaozhen.active || .linko.oversync.active || .linko.mods.oversync {let assertion_failure=1/0;}
        wait_until(20);");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .all(|hit| hit.damage_effect_id != "GE_Xiaozhen_Skill1_Damage")
    );
    super::assert_native_replay(&result);
}
