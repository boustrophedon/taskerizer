extern crate taskerizer_prototype as tkzr;

extern crate tempfile;
use tempfile::tempdir;

use tkzr::{commands, config::Config};
use commands::{TKZArgs, TKZCmd};

use commands::Add;

// TODO some setup code here is shared with the inner unit tests, maybe find a way to dedup

#[test]
fn test_cmd_add_empty() {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // don't use into_path because test_dir will not be deleted on drop
    let db_path = test_dir.path().to_path_buf();

    let cfg = Config {
        db_path: db_path,
    };

    let task = Vec::new();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        }))
    };

    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_err(), "Add command incorrectly succeded: {:?}", res.unwrap());
    let err = res.unwrap_err();

    assert!(err.to_string() == "Task cannot be empty.", "Incorrect error message: {}", err);
}

#[test]
fn test_cmd_add() {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // don't use into_path because test_dir will not be deleted on drop
    let db_path = test_dir.path().to_path_buf();

    let cfg = Config {
        db_path: db_path,
    };

    // kind of long but better than a million `.to_string()`s
    // i guess i could put it into a utility fn
    let task = vec!["hello", "this", "is", "a task"].into_iter().map(From::from).collect();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        }))
    };

    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "Add command failed: {}", res.unwrap_err());
    let output = res.unwrap();

    let expected = vec![
        format!("Task \"{}\" added to task list.", "hello this is a test"),
    ];
    assert_eq!(output, expected);
}

// TODO test failure modes?
