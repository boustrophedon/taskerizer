use super::Task;
use proptest::arbitrary::any;

#[test]
fn test_task_fmt_row_1() {
    let task = super::test_utils::example_task_1();
    let row = task.format_row(1);
    assert_eq!(row, "1 \t test task please ignore");

    let row = task.format_row(2);
    assert_eq!(row, " 1 \t test task please ignore");

    let row = task.format_row(4);
    assert_eq!(row, "   1 \t test task please ignore");
}

#[test]
fn test_task_fmt_row_2() {
    let task = super::test_utils::example_task_2();
    let row = task.format_row(1);
    assert_eq!(row, "12 \t test task please ignore 2");

    let row = task.format_row(2);
    assert_eq!(row, "12 \t test task please ignore 2");

    let row = task.format_row(4);
    assert_eq!(row, "  12 \t test task please ignore 2");
}

#[test]
fn test_task_fmt_long_1() {
    let task = super::test_utils::example_task_1();
    let long = task.format_long();
    assert_eq!(long, "Task: test task please ignore\n\
    Priority: 1\n\
    Category: Task")
}

#[test]
fn test_task_fmt_long_2() {
    let task = super::test_utils::example_task_3();
    let long = task.format_long();
    assert_eq!(long, "Task: just another task\n\
    Priority: 2\n\
    Category: Task")
}

#[test]
fn test_task_fmt_long_3() {
    let task = super::test_utils::example_task_break_2();
    let long = task.format_long();
    assert_eq!(long, "Task: break with high priority\n\
    Priority: 99\n\
    Category: Break")
}

#[test]
fn test_task_nonempty_task() {
    let res = Task::new_from_parts("".to_string(), 1, false);
    assert!(res.is_err(), "Task was valid with empty task description");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Empty task description"), "Incorrect error with empty task description: {}", err);
}

#[test]
fn test_task_zero_priority() {
    let res = Task::new_from_parts("a".to_string(), 0, false);
    assert!(res.is_err(), "Task was valid with zero priority");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Zero priority"), "Incorrect error with zero priority: {}", err);
}

#[test]
fn test_task_null_desc() {
    let res = Task::new_from_parts("\x00".to_string(), 1, false);
    assert!(res.is_err(), "Task was valid with null byte in string");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Null byte"), "Incorrect error with null byte: {}", err);

    // null bytes not at beginning
    let res = Task::new_from_parts("test task \x00 \x00 test".to_string(), 1, false);
    assert!(res.is_err(), "Task was valid with null byte in string");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Null byte"), "Incorrect error with null byte: {}", err);

    // null byte just at end
    let res = Task::new_from_parts("test task \x00".to_string(), 1, false);
    assert!(res.is_err(), "Task was valid with null byte in string");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Null byte"), "Incorrect error with null byte: {}", err);
}

#[test]
fn test_task_uuid_not_equal() {
    let res = Task::new_from_parts("hello".to_string(), 1, false);
    assert!(res.is_ok(), "Task was invalid with normal parts");
    let task1 = res.unwrap();

    let res = Task::new_from_parts("hello".to_string(), 1, false);
    assert!(res.is_ok(), "Task was invalid with normal parts");
    let task2 = res.unwrap();

    assert!(task1.uuid() != task2.uuid(), "Task uuids were the same");
    assert!(task1 != task2, "Tasks with same parts but different uuids are equal");
}

proptest! {
    #[test]
    /// Create a valid task and check that Task::new_from_parts works. Valid task has at least one
    /// character, no null bytes, and priority at least 1.
    fn test_task_valid_arb_task(task in "[^\x00]+", priority in 1u32.., reward in any::<bool>()) {
        let res = Task::new_from_parts(task.clone(), priority, reward);
        prop_assert!(res.is_ok(), "Task made from good parts returned an error: {}", res.unwrap_err());
        let task1 = res.unwrap();

        let res = Task::new_from_parts(task.clone(), priority, reward);
        prop_assert!(res.is_ok(), "Task made from good parts returned an error: {}", res.unwrap_err());
        let task2 = res.unwrap();

        prop_assert!(task1.uuid() != task2.uuid(), "Different task uuids were the same");
        prop_assert!(task1 != task2, "Tasks with same parts but different uuids are equal");
    }
}
