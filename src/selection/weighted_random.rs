use crate::task::{Category, Task};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use super::SelectionStrategy;

/// `SelectionStrategy` implementor that chooses randomly, weighted by priority. Specifically, it
/// chooses a task with probability `1/task.priority()`. Since a `Task` is not allowed to have
/// priority 0 this is not a problem, but it would be disallowed if zero priority tasks were
/// allowed regardless.
pub struct WeightedRandom {
    rng: ThreadRng,
    break_probability: f32,
}

impl WeightedRandom {
    pub fn new(break_probability: f32) -> WeightedRandom {
        WeightedRandom {
            rng: thread_rng(),
            break_probability,
        }
    }
}

impl SelectionStrategy for WeightedRandom {
    fn select_category(&mut self) -> Category {
        match self.rng.gen_bool(self.break_probability.into()) {
            true => Category::Break,
            false => Category::Task,
        }
    }

    fn select_task(&mut self, tasks: &[&Task]) -> usize {
        assert!(tasks.len() > 0, "Tasks slice is empty, nothing to select.");

        // TODO we convert the u32 into f32 to get around overflow issues but there's probably a
        // better algorithm.
        //
        // specifically, we are handling cases where the sum will overflow if we use ints. 
        let priorities = tasks.iter().map(|t| t.priority() as f32);
       
        let dist = WeightedIndex::new(priorities).expect("Error creating distribution that should never occur");

        dist.sample(&mut self.rng)
    }
}
