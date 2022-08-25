mod descry;

use colored::Colorize;
use hyper::rt::{run, Future};
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
                .help("Sets a custom execution path (default: descry.yaml)")
                .takes_value(true),
        ])
        .after_help("")
        .get_matches();

    let config_file = matches.value_of("config").unwrap_or("descry.yaml");

    match descry::init(config_file) {
        Ok((err_prone_server, _config)) => {
            let server = err_prone_server.map_err(|e| { 
                panic!("{} {}", format!("Error:").red().bold(), e)
            });

            run(server);
        },
        Err(err) => {
            println!("{} Descry was unable to launch the server due to a error-handling related failure. Refer: {}", format!("Error:").red().bold(), err)
        },
    }
}