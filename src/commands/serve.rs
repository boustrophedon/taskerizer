use crate::sync::server::DEFAULT_PORT_STR;

#[derive(StructOpt, Debug)]
pub struct Serve {
    /// The address to listen on.
    #[structopt(long = "address", default_value = "127.0.0.1")]
    pub address: std::net::IpAddr,

    /// The port to listen on.
    // NOTE: we have to use raw clap arguments here because structopt requires default_values to be
    // strings (and so does clap, so we have to have a separate static str)
    #[structopt(short = "p", long = "port", raw(default_value = r#"DEFAULT_PORT_STR"#))]
    pub port: u16,
}
