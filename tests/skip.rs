mod test_utils;

// -- TODO test top parameter

#[test]
fn test_cmd_skip_empty() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do skip command with no tasks

    let args = test_utils::example_skip();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Skipping task failed: {}", res.unwrap_err());

    // -- check output says no task was found
    let output = res.unwrap();

    let expected = vec!["No tasks."];

    assert_eq!(output, expected);
}

#[test]
/// Add task, do skip, only one task so we get the same current task again.
fn test_cmd_skip_1() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do skip command

    let args = test_utils::example_skip();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Skipping task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Current task is now \"{}\".\n", "hello this is a task"),
    ];

    assert_eq!(output, expected);
}

#[test]
/// Add break, do skip, only one task so we get the same current task again.
fn test_cmd_skip_2() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do skip command

    let args = test_utils::example_skip();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Skipping task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Current task is now \"{}\".\n", "yo this is a break"),
    ];

    assert_eq!(output, expected);
}

#[test]
/// Add a task and a break, skip the break (added first so it must have been selected), check that
/// we get the second task.
fn test_cmd_skip_3() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do another add command
    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do skip command
    let args = test_utils::example_skip();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Skipping task failed: {}", res.unwrap_err());

    // -- check output contains the second task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Current task is now \"{}\".\n", "hello this is a task"),
    ];

    assert_eq!(output, expected);

}

#[test]
/// Add two tasks, swap between them via skip.
fn test_cmd_skip_4() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do two add commands
    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");
    let args = test_utils::example_add_cmd_task2();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do skip command
    let args = test_utils::example_skip();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Skipping task failed: {}", res.unwrap_err());

    // -- check output contains the second task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Current task is now \"{}\".\n", "hello this is also a task"),
    ];

    assert_eq!(output, expected);


    // -- do skip command
    let args = test_utils::example_skip();
    let res = args.cmd().dispatch(&cfg);

    // -- check output contains the first task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Current task is now \"{}\".\n", "hello this is a task"),
    ];

    assert_eq!(output, expected);
}
