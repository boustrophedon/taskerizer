mod test_utils;

// -- TODO test top parameter

#[test]
fn test_cmd_complete_empty() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do complete command with no tasks

    let args = test_utils::example_complete();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Completing task failed: {}", res.unwrap_err());

    // -- check output says no task was found
    let output = res.unwrap();

    let expected = vec!["No tasks."];

    assert_eq!(output, expected);
}

#[test]
fn test_cmd_complete_1() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do complete command

    let args = test_utils::example_complete();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Completing task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    // TODO implement to_string (or format_*, see todo.txt) for Task and fix it here
    let expected = vec![
        format!("Task \"{}\" completed.\n", "hello this is a task"),
    ];

    assert_eq!(output, expected);
}

#[test]
fn test_cmd_complete_2() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do complete command

    let args = test_utils::example_complete();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Completing task failed: {}", res.unwrap_err());

    // -- check output is the task we added
    let output = res.unwrap();

    // TODO implement to_string (or format_*, see todo.txt) for Task and fix it here
    let expected = vec![
        format!("Task \"{}\" completed.\n", "yo this is a break"),
    ];

    assert_eq!(output, expected);
}

#[test]
fn test_cmd_complete_3() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do another add command
    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do complete command
    let args = test_utils::example_complete();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Completing task failed: {}", res.unwrap_err());

    // -- check output contains the first task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Task \"{}\" completed.\n", "yo this is a break"),
    ];

    assert_eq!(output, expected);

}

#[test]
fn test_cmd_complete_4() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do complete command
    let args = test_utils::example_complete();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Completing task failed: {}", res.unwrap_err());

    // -- check output contains the first task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Task \"{}\" completed.\n", "yo this is a break"),
    ];

    assert_eq!(output, expected);

    // -- do another add command
    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do complete command
    let args = test_utils::example_complete();
    let res = args.cmd().dispatch(&cfg);

    // -- check output contains the first task we added
    let output = res.unwrap();

    let expected = vec![
        format!("Task \"{}\" completed.\n", "hello this is a task"),
    ];

    assert_eq!(output, expected);


}
