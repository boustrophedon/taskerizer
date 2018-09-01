extern crate taskerizer_prototype as tkzr;

extern crate tempfile;
use tempfile::tempdir;

use tkzr::{commands, config::Config};
use commands::{TKZArgs, TKZCmd};

use commands::Add;

// TODO lots of duplicated setup code here between add and list

#[test]
fn test_cmd_list() {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // don't use into_path because test_dir will not be deleted on drop
    let db_path = test_dir.path().to_path_buf();

    let cfg = Config {
        db_path: db_path,
    };

    // -- do add command

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

    args.cmd().dispatch(&cfg).expect("Adding task failed");

    // -- do list command with same db that we just did add on

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // assert output is the task we previously added
    let output = res.unwrap();
    let expected = vec![
        "Item\tTask\tPriority".to_string(),
        "1\thello this is a task\t1".to_string(),
    ];
    assert_eq!(output, expected);

}

#[test]
fn test_cmd_list_two() {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // don't use into_path because test_dir will not be deleted on drop
    let db_path = test_dir.path().to_path_buf();

    let cfg = Config {
        db_path: db_path,
    };

    // -- do add command

    let task1 = vec!["hello", "this", "is", "a task"].into_iter().map(From::from).collect();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task1,
        }))
    };

    // -- do second add command

    args.cmd().dispatch(&cfg).expect("Adding task 1 failed");

    let task2 = vec!["yo", "this", "is", "another", "task"].into_iter().map(From::from).collect();
    let args = TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: true,
            priority: 4,
            task: task2,
        }))
    };

    args.cmd().dispatch(&cfg).expect("Adding task 2 failed");

    // -- do list command with same db that we just did adds to

    let args = TKZArgs {
        cmd: Some(TKZCmd::List)
    };
    let res = args.cmd().dispatch(&cfg);
    assert!(res.is_ok(), "List command failed: {}", res.unwrap_err());

    // assert output is the task we previously added
    let output = res.unwrap();
    let expected = vec![
        "Item\tTask\tPriority".to_string(),
        "1\thello this is a task\t1".to_string(),
        "2\tyo this is another task\t4".to_string(),
    ];
    assert_eq!(output, expected);

}

// TODO test failure modes?
