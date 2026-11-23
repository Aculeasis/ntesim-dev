use super::*;
use crate::dsl::number::ExactNumber;
use std::cell::Cell;

struct Host {
    time: u64,
    energy: u64,
    reads: Cell<usize>,
}

impl Host {
    fn new() -> Self {
        Self {
            time: 0,
            energy: 0,
            reads: Cell::new(0),
        }
    }
}

impl ExpressionContext for Host {
    fn identifier(&self, _: &str) -> Result<Value, String> {
        panic!("variables belong to the rotation")
    }
    fn field(&self, parts: &[String]) -> Result<Value, String> {
        self.reads.set(self.reads.get() + 1);
        let value = match parts
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["time"] => {
                ExactNumber::from(self.time as i64).divide(&ExactNumber::from(1_000_000))?
            }
            ["actor", "energy"] => ExactNumber::from(self.energy as i64),
            _ => return Err("Unknown live field.".into()),
        };
        Ok(Value::Number(value))
    }
}

fn script(rotation: &str) -> Script {
    parse_nte(&format!("options duration=20; {rotation}")).unwrap()
}

fn integer(value: &Value) -> u64 {
    value.number().unwrap().to_safe_integer().unwrap()
}

fn waits(rotation: &mut Rotation<'_>, host: &Host) -> Vec<u64> {
    let mut values = Vec::new();
    while let Some(occurrence) = rotation.next(host).unwrap() {
        let Input::Wait(value) = occurrence.input else {
            panic!("expected wait")
        };
        values.push(integer(&value));
    }
    values
}

#[test]
fn suspension_preserves_exact_values_and_samples_the_new_host_state() {
    let script = script(
        "let x=0.1+0.2; wait(1); if x==0.3 && .actor.energy>=10 {wait(.time);} else {wait(99);}",
    );
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 100, 10);
    let mut host = Host::new();
    assert_eq!(
        integer(&match rotation.next(&host).unwrap().unwrap().input {
            Input::Wait(value) => value,
            _ => unreachable!(),
        }),
        1
    );
    assert_eq!(host.reads.get(), 0);
    host.time = 2_000_000;
    host.energy = 10;
    assert_eq!(waits(&mut rotation, &host), [2]);
    assert_eq!(host.reads.get(), 2);
}

#[test]
fn nested_loop_control_unwinds_body_scopes_and_runs_for_updates() {
    let script = script(
        "let n=7; for let i=0; i<4; i=i+1 { let n=i; if i==1 {let x=0; continue;} repeat 3 {if n==2 {break;} wait(n);} } wait(n);",
    );
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 1000, 30);
    assert_eq!(waits(&mut rotation, &Host::new()), [0, 0, 0, 3, 3, 3, 7]);
}

#[test]
fn repeat_count_is_captured_once_and_body_scope_is_new_each_iteration() {
    let script =
        script("let n=3; repeat n {let x=n; n=n-1; if n==1 {continue;} wait(x);} wait(n);");
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 200, 10);
    assert_eq!(waits(&mut rotation, &Host::new()), [3, 1, 0]);
}

#[test]
fn live_while_and_energy_wait_compose_without_replaying_prior_work() {
    let script = script(
        "let n=0; while .time<3 { wait_for_energy(10+n); if .actor.energy>n {wait(1);} n=n+1;} wait(n);",
    );
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 1000, 20);
    let mut host = Host::new();
    for n in 0..3 {
        let occurrence = rotation.next(&host).unwrap().unwrap();
        assert_eq!(occurrence.index, n as usize * 2);
        assert!(
            matches!(occurrence.input, Input::WaitForEnergy(ref value) if integer(value)==10+n)
        );
        host.energy = 10 + n;
        assert!(
            matches!(rotation.next(&host).unwrap().unwrap().input, Input::Wait(ref value) if integer(value)==1)
        );
        host.time += 1_000_000;
    }
    assert_eq!(waits(&mut rotation, &host), [3]);
}

#[test]
fn short_circuits_do_not_read_unavailable_fields_and_zero_is_false() {
    let script = script(
        "if 0 && .missing {wait(99);} if 1 || .missing {wait(2);} if 0.1+0.2-0.3 {wait(99);} wait(3);",
    );
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 100, 10);
    let host = Host::new();
    assert_eq!(waits(&mut rotation, &host), [2, 3]);
    assert_eq!(host.reads.get(), 0);
}

#[test]
fn action_occurrences_keep_ranges_and_evaluate_params_after_each_resume() {
    let script = script("mint dash[critical=true]:2, attack; wait_until(4);");
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 100, 10);
    let host = Host::new();
    let first = rotation.next(&host).unwrap().unwrap();
    let second = rotation.next(&host).unwrap().unwrap();
    assert_eq!((first.index, second.index), (0, 1));
    assert_eq!(first.loc, second.loc);
    assert!(first.loc.length > 0);
    assert!(
        matches!(first.input, Input::Action { actor: "mint", action: "dash", ref params } if params == &[("critical", Value::Boolean(true))])
    );
    assert!(matches!(
        rotation.next(&host).unwrap().unwrap().input,
        Input::Action {
            action: "attack",
            ..
        }
    ));
    assert!(
        matches!(rotation.next(&host).unwrap().unwrap().input, Input::WaitUntil(ref value) if integer(value)==4)
    );
    assert!(rotation.next(&host).unwrap().is_none());
}

#[test]
fn empty_loops_and_zero_time_yields_are_bounded_at_the_authored_range() {
    let bounded = script("repeat 21000 {}");
    let limits = compiler::CompileLimits::default();
    let mut rotation = Rotation::new(
        &bounded.statements,
        &Bindings::default(),
        limits.max_steps,
        limits.max_actions,
    );
    assert!(rotation.next(&Host::new()).unwrap().is_none());
    assert!(rotation.steps() > 20_000);

    for source in [
        "while 1 {}",
        "repeat 9007199254740991 {}",
        "while 1 {continue;}",
        "for let i=0; 1; i=i+1 {}",
    ] {
        let script = script(source);
        let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 40, 100);
        let error = rotation.next(&Host::new()).unwrap_err();
        assert!(error.message.contains("budget"), "{error:?}");
        assert!(error.message.contains("limit 40 operations"), "{error:?}");
        assert!(error.loc.length > 0);
    }
    let script = script("while 1 {wait(0);}");
    let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 1000, 3);
    for _ in 0..3 {
        assert!(rotation.next(&Host::new()).unwrap().is_some());
    }
    let error = rotation.next(&Host::new()).unwrap_err();
    assert!(error.message.contains("input budget"));
    assert!(error.message.contains("limit 3 inputs"));
}

#[test]
fn exact_fractional_counts_and_arithmetic_fail_at_the_expression() {
    for source in [
        "repeat (1-0.0000000000000000001) {wait(1);}",
        "let x=1/0;",
        "let x=true;",
        "const x=1; x=2;",
    ] {
        let script = script(source);
        let mut rotation = Rotation::new(&script.statements, &Bindings::default(), 100, 10);
        let error = rotation.next(&Host::new()).unwrap_err();
        assert!(error.loc.offset > 0);
        assert!(error.loc.length > 0);
    }
}
