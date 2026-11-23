//! Declarative CLI specification using clap.

use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("ntesim")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Universal combat and damage simulator for Neverness to Everness (NTE)")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("profile")
                .long("profile")
                .global(true)
                .value_name("NEW_JSON_PATH")
                .help("Save command/JSON-encoding measurements to a new file"),
        )
        .subcommands(synthetic_subcommands())
        .subcommands(stats_subcommands())
        .subcommands(gameplay_subcommands())
        .subcommands(package_subcommands())
}

fn synthetic_subcommands() -> Vec<Command> {
    vec![
        Command::new("run-synthetic")
            .about("Execute a synthetic event fixture")
            .arg(
                Arg::new("fixture")
                    .required(true)
                    .help("Path to synthetic FIXTURE.json"),
            ),
        Command::new("run-rotation-synthetic")
            .about("Execute a synthetic timeline rotation")
            .arg(
                Arg::new("fixture")
                    .required(true)
                    .help("Path to FIXTURE.json"),
            )
            .arg(
                Arg::new("rotation")
                    .required(true)
                    .help("Path to ROTATION.nte"),
            )
            .arg(
                Arg::new("seed")
                    .long("seed")
                    .value_name("U64")
                    .help("Optional random seed"),
            ),
        Command::new("batch-rotation-synthetic")
            .about("Execute a batch of synthetic rotation seeds")
            .arg(Arg::new("fixture").required(true))
            .arg(Arg::new("rotation").required(true))
            .arg(Arg::new("seeds").required(true)),
        Command::new("compare-rotation-synthetic")
            .about("Compare two synthetic rotations across seeds")
            .arg(Arg::new("base_json").required(true))
            .arg(Arg::new("base_nte").required(true))
            .arg(Arg::new("cand_json").required(true))
            .arg(Arg::new("cand_nte").required(true))
            .arg(Arg::new("seeds").required(true)),
        Command::new("search-rotation-synthetic")
            .about("Search synthetic rotation parameters")
            .arg(Arg::new("fixture").required(true))
            .arg(Arg::new("search").required(true)),
        Command::new("check-cycles-synthetic")
            .about("Check synthetic timeline cycles against constraints")
            .arg(Arg::new("fixture").required(true))
            .arg(Arg::new("rotation").required(true))
            .arg(Arg::new("checks").required(true)),
    ]
}

fn stats_subcommands() -> Vec<Command> {
    vec![
        Command::new("project-studio-actor")
            .about("Project static stats for a single actor")
            .arg(Arg::new("request").required(true)),
        Command::new("project-studio-actors")
            .about("Project static stats for multiple actors")
            .arg(Arg::new("requests").required(true)),
        Command::new("character-stats")
            .about("Compute character progression stats")
            .arg(Arg::new("request").required(true)),
        Command::new("weapon-stats")
            .about("Compute weapon base and scaling stats")
            .arg(Arg::new("request").required(true)),
        Command::new("weapon-passive-values")
            .about("Extract weapon passive parameter figures")
            .arg(Arg::new("request").required(true)),
        Command::new("damage-values")
            .about("Extract damage effect rows and parameters")
            .arg(Arg::new("request").required(true)),
        Command::new("build-stats")
            .about("Compile full character build stats")
            .arg(Arg::new("request").required(true)),
        Command::new("team-stats")
            .about("Compile team synergy and static buffs")
            .arg(Arg::new("request").required(true)),
        Command::new("combat-stats")
            .about("Inspect combat attribute calculations")
            .arg(Arg::new("request").required(true)),
        Command::new("lifecycle-stats")
            .about("Inspect actor lifecycle timeline")
            .arg(Arg::new("request").required(true)),
        Command::new("dispatch-stats")
            .about("Inspect event dispatching")
            .arg(Arg::new("request").required(true)),
        Command::new("timing-stats")
            .about("Inspect hit and ability delays")
            .arg(Arg::new("request").required(true)),
        Command::new("reaction-stats")
            .about("Inspect elemental reaction tables")
            .arg(Arg::new("request").required(true)),
        Command::new("execution-stats")
            .about("Inspect low-level execution engine stats")
            .arg(Arg::new("request").required(true)),
    ]
}

