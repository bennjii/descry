use std::{collections::HashMap, thread, io::{ErrorKind, Error, BufReader, BufRead}, process::{Command, Stdio}};

use colored::Colorize;
use rifling::{HookFunc, Delivery, DeliveryType};
use run_script::{ScriptOptions, types::IoOptions};
use yaml_rust::Yaml;
use std::env;
use std::fs::File;
use std::io::prelude::*;

macro_rules! get_value {
    ($source:expr) => {
        match $source {
            Some(string) => string.as_str(),
            None => "unknown",
        }
    };
}

#[derive(Clone)]
/// Handler of the deliveries
pub struct Handler {
    config: Yaml,
}

impl Handler {
    pub fn new(config: Yaml) -> Handler {
        Handler { config }
    }

    pub fn process_commands(&self, event: &str, delivery: &Delivery) -> Option<String> {
        let common_command = self.config["events"]["common"].as_str().unwrap_or("");

        if let Some(command) = self.config["events"][event].as_str() {
            let mut exec = String::from(command);
            exec = format!("{}\n{}", &common_command, &exec);
            // Replace placeholders in commands
            exec = exec.replace(
                "{source}",
                format!("{:?}", &delivery.delivery_type).as_str(),
            );
            exec = exec.replace("{id}", get_value!(&delivery.id));
            exec = exec.replace("{event}", get_value!(&delivery.event));
            exec = exec.replace("{signature}", get_value!(&delivery.signature));
            exec = exec.replace("{payload}", get_value!(&delivery.unparsed_payload));
            exec = exec.replace("{request_body}", get_value!(&delivery.request_body));
            Some(exec)
        } else {
            None
        }
    }
}

impl HookFunc for Handler {
    /// Handle the delivery
    fn run(&self, delivery: &Delivery) {
        // Check if its running on a branch that is verified or that is wishing to be run, otherwise fork branches can be created which have the possibility of spam RCE to stall the server.
        let event = get_value!(&delivery.event);
        match &delivery.delivery_type {
            DeliveryType::GitHub => {
                let id = get_value!(&delivery.id);
                println!("Delivery ID: {}", id);
            }
            _ => {
                println!(
                    "Delivery ID not available for requests from {:?}",
                    &delivery.delivery_type
                );
            }
        }
        // Prepare the commands
        let mut commands_all: HashMap<String, Option<String>> = HashMap::new();

        // Prepare commands in `all` section
        commands_all.insert(
            "all".to_string(),
            self.process_commands("all", &delivery),
        );

        // Prepare commands matching the event
        if let Some(command) = self.process_commands(event, &delivery) {
            commands_all.insert(event.into(), Some(command));
        } else {
            commands_all.insert(
                "all".to_string(),
                self.process_commands("else", &delivery),
            );
        }

        // Execute the commands
        for (section_name, command) in commands_all {
            if let Some(exec) = command {
                let mut options = ScriptOptions::new();
                options.exit_on_error = self.config["settings"]["exit_on_error"]
                    .as_bool()
                    .unwrap_or(false);
                options.print_commands = self.config["settings"]["print_commands"]
                    .as_bool()
                    .unwrap_or(false);
                options.output_redirection = IoOptions::Pipe;

                // We have gathered all of the commands into their respective categories and joined them together.
                // Then, we parse and execute these commands - most efficient method.
                let _handler = thread::spawn(move || {
                    let temp_directory = env::temp_dir().join("descry").to_path_buf();
                    let directory_as_string = temp_directory.as_path().display();

                    println!("Running commands in \"{}\" section ({})", format!("{}", &section_name).green().bold(), format!("{}/{}.sh", directory_as_string, &section_name).green().bold());
                    println!("Command Format: \n\n{}\n\n", format!("{}", &exec).green().bold());

                    // As this is a custom category, we need to use the custom generated commands - which we have generated.
                    // We save these to a temp file in the temp directory with the name following: <category_type>-temp.sh
                    let mut file = File::create(&format!("{}/{}-temp.sh", directory_as_string, &section_name)).unwrap();
                    file.write_all(&exec.as_bytes()).unwrap();

                    // Run this temporary file.
                    let stdout = Command::new("sh")
                        .args(["-C", &format!("{}/{}-temp.sh", directory_as_string, &section_name)])
                        .stdout(Stdio::piped())
                        .spawn().expect("Could not spawn child process")
                        .stdout
                        .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture standard output."))
                        .expect("Failed to start output pipe");

                    let reader = BufReader::new(stdout);

                    thread::spawn(move || {
                        let saved_name = section_name.clone();
                        let mut output = String::new();
                    
                        reader
                            .lines()
                            .filter_map(|line| line.ok())
                            .for_each(|line| {
                                println!("[{}] {}", &saved_name, line);
                                output.insert_str(output.len(), &line);
                            });
                    })
                    
                }).join();
            }
        }
    }
}