use super::definition::{CliCommand, USAGE};

pub(crate) fn parse_command<'a>(args: &'a [String]) -> Result<CliCommand<'a>, String> {
    if args.is_empty() {
        return Err(USAGE.into());
    }
    let cli = crate::clap_app::build_cli();
    let matches = cli
        .try_get_matches_from(std::iter::once("ntesim".to_string()).chain(args.iter().cloned()))
        .map_err(|e| format!("{e}\n{USAGE}"))?;

    let (sub_name, sub_m) = matches.subcommand().ok_or(USAGE)?;
    let get_str = |id: &str| -> Option<&'a str> {
        let val = sub_m.get_one::<String>(id)?;
        args.iter().find(|&s| s == val).map(|s| s.as_str())
    };

    if let Some(cmd) = parse_synthetic_command(sub_name, sub_m, &get_str)? {
        return Ok(cmd);
    }
    if let Some(cmd) = parse_stats_command(sub_name, &get_str)? {
        return Ok(cmd);
    }
    if let Some(cmd) = parse_game_command(sub_name, &get_str)? {
        return Ok(cmd);
    }
    if let Some(cmd) = parse_package_command(sub_name, &get_str)? {
        return Ok(cmd);
    }
    Err(USAGE.into())
}

fn parse_synthetic_command<'a>(
    sub_name: &str,
    sub_m: &clap::ArgMatches,
    get_str: &impl Fn(&str) -> Option<&'a str>,
) -> Result<Option<CliCommand<'a>>, String> {
    let cmd = match sub_name {
        "run-synthetic" => CliCommand::RunSynthetic {
            input: get_str("fixture").ok_or(USAGE)?,
        },
        "run-rotation-synthetic" => {
            let seed = match sub_m.get_one::<String>("seed") {
                Some(s) => match s.parse::<u64>() {
                    Ok(val) => Some(val),
                    Err(_) => return Err("seed must be an unsigned 64-bit integer".into()),
                },
                None => None,
            };
            CliCommand::RunRotationSynthetic {
                input: get_str("fixture").ok_or(USAGE)?,
                rotation: get_str("rotation").ok_or(USAGE)?,
                seed,
            }
        }
        "batch-rotation-synthetic" => CliCommand::BatchRotationSynthetic {
            input: get_str("fixture").ok_or(USAGE)?,
            rotation: get_str("rotation").ok_or(USAGE)?,
            request: get_str("seeds").ok_or(USAGE)?,
        },
        "compare-rotation-synthetic" => CliCommand::CompareRotationSynthetic {
            left_input: get_str("base_json").ok_or(USAGE)?,
            left_source: get_str("base_nte").ok_or(USAGE)?,
            right_input: get_str("cand_json").ok_or(USAGE)?,
            right_source: get_str("cand_nte").ok_or(USAGE)?,
            request: get_str("seeds").ok_or(USAGE)?,
        },
        "search-rotation-synthetic" => CliCommand::SearchRotationSynthetic {
            input: get_str("fixture").ok_or(USAGE)?,
            request: get_str("search").ok_or(USAGE)?,
        },
        "check-cycles-synthetic" => CliCommand::CheckCyclesSynthetic {
            input: get_str("fixture").ok_or(USAGE)?,
            source: get_str("rotation").ok_or(USAGE)?,
            checks: get_str("checks").ok_or(USAGE)?,
        },
        _ => return Ok(None),
    };
    Ok(Some(cmd))
}

fn parse_stats_command<'a>(
    sub_name: &str,
    get_str: &impl Fn(&str) -> Option<&'a str>,
) -> Result<Option<CliCommand<'a>>, String> {
    let cmd = match sub_name {
        "project-studio-actor" => CliCommand::ProjectStudioActor {
            input: get_str("request").ok_or(USAGE)?,
        },
        "project-studio-actors" => CliCommand::ProjectStudioActors {
            input: get_str("requests").ok_or(USAGE)?,
        },
        "character-stats" => CliCommand::CharacterStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "weapon-stats" => CliCommand::WeaponStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "weapon-passive-values" => CliCommand::WeaponPassiveValues {
            input: get_str("request").ok_or(USAGE)?,
        },
        "damage-values" => CliCommand::DamageValues {
            input: get_str("request").ok_or(USAGE)?,
        },
        "build-stats" => CliCommand::BuildStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "team-stats" => CliCommand::TeamStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "combat-stats" => CliCommand::CombatStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "lifecycle-stats" => CliCommand::LifecycleStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "dispatch-stats" => CliCommand::DispatchStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "timing-stats" => CliCommand::TimingStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "reaction-stats" => CliCommand::ReactionStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        "execution-stats" => CliCommand::ExecutionStats {
            input: get_str("request").ok_or(USAGE)?,
        },
        _ => return Ok(None),
    };
    Ok(Some(cmd))
}

