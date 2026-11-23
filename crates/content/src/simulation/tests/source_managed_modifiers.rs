use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_zero_support_atk_follows_two_native_entries_and_off_field_expiry() {
    let result = run("options duration=50 crit=random;
        zero char lvl=80/80 awakening=5 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0; active mint;
        if .zero.mods.support_atk || .zero.support_atk.active {let assertion_failure=1/0;}
        for let i=0; i<2; i=i+1 {
            wait_until(i*25);
            if i==1 {nanally swap;}
            zero swap;
            if !.zero.support_atk.active || .zero.mods.support_atk!=i*25+20-.time {let assertion_failure=1/0;}
            mint swap;
            if .zero.onfield || .zero.mods.support_atk!=i*25+20-.time {let assertion_failure=1/0;}
            wait_until(i*25+20-0.000001);
            if .zero.mods.support_atk!=0.000001 {let assertion_failure=1/0;}
            wait(0.000001);
            if .zero.support_atk.active || .zero.mods.support_atk {let assertion_failure=1/0;}
        }");
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
            .filter(|event| event.event_kind == "zero_a5_atk_buff")
            .count(),
        2
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_zero_support_buff_has_no_living_carrier_after_owner_defeat() {
    let result = run("options duration=25 crit=random;
        zero char lvl=80/80 awakening=5 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0; active mint;
        incoming zero at=4 damage=100000000;
        zero swap; mint swap;
        if !.zero.support_atk.active {let assertion_failure=1/0;}
        wait_until(4);
        if .zero.hp || .zero.support_atk.active || .zero.mods.support_atk {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_mint_managed_buffs_keep_independent_reaction_refresh_and_skill_deadlines() {
    let result = run("options duration=25 crit=random;
        mint char lvl=80/80 awakening=5 talent=1,1,1 qte=1 start_cycle=100;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0; active mint;
        if .mint.attendance.active || .mint.crit_damage.active {let assertion_failure=1/0;}
        mint skill;
        if !.mint.crit_damage.active || .mint.mods.crit_damage!=6-.time {let assertion_failure=1/0;}
        let leave=.time;
        zero swap;
        if !.mint.attendance.active || .mint.mods.attendance!=leave+15-.time {let assertion_failure=1/0;}
        let enter=.time;
        mint swap;
        if .mint.mods.attendance!=enter+15-.time {let assertion_failure=1/0;}
        wait_until(6);
        if .mint.crit_damage.active || .mint.mods.crit_damage {let assertion_failure=1/0;}
        wait(.mint.skill.cd);
        let next=.time;
        mint skill;
        if !.mint.crit_damage.active || .mint.mods.crit_damage!=next+6-.time {let assertion_failure=1/0;}
        wait_until(next+6);
        if .mint.crit_damage.active || .mint.mods.crit_damage || !.mint.attendance.active {let assertion_failure=1/0;}
        wait_until(enter+15);
        if .mint.attendance.active || .mint.mods.attendance {let assertion_failure=1/0;}");
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
            .filter(|event| event.event_kind == "mint_a3_attendance_bonus_applied")
            .count(),
        2
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_sakiri_team_atk_reads_the_actual_carrier_across_two_paid_bursts() {
    let result = run("options duration=90 crit=random;
        sakiri char lvl=80/80 awakening=4 talent=1,1,1 qte=1 start_energy=120;
        sakiri add stats ce=3;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active sakiri;
        if .sakiri.mods.team_atk || .sakiri.team_atk.active {let assertion_failure=1/0;}
        repeat 2 {
            let start=.time;
            sakiri burst;
            if !.sakiri.team_atk.active || .sakiri.mods.team_atk!=start+20-.time {let assertion_failure=1/0;}
            mint swap;
            if .sakiri.onfield || .sakiri.mods.team_atk!=start+20-.time {let assertion_failure=1/0;}
            while .time<start+20 {wait(start+20-.time);}
            if .sakiri.team_atk.active || .sakiri.mods.team_atk {let assertion_failure=1/0;}
            sakiri swap;
            repeat 3 {sakiri attack:5;}
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
fn source_sakiri_team_atk_obeys_native_snapshot_and_living_recipient_lifetimes() {
    for (victim, expected) in [("sakiri", true), ("mint", false)] {
        let result = run(&format!(
            "options duration=25 crit=random;
            sakiri char lvl=80/80 awakening=4 talent=1,1,1 qte=1 start_energy=120;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active sakiri;
            incoming {victim} at=4 damage=100000000;
            sakiri burst;
            while .time<4 {{wait(4-.time);}}
            if .sakiri.team_atk.active!={} {{let assertion_failure=1/0;}}
            if .sakiri.mods.team_atk!={} {{let assertion_failure=1/0;}}
            while .time<20 {{wait(20-.time);}}
            if .sakiri.team_atk.active || .sakiri.mods.team_atk {{let assertion_failure=1/0;}}",
            u8::from(expected),
            if expected { 16 } else { 0 }
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{victim}: {:?}; {:?}",
            result.diagnostic,
            result.report.halt_diagnostic
        );
        super::assert_native_replay(&result);
    }
    let alone = run("options duration=21 crit=random;
        sakiri char lvl=80/80 awakening=4 talent=1,1,1 qte=1 start_energy=120;
        target hp=100000000 def=0 resist=0; active sakiri;
        sakiri burst;
        if .sakiri.team_atk.active || .sakiri.mods.team_atk {let assertion_failure=1/0;}");
    assert!(alone.report.execution_completed, "{:?}", alone.diagnostic);
}

#[test]
fn source_sakiri_defeat_bonus_reads_the_native_independent_deadline() {
    let result = run("options duration=16 crit=random;
        sakiri char lvl=80/80 awakening=3 talent=1,1,1 qte=1;
        target hp=1 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active sakiri;
        if .sakiri.mods.defeat_bonus || .sakiri.defeat_bonus.active {let assertion_failure=1/0;}
        sakiri skill;
        if !.sakiri.defeat_bonus.active || .sakiri.mods.defeat_bonus!=15-.time {let assertion_failure=1/0;}
        wait_until(15);
        if .sakiri.mods.defeat_bonus || .sakiri.defeat_bonus.active {let assertion_failure=1/0;}");
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_managed_modifiers_distinguish_absence_from_unknown_owner_names() {
    for owner in ["zero", "sakiri"] {
        let prefix = format!(
            "options duration=1 crit=random;
            {owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active {owner};"
        );
        let known = if owner == "zero" {
            "support_atk"
        } else {
            "team_atk"
        };
        assert!(
            run(&format!(
                "{prefix} if .{owner}.mods.{known} {{let assertion_failure=1/0;}}"
            ))
            .report
            .execution_completed
        );
        for (field, error) in [
            ("mods.unknown", "Unknown timed modifier"),
            ("unknown.active", "Unknown state"),
        ] {
            let result = run(&format!("{prefix} wait(.{owner}.{field});"));
            let diagnostic = result.diagnostic.unwrap();
            assert!(diagnostic.message.contains(error), "{diagnostic:?}");
            assert!(diagnostic.loc.length > 0);
            assert!(!result.report.execution_completed);
        }
    }
}
