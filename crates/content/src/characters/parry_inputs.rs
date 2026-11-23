//! Supported action families for an explicitly assumed parry at admission.
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(crate) enum Input {
    Dodge,
    Entry,
}

impl Input {
    pub(crate) fn source(action: &str) -> Result<Self, String> {
        match action {
            "dash" => Ok(Self::Dodge),
            "swap" => Ok(Self::Entry),
            _ => Err("parry requires an owner-supported Dodge or entry switch.".into()),
        }
    }
}

pub(crate) fn assumptions(
    character: &str,
    input: Input,
) -> Result<BTreeMap<String, String>, String> {
    use super::{blackbird, daffodil, haniel, hathor};
    let models: &[crate::HypothesisId] = match (character, input) {
        ("character:1019", Input::Dodge) => &[super::mint::clock::MODEL],
        ("character:1042", Input::Dodge) => &[
            blackbird::clock::MODEL,
            blackbird::secondary::MODEL,
            crate::HypothesisId::BlackbirdExplicitParryV1,
        ],
        ("character:1025", Input::Dodge) => &[hathor::secondary::MODEL],
        ("character:1020", Input::Dodge) => &[haniel::clock::MODEL, haniel::secondary::MODEL],
        ("character:1054", Input::Entry) => &[daffodil::phantom::MODEL],
        _ => return Err(format!("Unsupported parry action for {character}.")),
    };
    Ok(models
        .iter()
        .map(|id| ((*id).into(), id.description().into()))
        .collect())
}
