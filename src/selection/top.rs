use crate::task::{Category, Task};

use super::SelectionStrategy;

/// `SelectionStrategy` implementor that chooses the top priority task. `select_category` always
/// returns `Category::Task`. If there are no Task category tasks in the db then `select_task` will
/// be given the Break tasks regardless (in select_current_task), so `select_task` will choose the
/// Break with the highest priority.
///
/// Ties are resolved via whichever max-priority task comes last in the input, due to
/// `Iterator::max_by_key`. This will usually be alphabetical order.
pub struct Top {}

impl Top {
    pub fn new() -> Top {
        Top {}
    }
}

impl SelectionStrategy for Top {
    fn select_category(&mut self) -> Category {
        Category::Task
    }

    fn select_task(&mut self, tasks: &[&Task]) -> usize {
        assert!(tasks.len() > 0, "Tasks slice is empty, nothing to select.");

        tasks.iter()
            .enumerate()
            .max_by_key(|(_, t)| t.priority())
            .expect("Tasks slice cannot be empty due to above assertion")
            .0 // take the index
    }
}
