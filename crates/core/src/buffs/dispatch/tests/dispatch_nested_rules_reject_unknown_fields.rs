use super::*;

#[test]
fn dispatch_nested_rules_reject_unknown_fields() {
    assert!(
        serde_json::from_value::<IcdRule>(
            serde_json::json!({"rule":"timer_only","window_us":1,"windw_us":2})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<ProcPayload>(
            serde_json::json!({"kind":"grant_pool_resource","pool":"energy","amount":1,"amont":2})
        )
        .is_err()
    );
}
