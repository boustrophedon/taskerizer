use crate::task::Category;
use crate::task::test_utils::{example_task_1, example_task_2, example_task_3, arb_task_list};

use crate::selection::{WeightedRandom, SelectionStrategy};

// FIXME: compute actual 99.99% confidence intervals or whatever for the values and put
// computations in comment here.

// TBH we don't really need to test this, it's just a sanity check
#[test]
fn test_select_category_works() {
    let mut selector = WeightedRandom::new(1.0);
    assert_eq!(Category::Break, selector.select_category());

    let mut selector = WeightedRandom::new(0.0);
    assert_eq!(Category::Task, selector.select_category());
}

#[test]
fn test_weighted_random_works() {
    let mut selector = WeightedRandom::new(0.0);

    let tasks = vec![(0, example_task_1())];

    let item = selector.select_task(&tasks);

    assert_eq!(item, 0);
}

#[test]
fn test_weighted_random_equal_priorities() {
    let mut selector = WeightedRandom::new(0.0);

    // same task so they have the same priorities
    let tasks = vec![(0, example_task_1()), (1, example_task_1())];

    let mut counts: [usize; 2] = [0,0];
    for _ in 0..100 {
        counts[selector.select_task(&tasks)] += 1;
    }

    // "20" is basically arbitrary but it's unlikely to happen so it's good enough.

    assert!(counts[0] > 0, "First task was not chosen at all");
    assert!(counts[0] >= 20, "First task was not chosen enough");

    assert!(counts[1] > 0, "Second task was not chosen at all");
    assert!(counts[1] >= 20, "Second task was not chosen enough");
} 

#[test]
fn test_weighted_random_equal_priorities_3() {
    let mut selector = WeightedRandom::new(0.0);

    // same task so they have the same priorities
    let tasks = vec![(0, example_task_1()), (1, example_task_1()), (2, example_task_1())];

    let mut counts: [usize; 3] = [0,0,0];
    for _ in 0..100 {
        counts[selector.select_task(&tasks)] += 1;
    }

    // "10" is basically arbitrary but it's unlikely to happen so it's good enough.

    assert!(counts[0] > 0, "First task was not chosen at all");
    assert!(counts[0] >= 10, "First task was not chosen enough");

    assert!(counts[1] > 0, "Second task was not chosen at all");
    assert!(counts[1] >= 10, "Second task was not chosen enough");

    assert!(counts[2] > 0, "Third task was not chosen at all");
    assert!(counts[2] >= 10, "Third task was not chosen enough");
}

#[test]
fn test_weighted_random_nonequal_priorities() {
    let mut selector = WeightedRandom::new(0.0);

    // task 2: priority 12
    // task 3: priority 2
    let tasks = vec![(0, example_task_2()), (1, example_task_3())];

    let mut counts: [usize; 2] = [0,0];
    for _ in 0..100 {
        counts[selector.select_task(&tasks)] += 1;
    }

    // completely arbitrary heuristic: 1/4*n*p

    // 100*12/14*1/4 = 21
    assert!(counts[0] > 0, "Second task was not chosen at all");
    assert!(counts[0] >= 20, "Second task was not chosen enough");

    // 100*2/14*1/4 = 3 
    assert!(counts[1] > 0, "Third task was not chosen at all");
    assert!(counts[1] >= 2, "Third task was not chosen enough");
}

proptest! {
    #[test]
    fn test_weighted_random_nonequal_priorities_arb(_not_used: f32, tasks in arb_task_list()) {
        let mut selector = WeightedRandom::new(_not_used);

        let num_items = tasks.len();
        let tasks: Vec<_> = tasks.into_iter().enumerate().collect();
        
        let mut counts = vec![0; num_items];
        for _ in 0..1000 {
            counts[selector.select_task(&tasks)] += 1;
        }

        let p_denom = tasks.iter().fold(0f32, |acc, (_, task)| acc + task.priority() as f32);
        for (i, task) in tasks {
            // FIXME: again completely arbitrary
            let heuristic = (0.25 * 1000.0 * (task.priority() as f32 / p_denom).floor()) as usize;
            prop_assert!(counts[i] >= heuristic, "Task {} was not chosen enough: task {:?}, heuristic {}", i, task, heuristic);
        }

    }
}
