//! Compile the configuration prefix of one monolithic NTE script.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub use super::actor::CompiledActor;
use super::{
    actor::ActorDraft,
    catalog::Catalog,
    combat, dummy,
    evaluate::{Bindings, Evaluator},
    hurt::{self, CompiledHurtDeclaration},
    incoming::{self, CompiledIncomingEvent},
    interrupt,
    options::{CompiledOptions, compile_options},
    target::compile_target,
    validation::{Diagnostic, DiagnosticCode, validate_bindings},
    *,
};
use crate::{
    HypothesisId, build::EquipmentProjection, simulation::GameActorCombatTransitionEvent,
    simulation::GameActorDamageEvent, simulation::GameActorInterruptEvent,
    simulation::GameDummyCombatEvent, simulation::GameIncomingDamageBasis,
    simulation::GameTargetConfig,
};

#[derive(Clone, Debug)]
pub struct CompiledConfiguration {
    pub options: CompiledOptions,
    pub actors: Vec<CompiledActor>,
    pub targets: Vec<GameTargetConfig>,
    pub active_actor_index: usize,
    pub model_assumptions: BTreeMap<String, String>,
    pub constants: Bindings,
    pub rotation: Vec<Statement>,
    pub incoming_damage: Vec<(GameActorDamageEvent, SourceLocation)>,
    pub incoming_dummy_sources: BTreeMap<usize, usize>,
    pub incoming_damage_basis: BTreeMap<usize, GameIncomingDamageBasis>,
    pub hurt_declarations: Vec<CompiledHurtDeclaration>,
    pub dummy_combat: Vec<GameDummyCombatEvent>,
    pub interrupt_events: Vec<GameActorInterruptEvent>,
    pub combat_transition_events: Vec<GameActorCombatTransitionEvent>,
    pub authored_incoming: Vec<CompiledIncomingEvent>,
}

pub type SeededIncoming = (
    Vec<GameActorDamageEvent>,
    BTreeMap<usize, usize>,
    BTreeMap<usize, GameIncomingDamageBasis>,
);

type AssembledDamage = (
    Vec<(GameActorDamageEvent, SourceLocation)>,
    BTreeMap<usize, usize>,
    BTreeMap<usize, GameIncomingDamageBasis>,
);

impl CompiledConfiguration {
    pub fn has_stochastic_hurt(&self) -> bool {
        self.hurt_declarations.iter().any(|d| d.is_stochastic())
    }

    pub fn sample_incoming_for_seed(&self, seed: u64) -> Result<SeededIncoming, String> {
        let sampled = hurt::sample_hurt_events(
            &self.hurt_declarations,
            seed,
            self.options.relative_duration_us,
        )?;
        let mut all = Vec::new();
        for (event, _) in sampled {
            all.push((event, None, None));
        }
        for inc in &self.authored_incoming {
            all.push((inc.event.clone(), inc.source_target, inc.basis));
        }
        all.sort_by_key(|(event, _, _)| event.timestamp_us);

        let mut events = Vec::with_capacity(all.len());
        let mut sources = BTreeMap::new();
        let mut bases = BTreeMap::new();
        for (i, (event, source_target, basis)) in all.into_iter().enumerate() {
            if let Some(target) = source_target {
                sources.insert(i, target);
            }
            if let Some(b) = basis {
                bases.insert(i, b);
            }
            events.push(event);
        }
        Ok((events, sources, bases))
    }
}

pub(crate) fn failure(message: String, loc: &SourceLocation) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::Configuration,
        message,
        loc: loc.clone(),
    }
}

pub fn compile_configuration(
    script: &Script,
    tables: &BTreeMap<String, Value>,
) -> Result<CompiledConfiguration, Vec<Diagnostic>> {
    let diagnostics = validate_bindings(script);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let options = compile_options(script)?;
    compile(script, tables, options).map_err(|error| vec![error])
}

