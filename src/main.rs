mod descry;

use clap::{App, Arg};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let matches = App::new("descry")
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .args(&[
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom execution path (default: descry.sh)")
                .takes_value(true),
            Arg::with_name("test")
                .short("t")
                .long("test")
                .value_name("FILE")
                .help("Executes the file passed in to test its execution. (default: descry.sh)")
                .takes_value(true),
        ])
        .after_help("")
        .get_matches();

    let config_file = matches.value_of("config").unwrap_or("descry.sh");

    if let Err(e) = descry::init(config_file) {
        panic!("Unable to run application: {}", e);
    }
}