use super::*;
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};

#[test]
fn buff_grant_builder_constructs_valid_grant() {
    let grant = BuffGrantBuilder::new(0, "test_buff", "test_source")
        .recipient(BuffRecipient::SelfActor)
        .duration_us(10_000_000)
        .stat("atk_percent", ExactFraction::new(1, 4).unwrap())
        .receipt(
            "test_event",
            GameEventPayload {
                reason: Some("test".into()),
                ..Default::default()
            },
        )
        .build();

    assert_eq!(grant.actor_index, 0);
    assert_eq!(grant.event_kind, "test_event");
    assert_eq!(
        grant.data.as_ref().and_then(|d| d.reason.as_deref()),
        Some("test")
    );
    assert_eq!(grant.declarations.len(), 1);

    let decl = &grant.declarations[0];
    assert_eq!(decl.id, "test_buff");
    assert_eq!(decl.source, "test_source");
    assert_eq!(decl.duration_us, 10_000_000);
    assert_eq!(decl.recipient, BuffRecipient::SelfActor);
    assert_eq!(decl.stacking_scope, BuffStackingScope::PerSource);
    assert_eq!(
        decl.stat_modifiers.get("atk_percent").copied(),
        Some(ExactFraction::new(1, 4).unwrap())
    );
}

#[test]
fn buff_grant_builder_handles_multiple_declarations() {
    let extra = TimedBuffDeclaration::reset(
        "extra_buff",
        "extra_source",
        BuffRecipient::TeamAll,
        5_000_000,
        BuffStackingScope::Shared,
        BTreeMap::new(),
    );

    let grant = BuffGrantBuilder::new(1, "main_buff", "main_source")
        .duration_us(5_000_000)
        .add_declaration(extra)
        .receipt("multi_event", GameEventPayload::default())
        .build();

    assert_eq!(grant.actor_index, 1);
    assert_eq!(grant.declarations.len(), 2);
    assert_eq!(grant.declarations[0].id, "main_buff");
    assert_eq!(grant.declarations[1].id, "extra_buff");
}

#[test]
fn universal_modifier_variants() {
    let modifier = UniversalModifier {
        stat: "crit_rate".into(),
        kind: ModifierKind::Additive,
        value: ExactFraction::new(1, 10).unwrap(),
        duration_us: Some(15_000_000),
        tags: vec!["combat".into()],
    };

    assert_eq!(modifier.stat, "crit_rate");
    assert_eq!(modifier.kind, ModifierKind::Additive);
    assert_eq!(modifier.duration_us, Some(15_000_000));
}
