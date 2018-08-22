extern crate taskerizer_prototype;
use taskerizer_prototype::{commands, config::Config};

fn main() {
    let args = commands::TKZArgs::get_args();
    let cmd = match args.cmd {
        Some(cmd) => cmd,
        None => commands::TKZCmd::List,
    };

    println!("{:?}", cmd);

    let config = Config::config();

    let res = cmd.dispatch(&config);

    match res {
        Ok(()) => return,
        Err(f) => eprintln!("Error completing action: {}", f),
    }
}
