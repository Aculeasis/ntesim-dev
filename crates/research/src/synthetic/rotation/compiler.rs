use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct Origin {
    pub command_index: usize,
    pub action: String,
    pub effect_index: usize,
    pub position: Position,
    /// One-based outermost-to-innermost repeat iterations.
    pub iterations: Vec<u64>,
}

pub struct Compiled {
    pub commands: Vec<Command>,
    pub origins: Vec<Origin>,
}

pub(crate) struct Compiler<'a> {
    pub(crate) actions: &'a BTreeMap<String, Vec<Effect>>,
    pub(crate) cursors: BTreeMap<ClockId, u64>,
    pub(crate) selected: Option<ClockId>,
    pub(crate) work: usize,
    pub(crate) iterations: Vec<u64>,
    pub(crate) output: Compiled,
}

impl Compiler<'_> {
    pub(crate) fn expand(&mut self, nodes: &[Node]) -> Result<(), Error> {
        for node in nodes {
            self.work += 1;
            if self.work > MAX_WORK {
                return Err(error(node.position, "expanded statement limit exceeded"));
            }
            match &node.statement {
                Statement::Clock(id) => {
                    if !self.cursors.contains_key(id) {
                        return Err(error(node.position, format!("unknown clock {id}")));
                    }
                    self.selected = Some(*id);
                }
                Statement::Wait(delay) => {
                    let clock = self
                        .selected
                        .ok_or_else(|| error(node.position, "select a clock before wait"))?;
                    let cursor = self.cursors.get_mut(&clock).unwrap();
                    *cursor = cursor
                        .checked_add(*delay)
                        .ok_or_else(|| error(node.position, "timeline overflow"))?;
                }
                Statement::Do(action) => {
                    let clock = self
                        .selected
                        .ok_or_else(|| error(node.position, "select a clock before do"))?;
                    let effects = self
                        .actions
                        .get(action)
                        .ok_or_else(|| error(node.position, format!("unknown action {action}")))?;
                    if effects.is_empty() {
                        return Err(error(node.position, "action has no effects"));
                    }
                    if effects.len() > MAX_WORK - self.output.commands.len() {
                        return Err(error(node.position, "command limit exceeded"));
                    }
                    for (effect_index, effect) in effects.iter().enumerate() {
                        self.output.origins.push(Origin {
                            command_index: self.output.commands.len(),
                            action: action.clone(),
                            effect_index,
                            position: node.position,
                            iterations: self.iterations.clone(),
                        });
                        self.output.commands.push(Command {
                            clock,
                            delay_us: self.cursors[&clock],
                            effect: effect.clone(),
                        });
                    }
                }
                Statement::Repeat(count, body) => {
                    for iteration in 1..=*count {
                        self.iterations.push(iteration);
                        self.expand(body)?;
                        self.iterations.pop();
                    }
                }
            }
        }
        Ok(())
    }
}

/// Compile local-clock deadlines from zero; switching clocks keeps each cursor.
/// Source order determines tie order, not real-time sequencing between clocks.
pub fn compile(
    source: &str,
    actions: &BTreeMap<String, Vec<Effect>>,
    clocks: &[ClockInput],
) -> Result<Compiled, Error> {
    let mut parser = Parser {
        tokens: tokenize(source)?,
        next: 0,
        nodes: 0,
    };
    let nodes = parser.block(0)?;
    if !parser.peek().text.is_empty() {
        return Err(error(parser.peek().position, "unexpected closing brace"));
    }
    let start = Position { line: 1, column: 1 };
    let mut cursors = BTreeMap::new();
    for clock in clocks {
        if cursors.insert(clock.id, 0).is_some() {
            return Err(error(start, "duplicate clock"));
        }
    }
    let mut compiler = Compiler {
        actions,
        cursors,
        selected: None,
        work: 0,
        iterations: Vec::new(),
        output: Compiled {
            commands: Vec::new(),
            origins: Vec::new(),
        },
    };
    compiler.expand(&nodes)?;
    if compiler.output.commands.is_empty() {
        return Err(error(start, "rotation has no actions"));
    }
    Ok(compiler.output)
}