fn parse_game_command<'a>(
    sub_name: &str,
    get_str: &impl Fn(&str) -> Option<&'a str>,
) -> Result<Option<CliCommand<'a>>, String> {
    let cmd = match sub_name {
        "batch-script" => CliCommand::BatchScript {
            source: get_str("source").ok_or(USAGE)?,
            request: get_str("request").ok_or(USAGE)?,
        },
        "export-script" => CliCommand::ExportScript {
            source: get_str("source").ok_or(USAGE)?,
            output: get_str("output").ok_or(USAGE)?,
        },
        "replay-script" => CliCommand::ReplayScript {
            input: get_str("input").ok_or(USAGE)?,
        },
        "search-scripts" => CliCommand::SearchScripts {
            request: get_str("request").ok_or(USAGE)?,
            checkpoint: get_str("checkpoint"),
        },
        "compare-scripts" => CliCommand::CompareScripts {
            baseline: get_str("baseline").ok_or(USAGE)?,
            candidate: get_str("candidate").ok_or(USAGE)?,
            request: get_str("request").ok_or(USAGE)?,
        },
        "compile-script" | "validate-script" => CliCommand::CompileScript {
            source: get_str("source").ok_or(USAGE)?,
            validate_only: sub_name == "validate-script",
        },
        "run-script" => CliCommand::RunScript {
            source: get_str("source").ok_or(USAGE)?,
            stop_after_inputs: get_str("stop-after-inputs"),
            resume: get_str("resume"),
            cancel_file: get_str("cancel-file"),
        },
        "run-compiled-script" => CliCommand::RunCompiledScript {
            input: get_str("input").ok_or(USAGE)?,
        },
        "inspect-script" => CliCommand::InspectScript {
            source: get_str("source").ok_or(USAGE)?,
        },
        "run-game" => CliCommand::RunGame {
            input: get_str("scenario").ok_or(USAGE)?,
        },
        "run-game-program" => CliCommand::RunGameProgram {
            input: get_str("program").ok_or(USAGE)?,
        },
        "run-game-suite" => CliCommand::RunGameSuite {
            input: get_str("scenarios").ok_or(USAGE)?,
        },
        "run-game-built" => CliCommand::RunGameBuilt {
            input: get_str("request").ok_or(USAGE)?,
        },
        "batch-game" => CliCommand::BatchGame {
            input: get_str("scenario").ok_or(USAGE)?,
            batch: get_str("batch").ok_or(USAGE)?,
        },
        "batch-game-built" => CliCommand::BatchGameBuilt {
            input: get_str("build_request").ok_or(USAGE)?,
            batch: get_str("batch").ok_or(USAGE)?,
        },
        _ => return Ok(None),
    };
    Ok(Some(cmd))
}

fn parse_package_command<'a>(
    sub_name: &str,
    get_str: &impl Fn(&str) -> Option<&'a str>,
) -> Result<Option<CliCommand<'a>>, String> {
    let cmd = match sub_name {
        "studio-build-catalog" => CliCommand::StudioBuildCatalog,
        "published-variants" => CliCommand::PublishedVariants {
            input: get_str("variants"),
        },
        "built-variants" => {
            let first = get_str("variants_or_request").ok_or(USAGE)?;
            let second = get_str("request");
            let (csv_path, input) = match second {
                Some(req) => (Some(first), req),
                None => (None, first),
            };
            CliCommand::BuiltVariants { csv_path, input }
        }
        "profile-audit" => CliCommand::ProfileAudit {
            input: get_str("variants"),
        },
        "recommended-gear" => CliCommand::RecommendedGear,
        "search-builds" => CliCommand::SearchBuilds {
            input: get_str("variants"),
        },
        "compositions" => CliCommand::Compositions {
            variants_csv: get_str("variants"),
        },
        "acceptance-spec" => CliCommand::AcceptanceSpec {
            out_dir: get_str("output_dir"),
        },
        "validate-finalist" => {
            let first = get_str("variants_or_request").ok_or(USAGE)?;
            let second = get_str("request");
            let (variants_csv, input) = match second {
                Some(req) => (Some(first), req),
                None => (None, first),
            };
            CliCommand::ValidateFinalist {
                variants_csv,
                input,
            }
        }
        "verify-tabular" => CliCommand::VerifyTabular {
            fixture_path: get_str("fixture"),
        },
        "pilot" => CliCommand::Pilot {
            input: get_str("request"),
            checkpoint: get_str("checkpoint"),
        },
        "acceptance-run" => CliCommand::AcceptanceRun {
            input: get_str("request"),
            checkpoint: get_str("checkpoint"),
            variants_csv: get_str("variants"),
            conditions_json: get_str("conditions"),
        },
        _ => return Ok(None),
    };
    Ok(Some(cmd))
}
