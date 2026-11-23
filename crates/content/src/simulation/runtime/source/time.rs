use super::Trial;
use crate::dsl::SourceLocation;
use crate::simulation::{GameTimeContext, GameTimeContextKind, GameTimeSource};

impl Trial<'_> {
    pub(super) fn begin_time_context(
        &mut self,
        context: GameTimeContextKind,
        loc: &SourceLocation,
    ) -> usize {
        let index = self.request.input_time_contexts.len();
        self.request.input_time_contexts.push(GameTimeContext {
            start_us: self.execution.state.current_time_us,
            end_us: self.request.absolute_limit_us,
            actor_index: self.execution.state.active_actor_index,
            source: GameTimeSource {
                occurrence_index: self.occurrences.last().expect("current occurrence").index,
                loc: loc.clone(),
            },
            context,
        });
        index
    }

    pub(super) fn end_time_context(&mut self, index: usize) {
        self.request.input_time_contexts[index].end_us = self.execution.state.current_time_us;
        if self.request.input_time_contexts[index].start_us == self.execution.state.current_time_us
        {
            self.request.input_time_contexts.pop();
        }
    }
}
