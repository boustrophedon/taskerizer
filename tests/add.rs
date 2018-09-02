extern crate taskerizer_prototype as tkzr;

mod test_utils;

use tkzr::commands;
use commands::{TKZArgs, TKZCmd};

use commands::Add;

// TODO some setup code here is shared with the inner unit tests, maybe find a way to dedup

#[test]
fn test_cmd_add_empty() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command with empty task
    let task = Vec::new();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        }))
    };

    // -- check failure
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_err(), "Add command incorrectly succeded: {:?}", res.unwrap());
    let err = res.unwrap_err();

    // -- verify failure was due to empty task
    assert!(err.to_string() == "Task cannot be empty.", "Incorrect error message: {}", err);
}

#[test]
fn test_cmd_add_priority_0() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command with priority 0
    let task = vec!["hello", "this", "is", "a task"].into_iter().map(From::from).collect();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 0,
            task: task,
        }))
    };

    // -- check failure
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_err(), "Add command incorrectly succeded: {:?}", res.unwrap());
    let err = res.unwrap_err();

    // -- verify failure was due to 0 priority
    assert!(err.to_string() == "Task cannot have priority 0 since it will never be selected.", "Incorrect error message: {}", err);
}

#[test]
fn test_cmd_add() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd1();

    // -- check success
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());
    let output = res.unwrap();

    // -- verify output
    let expected = vec![
        format!("Task \"{}\" added to task list.", "hello this is a task"),
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_cmd_add_two() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd1();

    // -- check success
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());
    let output = res.unwrap();

    // -- verify output
    let expected = vec![
        format!("Task \"{}\" added to task list.", "hello this is a task"),
    ];
    assert_eq!(output, expected);


    // -- do second add command
    let args = test_utils::example_add_cmd2();

    // -- check success
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());
    let output = res.unwrap();

    // -- verify output
    let expected = vec![
        format!("Task \"{}\" added to task list.", "yo this is another task"),
    ];
    assert_eq!(output, expected);
}
