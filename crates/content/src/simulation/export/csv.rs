use super::SourceReportExport;
use serde::Serialize;
use std::collections::BTreeMap;

fn json(value: &impl Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

fn row(output: &mut String, cells: &[String]) {
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            output.push(',');
        }
        output.push('"');
        output.push_str(&cell.replace('"', "\"\""));
        output.push('"');
    }
    output.push_str("\r\n");
}

struct Table<'a> {
    export: &'a SourceReportExport,
    text: String,
}

impl<'a> Table<'a> {
    fn new(export: &'a SourceReportExport, columns: &[&str]) -> Self {
        let mut text = String::new();
        row(
            &mut text,
            &["source_sha256", "request_sha256", "seed", "input_sha256"]
                .iter()
                .chain(columns)
                .map(|v| (*v).into())
                .collect::<Vec<_>>(),
        );
        Self { export, text }
    }

    fn push(&mut self, cells: Vec<String>) {
        let mut all = vec![
            self.export.source_sha256.clone(),
            self.export.request_sha256.clone(),
            self.export.execution.report.seed.to_string(),
            self.export.input_sha256.clone(),
        ];
        all.extend(cells);
        row(&mut self.text, &all);
    }
}

fn damage(export: &SourceReportExport) -> Result<String, String> {
    let mut table = Table::new(
        export,
        &[
            "hit_index",
            "time_us",
            "actor_index",
            "active_actor_index",
            "target_index",
            "ability_id",
            "damage_effect_id",
            "exact_numerator",
            "exact_denominator",
            "rounded_damage",
            "applied_hp_loss",
            "overkill",
            "receipt_json",
        ],
    );
    for (i, hit) in export.execution.report.hit_receipts.iter().enumerate() {
        table.push(vec![
            i.to_string(),
            hit.time_us.to_string(),
            hit.actor_index.to_string(),
            hit.active_actor_index.to_string(),
            hit.target_index.to_string(),
            hit.ability_id.clone(),
            hit.damage_effect_id.clone(),
            hit.damage.numerator.to_string(),
            hit.damage.denominator.to_string(),
            hit.rounded_damage.to_string(),
            hit.applied_hp_loss.to_string(),
            hit.rounded_damage
                .checked_sub(hit.applied_hp_loss)
                .ok_or("invalid damage settlement")?
                .to_string(),
            json(hit)?,
        ]);
    }
    Ok(table.text)
}

fn contributions(export: &SourceReportExport) -> Result<String, String> {
    let mut table = Table::new(
        export,
        &[
            "actor_index",
            "target_index",
            "ability_id",
            "damage_effect_id",
            "on_field",
            "hits",
            "damage",
            "applied_hp_loss",
            "overkill",
            "dps_json",
        ],
    );
    for c in &export.damage.contributions {
        let t = &c.totals;
        table.push(vec![
            c.actor_index.to_string(),
            c.target_index.to_string(),
            c.ability_id.clone(),
            c.damage_effect_id.clone(),
            c.on_field.to_string(),
            t.hits.to_string(),
            t.exact.numerator.to_string(),
            t.applied_hp_loss.to_string(),
            t.overkill.to_string(),
            json(&t.dps)?,
        ]);
    }
    Ok(table.text)
}

pub(super) fn tables(
    export: &SourceReportExport,
) -> Result<BTreeMap<&'static str, String>, String> {
    let report = &export.execution.report;
    let mut tables = BTreeMap::from([
        ("damage.csv", damage(export)?),
        ("contributions.csv", contributions(export)?),
    ]);
    let mut timeline = Table::new(
        export,
        &[
            "frame_index",
            "time_us",
            "boundary_json",
            "active_actor_index",
            "hits_end",
            "healing_end",
            "trace_end",
            "state_json",
        ],
    );
    for (i, frame) in report.state_timeline.iter().enumerate() {
        timeline.push(vec![
            i.to_string(),
            frame.time_us.to_string(),
            json(&frame.boundary)?,
            frame.active_actor_index.to_string(),
            frame.hits_end.to_string(),
            frame.healing_end.to_string(),
            frame.trace_end.to_string(),
            json(frame)?,
        ]);
    }
    tables.insert("timeline.csv", timeline.text);
    let mut healing = Table::new(
        export,
        &[
            "receipt_index",
            "time_us",
            "source_actor_index",
            "recipient_actor_index",
            "gross",
            "applied",
            "excess",
            "hp_before",
            "hp_after",
            "receipt_json",
        ],
    );
    for (i, h) in report.healing_receipts.iter().enumerate() {
        healing.push(vec![
            i.to_string(),
            h.time_us.to_string(),
            h.source_actor_index.to_string(),
            h.recipient_actor_index.to_string(),
            h.gross.to_string(),
            h.applied.to_string(),
            h.excess.to_string(),
            h.hp_before.to_string(),
            h.hp_after.to_string(),
            json(h)?,
        ]);
    }
    tables.insert("healing.csv", healing.text);
    let mut energy = Table::new(export, &["trace_index", "time_us", "receipt_json"]);
    for (i, e) in report.trace.iter().enumerate() {
        if let Some(spend) = e.energy_spend() {
            energy.push(vec![i.to_string(), e.time_us.to_string(), json(spend)?]);
        }
        for grant in e.energy_grants() {
            energy.push(vec![i.to_string(), e.time_us.to_string(), json(&grant)?]);
        }
    }
    tables.insert("energy.csv", energy.text);
    let mut inputs = Table::new(
        export,
        &[
            "interval_index",
            "start_us",
            "end_us",
            "duration_us",
            "actor_index",
            "cause_json",
            "interval_json",
        ],
    );
    for (i, interval) in report.input_time.intervals.iter().enumerate() {
        inputs.push(vec![
            i.to_string(),
            interval.start_us.to_string(),
            interval.end_us.to_string(),
            interval.duration_us.to_string(),
            interval.actor_index.to_string(),
            json(&interval.cause)?,
            json(interval)?,
        ]);
    }
    tables.insert("input_time.csv", inputs.text);
    let mut builds = Table::new(
        export,
        &[
            "actor_index",
            "character_id",
            "weapon_json",
            "set_json",
            "static_stats_json",
            "decomposition_json",
        ],
    );
    for a in &export.party.actors {
        builds.push(vec![
            a.slot.to_string(),
            a.character_id.clone(),
            json(&a.weapon)?,
            json(&a.set)?,
            json(&a.static_stats)?,
            json(&a.build_decomposition)?,
        ]);
    }
    tables.insert("builds.csv", builds.text);
    let mut assumptions = Table::new(export, &["scope", "id", "description"]);
    for (scope, values) in [
        ("declared", &report.model_assumptions),
        ("reached", &report.resolved_model_assumptions),
        ("callback", &report.callback_model_assumptions),
    ] {
        for (id, description) in values {
            assumptions.push(vec![scope.into(), id.clone(), description.clone()]);
        }
    }
    tables.insert("assumptions.csv", assumptions.text);
    Ok(tables)
}
