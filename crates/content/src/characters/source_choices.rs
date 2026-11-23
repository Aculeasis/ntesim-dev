//! Model declarations for supported authored choices, shared by source adapters.
use super::{blackbird, chaos, chiz, daffodil, edgar, fadia, haniel, hathor};
use crate::HypothesisId;
use std::collections::BTreeMap;

/// Live execution initializes these owners even without an authored owner cast.
/// The frozen scheduled Studio adapter keeps its separate timing contract.
pub(crate) fn initial_assumptions(character: &str) -> BTreeMap<String, String> {
    match character {
        "character:1019" => super::mint::clock::source::assumptions(),
        "character:1036" => super::zankou::source_assumptions(),
        "character:1003" => super::sakiri::source_assumptions(),
        "character:1075" => super::iroi::source_assumptions(),
        _ => BTreeMap::new(),
    }
}

pub(crate) fn assumptions(character: &str, action: &str, held: bool) -> BTreeMap<String, String> {
    let models: &[HypothesisId] = match (character, action, held) {
        ("character:1036", "attack" | "dash" | "plunge" | "jump", _) => {
            &[super::zankou::forms::MODEL]
        }
        ("character:1019", "attack" | "plunge", true) => {
            &[super::mint::clock::MODEL, super::mint::hold::HYPOTHESIS_ID]
        }
        ("character:1019", "dash" | "plunge" | "jump", _) => &[super::mint::clock::MODEL],
        ("character:1042", "attack", true) => &[
            blackbird::clock::MODEL,
            blackbird::malice::MODEL,
            blackbird::branches::MODEL,
        ],
        ("character:1042", "jump", _) => &[
            blackbird::clock::MODEL,
            blackbird::malice::MODEL,
            blackbird::secondary::MODEL,
            blackbird::cancel::MODEL,
        ],
        ("character:1071", "jump", _) => &[
            chaos::clock::MODEL,
            chaos::resources::MODEL,
            chaos::secondary::MODEL,
            chaos::cancel::JUMP,
        ],
        ("character:1010", "jump", _) => &[super::nanally::clock::MODEL],
        ("character:1071", "attack", true) => &[
            chaos::clock::MODEL,
            chaos::resources::MODEL,
            chaos::verdict::MODEL,
            chaos::cancel::COMBO,
            chaos::cancel::CONTINUATION,
        ],
        ("character:1054", "attack", true) => &[
            daffodil::clock::MODEL,
            daffodil::clock::cancel::MODEL,
            daffodil::secondary::MODEL,
            daffodil::held::MODEL,
        ],
        ("character:1020", "attack", true) => &[haniel::clock::MODEL, haniel::cannon::MODEL],
        ("character:1073", "attack", true) => &[
            chiz::clock::MODEL,
            chiz::pressed::MODEL,
            chiz::finance::MODEL,
            chiz::cancel::MODEL,
            chiz::secondary::MODEL,
            chiz::barren::MODEL,
        ],
        ("character:1042", "dash" | "plunge", _) => &[
            blackbird::clock::MODEL,
            blackbird::malice::MODEL,
            blackbird::secondary::MODEL,
        ],
        ("character:1071", "dash", _) => &[
            chaos::clock::MODEL,
            chaos::resources::MODEL,
            chaos::secondary::MODEL,
        ],
        ("character:1054", "dash" | "plunge" | "jump", _) => &[
            daffodil::clock::MODEL,
            daffodil::clock::cancel::MODEL,
            daffodil::secondary::MODEL,
        ],
        ("character:1020", "dash" | "plunge" | "jump", _) => {
            &[haniel::clock::MODEL, haniel::secondary::MODEL]
        }
        ("character:1073", "dash" | "plunge" | "jump", _) => &[
            chiz::clock::MODEL,
            chiz::pressed::MODEL,
            chiz::finance::MODEL,
            chiz::cancel::MODEL,
            chiz::secondary::MODEL,
        ],
        ("character:1025", "plunge", _) => &[hathor::air::MODEL],
        ("character:1075", "attack", true) => &[
            super::iroi::clock::MODEL,
            super::iroi::branch::HYPOTHESIS_ID,
        ],
        ("character:1075", "skill", true) => &[
            super::iroi::clock::MODEL,
            super::iroi::regression::HYPOTHESIS_ID,
        ],
        ("character:1075", "burst", _) => {
            &[super::iroi::clock::MODEL, super::iroi::dream::HYPOTHESIS_ID]
        }
        ("character:1075", "dash" | "plunge" | "jump", _) => &[
            super::iroi::clock::MODEL,
            super::iroi::riposte::HYPOTHESIS_ID,
            super::iroi::plunge::HYPOTHESIS_ID,
        ],
        _ => &[],
    };
    models
        .iter()
        .map(|id| ((*id).into(), id.description().into()))
        .collect()
}

