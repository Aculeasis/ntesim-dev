use super::*;

pub(crate) const EXAMPLE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/synthetic-state.json"
));

pub(crate) fn observation_fixture() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/synthetic-cycles.json"
    )))
    .unwrap();
    let attack = value["actions"]["attack"][0].clone();
    let observe = value["actions"]["observe"][0].clone();
    value.as_object_mut().unwrap().remove("actions");
    let mut commands = vec![json!({"clock":0,"delay_us":0,"effect":observe})];
    for start in [1, 11, 21] {
        commands.push(json!({"clock":0,"delay_us":start,"effect":attack}));
        commands.push(json!({"clock":0,"delay_us":start+9,"effect":observe}));
    }
    value["commands"] = json!(commands);
    value
}

pub(crate) fn random_fixture() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/synthetic-random.json"
    )))
    .unwrap();
    let effect = value["actions"]["attack"][0].clone();
    value.as_object_mut().unwrap().remove("actions");
    value["commands"] = json!([
        {"clock":0,"delay_us":0,"effect":effect},
        {"clock":0,"delay_us":10,"effect":effect},
        {"clock":0,"delay_us":20,"effect":effect}
    ]);
    value
}

pub(crate) fn actor_fixture() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/synthetic-actions.json"
    )))
    .unwrap();
    let effect = value["actions"]["attack"][0].clone();
    value.as_object_mut().unwrap().remove("actions");
    value["commands"] = json!([
        {"clock":1,"delay_us":0,"effect":effect},
        {"clock":1,"delay_us":10,"effect":effect},
        {"clock":1,"delay_us":20,"effect":effect}
    ]);
    value
}

pub(crate) fn formula_fixture() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/synthetic-formula.json"
    )))
    .unwrap();
    let attack = value["actions"]["attack"][0].clone();
    value.as_object_mut().unwrap().remove("actions");
    value["commands"] = json!([{ "clock":0,"delay_us":10,"effect":attack }]);
    value
}
