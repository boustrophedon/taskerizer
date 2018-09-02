extern crate tempfile;

extern crate taskerizer_prototype as tkzr;

use self::tempfile::{tempdir, TempDir};

use tkzr::commands::{TKZArgs, TKZCmd};
use tkzr::commands::Add;

use tkzr::config::Config;

/// Create a test config with the database in a temporary directory. We return the TempDir because
/// it is deleted when it is dropped.
pub fn temp_config() -> (TempDir, Config) {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // don't use into_path because test_dir will not be deleted on drop
    let db_path = test_dir.path().to_path_buf();

    let cfg = Config {
        db_path: db_path,
    };

    (test_dir, cfg)
}

pub fn example_add_cmd1() -> TKZArgs {
    let task = vec!["hello", "this", "is", "a task"].into_iter().map(From::from).collect();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        })) 
    }  
}

pub fn example_add_cmd2() -> TKZArgs {
    let task = vec!["yo", "this", "is", "another", "task"].into_iter().map(From::from).collect();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: true,
            priority: 4,
            task: task,
        }))
    }
}
