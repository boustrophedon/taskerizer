extern crate taskerizer_prototype;
use taskerizer_prototype::{commands, config::Config};
use taskerizer_prototype::sync::server::TkzrServer;

use std::path::PathBuf;

fn main() {
    let args = commands::TKZArgs::get_args();
    let cmd = args.cmd();

    let config = Config {
        db_path: PathBuf::from("/tmp/tkzr"),
        break_cutoff: 0.33,
    };

    if let commands::TKZCmd::Serve(params) = cmd {
        // TODO: logging
        println!("Listening on {}:{}", params.address, params.port);
        TkzrServer::new((params.address, params.port), config).start().unwrap().join().unwrap();
    }
    else {
        match cmd.dispatch(&config) {
            Ok(output) => {
                for line in output {
                    println!("{}", line);
                }
            },
            Err(e) => eprintln!("Error completing action: {}", e),
        }
    }
}