pub(crate) fn approach(character: &str, action: &str) -> Result<(), String> {
    if character == "character:1019" && action == "attack" {
        Ok(())
    } else {
        Err("approach requires a supported Mint attack.".into())
    }
}

pub(crate) fn window_assumptions(
    character: &str,
    window: crate::simulation::SourceActionWindow,
) -> Result<BTreeMap<String, String>, String> {
    if window == crate::simulation::SourceActionWindow::Attack {
        return if character == "character:1052" {
            Ok(BTreeMap::from([(
                super::hotori::katana::MODEL.as_str().into(),
                super::hotori::katana::MODEL.description().into(),
            )]))
        } else {
            Err(format!("Unsupported attack input window for {character}."))
        };
    }
    if window == crate::simulation::SourceActionWindow::Hold {
        return if matches!(
            character,
            "character:1042" | "character:1071" | "character:1054" | "character:1036"
        ) {
            Ok(assumptions(character, "attack", true))
        } else {
            Err(format!("Unsupported hold input window for {character}."))
        };
    }
    super::player_inputs::validate(character, crate::simulation::PlayerInput::Jump).map_err(
        |_| {
            format!(
                "Unsupported {} input window for {character}.",
                window.label()
            )
        },
    )?;
    Ok(assumptions(character, "jump", false))
}

/// Full source recovery must also pass a later native next-Normal gate.
/// Hotori's fifth Katana stage is the owner case where Any recovery is earlier.
pub(crate) fn completion_deadline(
    actor: &crate::simulation::state::ActorRuntime,
    now_us: u64,
) -> u64 {
    super::hotori::katana::recovery_lock(actor, "GA_Jin_Melee", now_us)
        .map_or(actor.locked_until_us, |(deadline, _)| {
            actor.locked_until_us.max(deadline)
        })
}

/// Bind an already implemented Normal-family airborne row. Its owner still
/// derives landing contacts, recovery, resources and recipients from the profile.
pub(crate) fn plunge(
    character: &str,
    normal: &crate::build::PublicAction,
) -> Result<Option<crate::build::PublicAction>, String> {
    let packet = match character {
        "character:1019" => super::mint::plunge::PACKET,
        "character:1036" => super::zankou::secondary::PLUNGE,
        "character:1075" => super::iroi::plunge::PACKET,
        "character:1042" => blackbird::secondary::PLUNGE,
        "character:1073" => chiz::secondary::PLUNGE,
        "character:1054" => daffodil::secondary::PLUNGE,
        "character:1021" => edgar::secondary::PLUNGE,
        "character:1039" => fadia::secondary::PLUNGE,
        "character:1020" => haniel::secondary::PLUNGE,
        "character:1025" => hathor::air::PACKET,
        _ => return Ok(None),
    };
    let mut action = normal.clone();
    let crate::simulation::GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut action.command
    else {
        return Err("A source plunge requires an ordinary ability descriptor.".into());
    };
    *damage_effect_id = packet.into();
    action
        .model_assumptions
        .extend(assumptions(character, "plunge", false));
    Ok(Some(action))
}
