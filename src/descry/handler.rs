use std::{collections::HashMap, thread};

use rifling::{HookFunc, Delivery, DeliveryType};
use run_script::{ScriptOptions};
use yaml_rust::Yaml;

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
            if let Some(exec) = command {
                println!("Running commands in \"{}\" section", &section_name);
                println!("Parsed command: {}", &exec);
                let mut options = ScriptOptions::new();
                options.exit_on_error = self.config["settings"]["exit_on_error"]
                    .as_bool()
                    .unwrap_or(false);
                options.print_commands = self.config["settings"]["print_commands"]
                    .as_bool()
                    .unwrap_or(false);
                println!("Executor option: {:#?}", &options);
                let args = vec![];

                thread::spawn(move || {
                    let (code, output, error) = run_script::run(&exec.as_str(), &args, &options)
                        .expect("Failed to execute command");

                    println!("Commands in \"{}\" section exited with code {}", &section_name, code);

                    println!("stdout:\n{}", output);
                    println!("stderr:\n{}", error);
                });
            }
        }

        println!("Returning 200");
    }
}