#[derive(Debug, Clone, PartialEq, Eq)]
/// Task representation. Priority must not be 0.
pub struct Task {
    pub task: String,
    pub priority: u32,
    pub reward: bool,
}

#[cfg(test)]
pub(crate) mod test_utils;