fn compile(
    script: &Script,
    tables: &BTreeMap<String, Value>,
    options: CompiledOptions,
) -> Result<CompiledConfiguration, Diagnostic> {
    let start = script
        .options
        .first()
        .map(|o| o.loc.clone())
        .unwrap_or(SourceLocation {
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        });
    let fail = |message| failure(message, &start);
    let catalog = Catalog::new(tables).map_err(fail)?;
    let mut evaluator = Evaluator::default();
    let mut constants = Bindings::default();
    let mut drafts = Vec::<ActorDraft>::new();
    let mut targets = Vec::new();
    let mut active_actor_index = None;
    let mut rotation = Vec::new();
    let mut rotation_started = false;

    let mut authored_incoming = Vec::new();
    let mut hurt_declarations = Vec::new();
    let mut dummy_combat = Vec::new();
    let mut interrupt_events = Vec::new();
    let mut combat_transition_events = Vec::new();

    for item in script.items() {
        match item {
            ScriptItem::Statement(statement) => {
                if !rotation_started
                    && let StatementKind::Variable {
                        name,
                        is_const: true,
                        value,
                    } = &statement.kind
                {
                    let value = evaluator
                        .evaluate(value, &constants)
                        .map_err(Diagnostic::from)?;
                    value
                        .number()
                        .map_err(|message| failure(message, &statement.loc))?;
                    constants.0.insert(name.clone(), value);
                } else {
                    rotation_started = true;
                    rotation.push(statement.clone());
                }
            }
            ScriptItem::Declaration(declaration) => {
                if rotation_started {
                    return Err(failure(
                        "Configuration must precede rotation.".into(),
                        &declaration.loc,
                    ));
                }
                match &declaration.kind {
                    DeclarationKind::Incoming { .. } => {
                        let compiled = incoming::compile(
                            declaration,
                            &drafts,
                            targets.len(),
                            &constants,
                            &mut evaluator,
                        )?;
                        authored_incoming.push(compiled);
                        continue;
                    }
                    DeclarationKind::Hurt { .. } => {
                        let compiled =
                            hurt::compile(declaration, &drafts, &constants, &mut evaluator)?;
                        hurt_declarations.push(compiled);
                        continue;
                    }
                    DeclarationKind::Dummy { .. } => {
                        let compiled = dummy::compile(
                            declaration,
                            &drafts,
                            targets.len(),
                            &constants,
                            &mut evaluator,
                        )?;
                        dummy_combat.push(compiled);
                        continue;
                    }
                    DeclarationKind::Interrupt { .. } => {
                        let compiled =
                            interrupt::compile(declaration, &drafts, &constants, &mut evaluator)?;
                        interrupt_events.push(compiled);
                        continue;
                    }
                    DeclarationKind::Combat { .. } => {
                        let compiled =
                            combat::compile(declaration, &drafts, &constants, &mut evaluator)?;
                        combat_transition_events.push(compiled);
                        continue;
                    }
                    _ => {}
                }
                apply_declaration(
                    declaration,
                    &catalog,
                    &constants,
                    &mut evaluator,
                    &mut drafts,
                    &mut targets,
                    &mut active_actor_index,
                )?;
            }
        }
    }
    if drafts.is_empty() {
        return Err(fail("Declare at least one character.".into()));
    }
    if targets.is_empty() {
        return Err(fail("Declare at least one target.".into()));
    }
    let active_actor_index = active_actor_index
        .ok_or_else(|| fail("Declare active <character> before rotation.".into()))?;

    let (actors, mut model_assumptions) = finish_actors(drafts, tables)?;

    let (incoming_damage, incoming_dummy_sources, incoming_damage_basis) =
        assemble_initial_damage(&hurt_declarations, &authored_incoming, &options, &start)?;

    lower_declarations_assumptions(
        &actors,
        &interrupt_events,
        &combat_transition_events,
        &dummy_combat,
        &incoming_damage_basis,
        !incoming_damage.is_empty(),
        &mut model_assumptions,
    );

    dummy_combat.sort_by_key(|e| e.timestamp_us());
    interrupt_events.sort_by_key(|e| e.timestamp_us);
    combat_transition_events.sort_by_key(|e| e.timestamp_us);

    Ok(CompiledConfiguration {
        options,
        actors,
        targets,
        active_actor_index,
        model_assumptions,
        constants,
        rotation,
        incoming_damage,
        incoming_dummy_sources,
        incoming_damage_basis,
        hurt_declarations,
        dummy_combat,
        interrupt_events,
        combat_transition_events,
        authored_incoming,
    })
}