fn gameplay_subcommands() -> Vec<Command> {
    vec![
        Command::new("export-script")
            .about("Export exact native source reports and causal CSV tables into a new directory")
            .arg(Arg::new("source").required(true))
            .arg(Arg::new("output").required(true)),
        Command::new("replay-script")
            .about("Reconcile a saved report.json against source execution and native replay")
            .arg(Arg::new("input").required(true)),
        Command::new("batch-script")
            .about("Run an authored native script over explicit seeds and budgets")
            .arg(Arg::new("source").required(true))
            .arg(Arg::new("request").required(true)),
        Command::new("search-scripts")
            .about("Search explicit authored source variants with trial-boundary checkpoints")
            .arg(Arg::new("request").required(true))
            .arg(Arg::new("checkpoint")),
        Command::new("compare-scripts")
            .about("Compare native scripts on common seeds and compatible dummy assumptions")
            .arg(Arg::new("baseline").required(true))
            .arg(Arg::new("candidate").required(true))
            .arg(Arg::new("request").required(true)),
        Command::new("inspect-script")
            .about("Return the complete party and exact static stats without running a rotation")
            .arg(Arg::new("source").required(true)),
        Command::new("compile-script")
            .about("Compile an NTE script using the shared Rust frontend")
            .arg(Arg::new("source").required(true)),
        Command::new("run-script")
            .about("Execute one source trial using the authored seed and live combat state")
            .arg(Arg::new("source").required(true))
            .arg(Arg::new("stop-after-inputs").long("stop-after-inputs").help("Stop after N additional complete source inputs, retaining a replay checkpoint"))
            .arg(Arg::new("resume").long("resume").help("Resume a source checkpoint JSON on the same source and engine"))
            .arg(Arg::new("cancel-file").long("cancel-file").help("Cooperatively stop at the next source input boundary when this file exists")),
        Command::new("run-compiled-script")
            .about("Execute a saved native source compile response with matching identities")
            .arg(Arg::new("input").required(true)),
        Command::new("validate-script")
            .about("Return structured NTE source diagnostics")
            .arg(Arg::new("source").required(true)),
        Command::new("run-game")
            .about("Execute a single game scenario JSON")
            .arg(Arg::new("scenario").required(true)),
        Command::new("run-game-program")
            .about("Execute a scenario with dynamic live program conditions")
            .arg(Arg::new("program").required(true)),
        Command::new("run-game-suite")
            .about("Execute a suite of scenario requests")
            .arg(Arg::new("scenarios").required(true)),
        Command::new("run-game-built")
            .about("Compile a scenario from build request and execute")
            .arg(Arg::new("request").required(true)),
        Command::new("batch-game")
            .about("Execute a batch scenario across multiple seeds")
            .arg(Arg::new("scenario").required(true))
            .arg(Arg::new("batch").required(true)),
        Command::new("batch-game-built")
            .about("Execute a batch built scenario across multiple seeds")
            .arg(Arg::new("build_request").required(true))
            .arg(Arg::new("batch").required(true)),
    ]
}

fn package_subcommands() -> Vec<Command> {
    vec![
        Command::new("studio-build-catalog").about("Generate catalog for web studio"),
        Command::new("published-variants")
            .about("Enumerate published variants")
            .arg(Arg::new("variants").required(false)),
        Command::new("built-variants")
            .about("Enumerate built variants")
            .arg(Arg::new("variants_or_request").required(true))
            .arg(Arg::new("request").required(false)),
        Command::new("profile-audit")
            .about("Audit character build profiles")
            .arg(Arg::new("variants").required(false)),
        Command::new("recommended-gear").about("Query recommended gear"),
        Command::new("search-builds")
            .about("Search optimal build configurations")
            .arg(Arg::new("variants").required(false)),
        Command::new("compositions")
            .about("Enumerate valid party compositions")
            .arg(Arg::new("variants").required(false)),
        Command::new("acceptance-spec")
            .about("Generate acceptance test specification")
            .arg(Arg::new("output_dir").required(false)),
        Command::new("pilot")
            .about("Run research pilot evaluation")
            .arg(Arg::new("request").required(false))
            .arg(Arg::new("checkpoint").required(false)),
        Command::new("acceptance-run")
            .about("Execute full research acceptance pipeline")
            .arg(Arg::new("request").required(false))
            .arg(Arg::new("checkpoint").required(false))
            .arg(Arg::new("variants").required(false))
            .arg(Arg::new("conditions").required(false)),
        Command::new("validate-finalist")
            .about("Validate candidate finalist")
            .arg(Arg::new("variants_or_request").required(true))
            .arg(Arg::new("request").required(false)),
        Command::new("verify-tabular")
            .about("Verify tabular damage assertions")
            .arg(Arg::new("fixture").required(false)),
    ]
}
