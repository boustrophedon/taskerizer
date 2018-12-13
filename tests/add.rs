use taskerizer_prototype as tkzr;

mod test_utils;

use self::tkzr::commands::{TKZArgs, TKZCmd, Add};

// TODO some task example data and test code here is shared with the inner unit tests, maybe find a way to dedup

#[test]
fn test_cmd_add_empty_task() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command with empty task
    let task = String::new();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        }))
    };

    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_err(), "Add command incorrectly succeeded with invalid input: {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Empty task description"), "Incorrect error message with invalid input: {}", err);
}

#[test]
fn test_cmd_add_priority_0() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command with priority 0
    let task = "test".to_string();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 0,
            task: task,
        }))
    };

    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_err(), "Add command incorrectly succeeded with invalid input: {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Zero priority"), "Incorrect error message with invalid input: {}", err);
}

#[test]
fn test_cmd_add() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_task1();

    // -- check success
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());

    // -- check output has our task
    let output = res.unwrap();
    let expected = vec![
        format!("Task \"{}\" added to task list.", "hello this is a task"),
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_cmd_add_two() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd_task1();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());

    // -- check output has our first task
    let output = res.unwrap();
    let expected = vec![
        format!("Task \"{}\" added to task list.", "hello this is a task"),
    ];
    assert_eq!(output, expected);


    // -- do second add command
    let args = test_utils::example_add_cmd_break1();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());

    // -- check output has our second task
    let output = res.unwrap();
    let expected = vec![
        format!("Task \"{}\" added to task list.", "yo this is a break"),
    ];
    assert_eq!(output, expected);
}