fn finish_actors(
    drafts: Vec<ActorDraft>,
    tables: &BTreeMap<String, Value>,
) -> Result<(Vec<CompiledActor>, BTreeMap<String, String>), Diagnostic> {
    let mut model_assumptions = BTreeMap::new();
    let mut actors = Vec::new();
    let mut identities = BTreeSet::new();
    for draft in drafts {
        let loc = draft.loc.clone();
        let compiled = draft.finish(tables)?;
        if !identities.insert(compiled.actor.character_id.clone()) {
            return Err(failure(
                "A character cannot occupy multiple team slots through different aliases.".into(),
                &loc,
            ));
        }
        merge_assumptions(
            &mut model_assumptions,
            &compiled.projection.model_assumptions,
        )
        .map_err(|message| failure(message, &loc))?;
        merge_assumptions(
            &mut model_assumptions,
            &crate::registry::initial_resource_assumptions(&compiled.actor.initial_resources),
        )
        .map_err(|message| failure(message, &loc))?;
        if let EquipmentProjection::Supported {
            model_assumptions: equipment,
            ..
        } = &compiled.projection.equipment_projection
        {
            merge_assumptions(&mut model_assumptions, equipment)
                .map_err(|message| failure(message, &loc))?;
        }
        actors.push(compiled);
    }
    Ok((actors, model_assumptions))
}

fn assemble_initial_damage(
    hurt_declarations: &[CompiledHurtDeclaration],
    authored_incoming: &[CompiledIncomingEvent],
    options: &CompiledOptions,
    loc: &SourceLocation,
) -> Result<AssembledDamage, Diagnostic> {
    let sampled_hurt = hurt::sample_hurt_events(
        hurt_declarations,
        options.seed,
        options.relative_duration_us,
    )
    .map_err(|msg| failure(msg, loc))?;
    let mut all_damage = Vec::new();
    for (event, loc) in sampled_hurt {
        all_damage.push((event, None, None, loc));
    }
    for inc in authored_incoming {
        all_damage.push((
            inc.event.clone(),
            inc.source_target,
            inc.basis,
            inc.loc.clone(),
        ));
    }
    all_damage.sort_by_key(|(event, _, _, _)| event.timestamp_us);

    let mut incoming_damage = Vec::with_capacity(all_damage.len());
    let mut incoming_dummy_sources = BTreeMap::new();
    let mut incoming_damage_basis = BTreeMap::new();

    for (i, (event, source_target, basis, loc)) in all_damage.into_iter().enumerate() {
        if let Some(target) = source_target {
            incoming_dummy_sources.insert(i, target);
        }
        if let Some(b) = basis {
            incoming_damage_basis.insert(i, b);
        }
        incoming_damage.push((event, loc));
    }
    Ok((
        incoming_damage,
        incoming_dummy_sources,
        incoming_damage_basis,
    ))
}

fn lower_declarations_assumptions(
    actors: &[CompiledActor],
    interrupt_events: &[GameActorInterruptEvent],
    combat_transition_events: &[GameActorCombatTransitionEvent],
    dummy_combat: &[GameDummyCombatEvent],
    incoming_damage_basis: &BTreeMap<usize, GameIncomingDamageBasis>,
    has_incoming_damage: bool,
    model_assumptions: &mut BTreeMap<String, String>,
) {
    for event in interrupt_events {
        lower_interruption_assumptions(
            &actors[event.actor_index].actor.character_id,
            model_assumptions,
        );
    }
    for event in combat_transition_events {
        lower_combat_transition_assumptions(
            &actors[event.actor_index].actor.character_id,
            model_assumptions,
        );
    }
    if !dummy_combat.is_empty() {
        for actor in actors {
            if actor.actor.character_id == "character:1025" {
                let id = HypothesisId::HathorGazeV1;
                model_assumptions.insert(id.into(), id.description().into());
                let tap = HypothesisId::HathorTapPowerV1;
                model_assumptions.insert(tap.into(), tap.description().into());
            }
        }
    }
    if has_incoming_damage {
        let id = HypothesisId::CombatIncomingDamageFixtureV1;
        model_assumptions.insert(id.into(), id.description().into());
    }
    if incoming_damage_basis
        .values()
        .any(|b| *b == GameIncomingDamageBasis::BeforeDefense)
    {
        let id = HypothesisId::CombatIncomingDefenseV1;
        model_assumptions.insert(id.into(), id.description().into());
    }
}

