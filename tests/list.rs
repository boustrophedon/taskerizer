use taskerizer_prototype as tkzr;

mod test_utils;

use self::tkzr::commands::{TKZArgs, TKZCmd};

#[test]
fn test_cmd_list() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command

    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do list command with same db that we just did add on

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // -- check output has exactly the task we previously added
    let output = res.unwrap();
    let expected = vec![
        "Priority \t Task".to_string(),
        "   1 \t hello this is a task".to_string(),
    ];
    assert_eq!(output, expected);

}

#[test]
fn test_cmd_list_two() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command

    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task 1 failed");

    // -- do second add command

    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding task 2 failed");

    // -- do list command with same db that we just did adds to

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // -- assert output has both tasks we previously added
    let output = res.unwrap();
    let expected = vec![
        "Priority \t Task".to_string(),
        "   1 \t hello this is a task".to_string(),
        "   2 \t yo this is a break".to_string(),
    ];
    assert_eq!(output, expected);

}

// this test explicitly checks the order of the output
#[test]
fn test_cmd_list_four() {
    let (_dir, cfg) = test_utils::temp_config();

    // -- do add command

    let args = test_utils::example_add_cmd_task1();
    args.cmd().dispatch(&cfg).expect("Adding task 1 failed");

    // -- do second add command

    let args = test_utils::example_add_cmd_task2();
    args.cmd().dispatch(&cfg).expect("Adding task 2 failed");

    let args = test_utils::example_add_cmd_break1();
    args.cmd().dispatch(&cfg).expect("Adding break 1 failed");

    // -- do second add command

    let args = test_utils::example_add_cmd_break2();
    args.cmd().dispatch(&cfg).expect("Adding break 2 failed");

    // -- do list command with same db that we just did adds to

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);

    // -- check success
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // -- assert output has all tasks we previously added
    let output = res.unwrap();
    let expected = vec![
        "Priority \t Task".to_string(),
        "   1 \t hello this is a task".to_string(),
        "   9 \t hello this is also a task".to_string(),
        "   2 \t yo this is a break".to_string(),
        "   4 \t ayyy this is another break".to_string(),
    ];
    assert_eq!(output, expected);

}
// TODO test failure modes
