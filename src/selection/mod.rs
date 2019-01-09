use crate::task::{Category, Task};

mod weighted_random;
pub use self::weighted_random::WeightedRandom;
// mod top;
// pub use top::Top;

pub trait SelectionStrategy {
    /// Select a `Category` via some method determined by the implementor.
    fn select_category(&mut self) -> Category;

    /// Select the index of a `Task` from the slice via some method determined by the implementor.
    /// If the slice is empty, implementors are expected to panic.
    // FIXME: is there a better way of doing the tasks parameter so it can come from a
    // Vec<(T, Task)> but still be passed as a trait object
    fn select_task(&mut self, tasks: &[&Task]) -> usize;
}

#[cfg(test)]
mod tests;
