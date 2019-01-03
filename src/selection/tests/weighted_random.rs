use crate::task::test_utils::{example_task_1, example_task_3, example_task_list, arb_task_list_bounded};
use crate::selection::{WeightedRandom, SelectionStrategy};

// FIXME: compute actual 99.99% confidence intervals or whatever for the values and put
// computations in comment here.


#[test]
fn test_weighted_random_works() {
    let mut selector = WeightedRandom::new();

    let tasks = vec![(0, example_task_1())];

    let item = selector.select_task(&tasks);

    assert_eq!(item, 0);
}

#[test]
fn test_weighted_random_equal_priorities() {
    let mut selector = WeightedRandom::new();

    // same task so they have the same priorities
    let tasks = vec![(0, example_task_1()), (1, example_task_1())];

    let mut counts: [usize; 2] = [0,0];
    for _ in 0..100 {
        counts[selector.select_task(&tasks)] += 1;
    }

    // "20" is basically arbitrary but it's unlikely to happen so it's good enough.

    assert!(counts[0] > 0, "First task was not chosen at all");
    assert!(counts[0] > 20, "First task was not chosen enough");

    assert!(counts[1] > 0, "Second task was not chosen at all");
    assert!(counts[1] > 20, "Second task was not chosen enough");
} 

#[test]
fn test_weighted_random_equal_priorities_3() {
    let mut selector = WeightedRandom::new();

    // same task so they have the same priorities
    let tasks = vec![(0, example_task_1()), (1, example_task_1()), (2, example_task_1())];

    let mut counts: [usize; 3] = [0,0,0];
    for _ in 0..100 {
        counts[selector.select_task(&tasks)] += 1;
    }

    // "10" is basically arbitrary but it's unlikely to happen so it's good enough.

    assert!(counts[0] > 0, "First task was not chosen at all");
    assert!(counts[0] > 10, "First task was not chosen enough");

    assert!(counts[1] > 0, "Second task was not chosen at all");
    assert!(counts[1] > 10, "Second task was not chosen enough");

    assert!(counts[2] > 0, "Third task was not chosen at all");
    assert!(counts[2] > 10, "Third task was not chosen enough");
} 
