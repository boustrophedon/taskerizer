use crate::task::Task;

mod weighted_random;
pub use self::weighted_random::WeightedRandom;
// mod top;
// pub use top::Top;

pub trait SelectionStrategy {
    /// Select a T from the slice of (T, Tasks) via some method determined by the implementor. If
    /// the slice is empty, implementors are expected to panic.
    ///
    /// NB: T will pretty much always be a db::transaction::RowId, but it's not a public type. This
    /// also makes it easier to test.
    fn select_task<T>(&mut self, tasks: &[(T, Task)]) -> usize;
}

#[cfg(test)]
mod tests;
