use super::*;

#[test]
fn unknown_element_never_defaults_to_zero_bonus() {
    assert!(element_attribute("unknown").is_err());
    assert_eq!(
        element_attribute("ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE").unwrap(),
        "DamageUpNatureBase"
    );
}
