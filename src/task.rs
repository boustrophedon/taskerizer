#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub task: String,
    pub priority: u32,
    pub reward: bool,
}
