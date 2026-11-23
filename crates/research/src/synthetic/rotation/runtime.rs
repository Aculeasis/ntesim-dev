use super::*;

pub fn run(fixture_bytes: &[u8], source: &str) -> Result<Report, String> {
    prepare(fixture_bytes, source)?.run(None)
}

pub fn run_seeded(fixture_bytes: &[u8], source: &str, seed: u64) -> Result<Report, String> {
    prepare(fixture_bytes, source)?.run(Some(seed))
}

pub(crate) struct Prepared {
    pub(crate) input: synthetic::Fixture,
    pub(crate) identity: Vec<u8>,
    pub(crate) metadata: Metadata,
}

impl Prepared {
    pub(crate) fn has_resource(&self, id: &str) -> bool {
        self.input.resources.contains_key(id)
    }
    pub(crate) fn comparison_context(&self) -> serde_json::Value {
        let clocks: BTreeMap<_, _> = self
            .input
            .clocks
            .iter()
            .map(|c| (c.id, c.rate_ppm))
            .collect();
        let targets: BTreeMap<_, _> = self
            .input
            .targets
            .iter()
            .map(|(id, t)| {
                (
                    id,
                    serde_json::json!({"hp":t.hp,"max_hp":t.max_hp,"shield_order":t.shield_order}),
                )
            })
            .collect();
        serde_json::json!({"duration_us":self.input.duration_us,"mode_clock":self.input.mode_clock,"clocks":clocks,"targets":targets,"rng_streams":self.input.rng_streams})
    }

    pub(crate) fn run(&self, seed: Option<u64>) -> Result<Report, String> {
        let mut input = self.input.clone();
        let mut identity = self.identity.clone();
        let mut metadata = self.metadata.clone();
        if let Some(seed) = seed {
            input.seed = seed;
            metadata.seed_override = Some(seed);
            identity.extend_from_slice(b"seed-override-v1");
            identity.extend_from_slice(&seed.to_be_bytes());
        }
        Ok(Report {
            simulation: synthetic::execute(input, sha256(&identity))?,
            rotation: metadata,
        })
    }

    pub(crate) fn max_events(&self) -> Result<usize, String> {
        fn leaves(effects: &[Effect]) -> usize {
            effects
                .iter()
                .map(|effect| {
                    1 + usize::from(matches!(
                        effect,
                        Effect::Damage { .. }
                            | Effect::FormulaDamage { .. }
                            | Effect::RandomDamage { .. }
                    ))
                })
                .sum()
        }
        let mut count = 0_usize;
        for command in &self.input.commands {
            let children = match &command.effect {
                Effect::InvokeAction { actor, action } => {
                    let definition = self
                        .input
                        .actors
                        .get(actor)
                        .and_then(|a| a.actions.get(action))
                        .ok_or("unknown selected actor action")?;
                    if definition
                        .effects
                        .iter()
                        .any(|e| matches!(e, Effect::InvokeAction { .. }))
                    {
                        return Err("nested actor invocations are unsupported".into());
                    }
                    leaves(&definition.effects)
                }
                Effect::Damage { .. }
                | Effect::FormulaDamage { .. }
                | Effect::RandomDamage { .. } => 1,
                _ => 0,
            };
            count = count
                .checked_add(2)
                .and_then(|n| n.checked_add(children))
                .ok_or("batch event estimate overflow")?;
        }
        Ok(count)
    }
}

pub(crate) fn prepare(fixture_bytes: &[u8], source: &str) -> Result<Prepared, String> {
    let fixture: Fixture = serde_json::from_slice(fixture_bytes).map_err(|e| e.to_string())?;
    let compiled = compile(source, &fixture.actions, &fixture.clocks).map_err(|e| e.to_string())?;
    // Version/domain tag and byte lengths make the two-input identity unambiguous.
    let mut identity = LANGUAGE_VERSION.as_bytes().to_vec();
    for bytes in [fixture_bytes, source.as_bytes()] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    let input = synthetic::Fixture {
        schema_version: fixture.schema_version,
        content_id: fixture.content_id,
        data_status: fixture.data_status,
        seed: fixture.seed,
        rng_streams: fixture.rng_streams,
        duration_us: fixture.duration_us,
        mode_clock: fixture.mode_clock,
        clocks: fixture.clocks,
        targets: fixture.targets,
        resources: fixture.resources,
        actors: fixture.actors,
        commands: compiled.commands,
    };
    Ok(Prepared {
        input,
        identity,
        metadata: Metadata {
            language_version: LANGUAGE_VERSION.into(),
            fixture_sha256: sha256(fixture_bytes),
            source_sha256: sha256(source.as_bytes()),
            seed_override: None,
            origins: compiled.origins,
        },
    })
}
