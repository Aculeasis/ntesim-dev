use super::*;

pub(crate) fn create_test_team() -> TeamLifecycle {
    let mut resources = BTreeMap::new();
    resources.insert("energy".to_string(), Pool::new(100, 100).unwrap());
    resources.insert("stamina".to_string(), Pool::new(200, 200).unwrap());

    TeamLifecycle::new(
        4,
        vec![
            "sakiri".to_string(),
            "lacrimosa".to_string(),
            "skia".to_string(),
            "nanally".to_string(),
        ],
        0,
        1_000_000,
        resources,
    )
    .unwrap()
}
