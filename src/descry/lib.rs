use std::{error::Error, fs::{File, self}, io::{BufReader, Read, Write}, process, net::{SocketAddr}};
use colored::Colorize;

use rifling::{Constructor, Hook};
use yaml_rust::{YamlLoader, Yaml};

use hyper::{server::conn::AddrIncoming};
use hyper::Server;
use std::env;

use crate::descry::Handler;

pub fn init(config_file: &str) -> Result<(Server<AddrIncoming, Constructor>, Yaml), Box<dyn Error>>  {
    let mut config_content = String::new();
    let config_file = match File::open(config_file) {
        Ok(file) => {
            file
        },
        Err(_) => {
            println!("{} \n{}", format!("Descry was unable to find the file specified.").red().bold(), "Please input a valid path URL.");
            process::exit(0);
        },
    };

    let mut buf_reader = BufReader::new(config_file);
    buf_reader.read_to_string(&mut config_content)?;

    let config = &YamlLoader::load_from_str(config_content.as_str())?[0];

    println!("{}", format!("Loaded Configuration File ✅").green().bold());

    let events_map = config["events"].clone().into_hash().expect("Unable to convert to hashmap");

    println!("Listening on port :{} for {:?} Events", &config["settings"]["host"].as_str().expect("").split("0.0.0.0:").collect::<String>(), events_map.keys().len());

    let temp_directory = env::temp_dir().join("descry").to_path_buf();
    let directory_as_string = temp_directory.as_path().display();

    println!("Creating in temporary directory: {}", format!("{}", directory_as_string).green().bold());

    fs::create_dir_all(directory_as_string.to_string()).expect("Was unable to create /scripts directory");

    for elem in events_map.into_iter() {
        let item_name = elem.0.into_string().expect("Unable to scrub name for element");
        let item_value = elem.1.into_string().expect("Unable to scrub value for element");

        let mut file = fs::File::create(format!("{}/{}.sh", directory_as_string, item_name)).expect("Failed to create file in scripts folder");
        writeln!(&mut file, "{}", item_value).unwrap();

        println!(" -  {}", item_name);
    }

    let secret = if let Some(secret) = config["settings"]["secret"].as_str() {
        Some(String::from(secret))
    } else {
        None
    };

    let handler = Handler::new(config.clone());
    let mut cons = Constructor::new();
    let hook = Hook::new("*", secret, handler);
    cons.register(hook);
    
    let addr: SocketAddr = config["settings"]["host"]
        .as_str()
        .expect("Unable to read host address")
        .parse()
        .expect("Unable to parse host address");

    let server = Server::bind(&addr)
        .serve(cons);

    println!("\n{}", format!("Descry Actively Listening...").green().bold());

    Ok((server, config.to_owned()))
}