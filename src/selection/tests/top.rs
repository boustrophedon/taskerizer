use crate::task::{Category, Task};
use crate::task::test_utils::{example_task_1, example_task_2, example_task_3, arb_task_list};

use crate::selection::{Top, SelectionStrategy};

#[test]
fn test_selection_selection_top_works() {
    let mut selector = Top::new();

    let tasks = [example_task_1()];
    let task_refs: Vec<&Task> = tasks.iter().collect();

    let item = selector.select_task(&task_refs);

    assert_eq!(item, 0);
}

#[test]
fn test_selection_top_equal_priorities_3() {
    let mut selector = Top::new();

    // same task so they have the same priorities
    let tasks = vec![example_task_1(), example_task_1(), example_task_1()];
    let task_refs: Vec<&Task> = tasks.iter().collect();

    // if they're equal, Top selects the last one
    assert_eq!(selector.select_task(&task_refs), 2);
}

#[test]
fn test_selection_top_nonequal_priorities() {
    let mut selector = Top::new();

    // task 2: priority 12
    // task 3: priority 2
    let tasks = vec![example_task_2(), example_task_3()];
    let task_refs: Vec<&Task> = tasks.iter().collect();

    // 12 > 2
    assert_eq!(selector.select_task(&task_refs), 0);
}

proptest! {
    #[test]
    fn test_selection_top_arb(tasks in arb_task_list()) {
        let mut selector = Top::new();

        // always selects Category::Task
        prop_assert_eq!(selector.select_category(), Category::Task);

        let mut task_refs: Vec<&Task> = tasks.iter().collect();
        task_refs.sort_by_key(|t| t.priority());
       
        // sort by priority, then check the last element is always the one selected
        // remove it and repeat
        while !task_refs.is_empty() {
            prop_assert_eq!(selector.select_task(&task_refs), task_refs.len()-1);
            task_refs.pop();
        }
    }
}
