mod test_utils;

// TODO test current top flag

#[test]
fn test_cmd_current_empty() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do current command with no tasks

    let args = test_utils::example_current();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    // -- check output says no task was found
    let output = res.unwrap();

    let expected = vec!["No tasks."];

    assert_eq!(output, expected);
}

#[test]
fn test_cmd_current_1() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do current command

    let args = test_utils::example_current();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    // TODO implement to_string (or format_*, see todo.txt) for Task and fix it here
    let expected = vec![
        format!("{}\n", "hello this is a task"),
        "Category: Task".to_string(),
        "Priority: 1".to_string(),
    ];

    assert_eq!(output, expected);
}

#[test]
fn test_cmd_current_2() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd2();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do current command

    let args = test_utils::example_current();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    // TODO implement to_string (or format_*, see todo.txt) for Task and fix it here
    let expected = vec![
        format!("{}\n", "yo this is another task"),
        "Category: Break".to_string(),
        "Priority: 4".to_string(),
    ];

    assert_eq!(output, expected);
}

#[test]
fn test_cmd_current_3_interspersed() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd2();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do current command

    let args = test_utils::example_current();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    // TODO implement to_string (or format_*, see todo.txt) for Task and fix it here
    let expected = vec![
        format!("{}\n", "yo this is another task"),
        "Category: Break".to_string(),
        "Priority: 4".to_string(),
    ];

    assert_eq!(output, expected);

    // -- do another add command
    let args = test_utils::example_add_cmd1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    //
    // same exact operations asserts as first current command
    //

    // -- do current command again

    let args = test_utils::example_current();
    let res = args.cmd().dispatch(&cfg);

    // -- check success again
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    // -- check output continues to be the first task we added
    let output = res.unwrap();

    let expected = vec![
        format!("{}\n", "yo this is another task"),
        "Category: Break".to_string(),
        "Priority: 4".to_string(),
    ];

    assert_eq!(output, expected);

}
