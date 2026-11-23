use super::*;

pub(crate) enum Task {
    Root {
        index: usize,
        effect: Effect,
    },
    Apply {
        index: usize,
        effect: Effect,
        origin: Option<ActionOrigin>,
    },
    Hit {
        index: usize,
        target: String,
        amount: u64,
        origin: Option<ActionOrigin>,
    },
}
