use std::{collections::HashMap, thread, io::{ErrorKind, Error, BufReader, BufRead}, process::{Command, Stdio}};

use colored::Colorize;
use rifling::{HookFunc, Delivery, DeliveryType};
use run_script::{ScriptOptions, types::IoOptions};
use yaml_rust::Yaml;
use std::env;

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
        println!("Running {:#?}", delivery.delivery_type);

        let event = get_value!(&delivery.event);
        match &delivery.delivery_type {
            DeliveryType::GitHub => {
                let id = get_value!(&delivery.id);
                println!("Delivery ID: \"{}\"", id);
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
            if let Some(_exec) = command {
                println!("Running commands in \"{}\" section", format!("{}", &section_name).green().bold());

                let mut options = ScriptOptions::new();
                options.exit_on_error = self.config["settings"]["exit_on_error"]
                    .as_bool()
                    .unwrap_or(false);
                options.print_commands = self.config["settings"]["print_commands"]
                    .as_bool()
                    .unwrap_or(false);
                options.output_redirection = IoOptions::Pipe;

                let _handler = thread::spawn(move || {
                    println!("Spawned Thread to Handle PUSH script.");

                    let temp_directory = env::temp_dir().join("descry").to_path_buf();
                    let directory_as_string = temp_directory.as_path().display();

                    let stdout = Command::new("sh")
                        .args(["-C", &format!("{}/{}.sh", directory_as_string, section_name)])
                        .stdout(Stdio::piped())
                        .spawn().expect("Could not spawn child process")
                        .stdout
                        .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture standard output."))
                        .expect("Failed to start output pipe");

                    let reader = BufReader::new(stdout);

                    let mut output = String::new();
                    
                    reader
                        .lines()
                        .filter_map(|line| line.ok())
                        .for_each(|line| {
                            println!("hello {}", line);
                            output.insert_str(output.len(), &line);
                        });
                }).join();

                println!("Completed!");
            }
        }
    }
}