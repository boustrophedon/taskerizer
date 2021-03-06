extern crate taskerizer_prototype;
use taskerizer_prototype::{commands, config::Config};

use std::path::PathBuf;

fn main() {
    let args = commands::TKZArgs::get_args();
    let cmd = args.cmd();

    let config = Config {
        db_path: PathBuf::from("/tmp/tkzr"),
        break_cutoff: 0.33,
    };

    match cmd.dispatch(&config) {
        Ok(output) => {
            for line in output {
                println!("{}", line);
            }
        },
        Err(e) => eprintln!("Error completing action: {}", e),
    }
}
