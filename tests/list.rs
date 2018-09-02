extern crate taskerizer_prototype as tkzr;

mod test_utils;

use tkzr::commands;
use commands::{TKZArgs, TKZCmd};

#[test]
fn test_cmd_list() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command

    let args = test_utils::example_add_cmd1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do list command with same db that we just did add on

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // -- assert output is the task we previously added
    let output = res.unwrap();
    let expected = vec![
        "Item\tTask\tPriority".to_string(),
        "1\thello this is a task\t1".to_string(),
    ];
    assert_eq!(output, expected);

}

#[test]
fn test_cmd_list_two() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command

    let args = test_utils::example_add_cmd1();
    args.cmd().dispatch(&cfg).expect("Adding task 1 failed");

    // -- do second add command

    let args = test_utils::example_add_cmd2();
    args.cmd().dispatch(&cfg).expect("Adding task 2 failed");

    // -- do list command with same db that we just did adds to

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // -- assert output is the task we previously added
    let output = res.unwrap();
    let expected = vec![
        "Item\tTask\tPriority".to_string(),
        "1\thello this is a task\t1".to_string(),
        "2\tyo this is another task\t4".to_string(),
    ];
    assert_eq!(output, expected);

}

// TODO test failure modes
