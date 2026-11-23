use super::*;

#[test]
fn eternal_waltz_native_hp_ratio_is_added_against_base_hp_not_derived_hp() {
    let (tables, _, request) = fireflies_fixture();
    let mut actor = request.scenario.actors[0].clone();
    actor.character_id = "character:1039".into();
    actor.weapon_id = "weapon:fork_Arachne".into();
    actor.weapon_tier = 1;

    let native = crate::weapons::eternal_waltz::projection::compile_native(
        &tables,
        1,
        "ntesim-current",
        &actor,
    )
    .unwrap();
    assert_eq!(native["weapon_id"], "weapon:fork_Arachne");
    assert_eq!(
        receipt(&native["parameters"][0]["selected"]["value"]).unwrap(),
        ExactFraction::new(1, 5).unwrap()
    );

    let base_hp = ExactFraction::new(10_000, 1).unwrap();
    let derived_hp_with_other_hp_percent = ExactFraction::new(12_000, 1).unwrap();
    let projected = crate::weapons::eternal_waltz::projection::add_base_hp_ratio(
        derived_hp_with_other_hp_percent,
        base_hp,
        ExactFraction::new(1, 5).unwrap(),
    )
    .unwrap();
    assert_eq!(projected, ExactFraction::new(14_000, 1).unwrap());
    assert_ne!(projected, ExactFraction::new(14_400, 1).unwrap());

    for (tier, ratio, expected) in [(1, (1, 5), 12_000), (5, (8, 25), 13_200)] {
        let projected = crate::weapons::eternal_waltz::projection::add_base_hp_ratio(
            base_hp,
            base_hp,
            ExactFraction::new(ratio.0, ratio.1).unwrap(),
        )
        .unwrap();
        assert_eq!(
            projected.round_u64().unwrap(),
            expected,
            "refinement {tier}"
        );
    }
}