fn lower_interruption_assumptions(
    character_id: &str,
    model_assumptions: &mut BTreeMap<String, String>,
) {
    let ids: &[HypothesisId] = match character_id {
        "character:1019" => &[HypothesisId::MintOrdinaryClockV1],
        "character:1010" => &[HypothesisId::NanallyActionClockV1],
        "character:1020" => &[HypothesisId::HanielOrdinaryClockV1],
        "character:1021" => &[HypothesisId::EdgarFullKitV1],
        "character:1023" => &[
            HypothesisId::BaicangForcedInterruptV1,
            HypothesisId::BaicangPowerWordsV1,
        ],
        "character:1025" => &[HypothesisId::HathorOrdinaryClockV1],
        "character:1039" => &[HypothesisId::FadiaOrdinaryClockV1],
        "character:1054" => &[
            HypothesisId::DaffodilActionCancelsV1,
            HypothesisId::DaffodilOrdinaryClockV1,
        ],
        "character:1071" => &[
            HypothesisId::ChaosOrdinaryClockV1,
            HypothesisId::ChaosForcedInterruptV1,
        ],
        "character:1075" => &[HypothesisId::IroiActionClockV1],
        _ => &[],
    };
    for id in ids {
        model_assumptions.insert((*id).into(), id.description().into());
    }
}

fn lower_combat_transition_assumptions(
    character_id: &str,
    model_assumptions: &mut BTreeMap<String, String>,
) {
    let ids: &[HypothesisId] = match character_id {
        "character:1010" => &[HypothesisId::NanallyCombatLifecycleV1],
        "character:1020" => &[HypothesisId::HanielNovaExtractionV1],
        "character:1021" => &[HypothesisId::EdgarFullKitV1],
        "character:1025" => &[
            HypothesisId::HathorCombatLifecycleV1,
            HypothesisId::HathorGazeV1,
            HypothesisId::HathorBurstEmergencyV1,
        ],
        "character:1039" => &[HypothesisId::FadiaFullKitV1],
        "character:1042" => &[HypothesisId::BlackbirdCombatTransitionsV1],
        "character:1073" => &[HypothesisId::ChizBattleEntryV1],
        _ => &[],
    };
    for id in ids {
        model_assumptions.insert((*id).into(), id.description().into());
    }
}

fn apply_declaration(
    declaration: &Declaration,
    catalog: &Catalog,
    constants: &Bindings,
    evaluator: &mut Evaluator,
    drafts: &mut Vec<ActorDraft>,
    targets: &mut Vec<GameTargetConfig>,
    active: &mut Option<usize>,
) -> Result<(), Diagnostic> {
    let fail = |message| failure(message, &declaration.loc);
    let owner = match &declaration.kind {
        DeclarationKind::Character { .. } => {
            crate::simulation::validate_party_size(drafts.len() + 1).map_err(fail)?;
            drafts.push(ActorDraft::new(
                declaration,
                drafts.len(),
                catalog,
                constants,
                evaluator,
            )?);
            return Ok(());
        }
        DeclarationKind::Target { .. } => {
            crate::simulation::validate_target_count(
                targets.len() + 1,
                crate::execution::ExecutionLimits::default().max_events,
            )
            .map_err(fail)?;
            targets.push(compile_target(
                declaration,
                targets.len(),
                constants,
                evaluator,
            )?);
            return Ok(());
        }
        DeclarationKind::Weapon { actor, .. }
        | DeclarationKind::Cartridge { actor, .. }
        | DeclarationKind::Stats { actor, .. }
        | DeclarationKind::Active { actor } => actor,
        DeclarationKind::Hurt { .. }
        | DeclarationKind::Incoming { .. }
        | DeclarationKind::Dummy { .. }
        | DeclarationKind::Interrupt { .. }
        | DeclarationKind::Combat { .. } => {
            return Ok(());
        }
    };
    let index = drafts
        .iter()
        .position(|draft| &draft.name == owner)
        .ok_or_else(|| fail(format!("Declare '{owner} char ...;' first.")))?;
    match &declaration.kind {
        DeclarationKind::Active { .. } => {
            if active.replace(index).is_some() {
                return Err(fail("Declare active exactly once.".into()));
            }
            Ok(())
        }
        DeclarationKind::Stats { pairs, .. } => drafts[index].stats(pairs, constants, evaluator),
        _ => drafts[index].equip(declaration, catalog),
    }
}

pub(crate) fn merge_assumptions(
    destination: &mut BTreeMap<String, String>,
    source: &BTreeMap<String, String>,
) -> Result<(), String> {
    for (id, description) in source {
        match destination.get(id) {
            Some(old) if old != description => {
                return Err(format!("Conflicting compiled model assumption: {id}"));
            }
            Some(_) => {}
            None => {
                destination.insert(id.clone(), description.clone());
            }
        }
    }
    Ok(())
}
