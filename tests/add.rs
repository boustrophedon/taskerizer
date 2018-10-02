extern crate taskerizer_prototype as tkzr;

mod test_utils;

use tkzr::commands;
use commands::{TKZArgs, TKZCmd};

use commands::Add;

// TODO some setup code here is shared with the inner unit tests, maybe find a way to dedup

// TODO see commands/add.rs this test and test below will be obviated when we write
// Task::from_parts().
#[test]
#[should_panic(expected = "Task description cannot be empty.")]
fn test_cmd_add_empty() {
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

    // this will panic
    let _ = args.cmd().dispatch(&cfg);
}

#[test]
#[should_panic(expected = "Task priority cannot be zero.")]
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

    // this will panic
    let _ = args.cmd().dispatch(&cfg);
}

#[test]
fn test_cmd_add() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command
    let args = test_utils::example_add_cmd1();

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
    let args = test_utils::example_add_cmd1();
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
    let args = test_utils::example_add_cmd2();
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());

    // -- check output has our second task
    let output = res.unwrap();
    let expected = vec![
        format!("Task \"{}\" added to task list.", "yo this is another task"),
    ];
    assert_eq!(output, expected);
}
