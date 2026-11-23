use super::{CallbackContext, ExactFraction};

/// Final source DEF shared by shield snapshot and live overflow mitigation.
pub(crate) fn source_def(
    request: &crate::simulation::GameScenarioRequest,
    source: usize,
    buffs: &std::collections::BTreeMap<String, crate::formula::Ratio>,
) -> Result<ExactFraction, String> {
    let config = request
        .actors
        .get(source)
        .ok_or("source DEF config missing")?;
    let passive = crate::gear::source_stats::def_percent(
        config,
        crate::gear::source_stats::team_has_three_elements(request)?,
    )?;
    let base = config
        .static_stats
        .base_def
        .unwrap_or(config.static_stats.def);
    crate::simulation::effective_def(config, buffs)?
        .add(ExactFraction::new(base.into(), 1)?.mul(passive)?)
}

impl CallbackContext<'_> {
    /// Sample the same base/static/live DEF zones used by owner shields and packets.
    pub(crate) fn live_def(&self, source: usize) -> Result<ExactFraction, String> {
        let living = self
            .state
            .actors
            .iter()
            .enumerate()
            .filter_map(|(i, a)| (a.hp > 0).then_some(i))
            .collect::<std::collections::BTreeSet<_>>();
        let buffs = self
            .state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(
                source,
                self.state.active_actor_index,
                self.state.mode_clock.relative_us(),
                &living,
            )
            .map_err(|e| format!("incoming source DEF query failed: {e:?}"))?;
        source_def(self.request, source, &buffs)
    }

    /// Final source ATK without inventing a packet, recipient or damage category.
    pub(crate) fn live_atk(&mut self, source: usize) -> Result<ExactFraction, String> {
        let living = self
            .state
            .actors
            .iter()
            .enumerate()
            .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
            .collect::<std::collections::BTreeSet<_>>();
        let buffs = self
            .state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(
                source,
                self.state.active_actor_index,
                self.state.mode_clock.relative_us(),
                &living,
            )
            .map_err(|error| format!("source ATK modifier query failed: {error:?}"))?;
        let stat = |key| {
            buffs
                .get(key)
                .copied()
                .map(ExactFraction::from_ratio)
                .unwrap_or_else(|| ExactFraction::new(0, 1))
        };
        let config = self
            .request
            .actors
            .get(source)
            .ok_or("source ATK config missing")?;
        let actor = self
            .state
            .actors
            .get(source)
            .ok_or("source ATK state missing")?;
        let passive = crate::gear::source_stats::atk_percent(
            config,
            actor,
            self.state.current_time_us,
            self.state.team_has_three_elements()?,
        )?;
        let owner = crate::registry::source_atk_percent(
            self.request,
            config,
            actor,
            self.state.current_time_us,
        )?;
        let target_count = self
            .state
            .targets
            .iter()
            .filter(|target| target.hp_current > 0)
            .count();
        let mut team = super::HitModifiers::new(ExactFraction::new(0, 1)?)?;
        for (source_index, (source_config, source_actor)) in self
            .request
            .actors
            .iter()
            .zip(&self.state.actors)
            .enumerate()
        {
            team = crate::characters::team::modifiers(
                &super::RecipientContext {
                    source_index,
                    source_config,
                    source_actor,
                    team_actors: &self.state.actors,

                    time_us: self.state.current_time_us,
                    target_count,
                    packet: None,
                    request: Some(self.request),
                },
                team,
            )?;
        }
        ExactFraction::from_ratio(config.static_stats.atk)?
            .add(
                ExactFraction::new(config.static_stats.base_atk.into(), 1)?.mul(
                    stat("atk_percent")?
                        .add(passive)?
                        .add(team.atk_percent)?
                        .add(owner)?,
                )?,
            )?
            .add(stat("atk_add")?)
    }
}
