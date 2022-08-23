use std::{collections::HashMap, thread, sync::{Mutex, Arc}, io::Read};

use colored::Colorize;
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
            if let Some(exec) = command {
                println!("Running commands in \"{}\" section", format!("{}", &section_name).green().bold());
                let mut options = ScriptOptions::new();
                options.exit_on_error = self.config["settings"]["exit_on_error"]
                    .as_bool()
                    .unwrap_or(false);
                options.print_commands = self.config["settings"]["print_commands"]
                    .as_bool()
                    .unwrap_or(false);
                let args = vec![];

                thread::spawn(move || {
                    let mut child = run_script::spawn_script!(&exec.as_str(), &args, &options)
                        .expect("Failed to execute command");

                    let stdout = match child.stdout.take() {
                        Some(e) => {
                            String::from_utf8(child_stream_to_vec(e).lock().expect("").to_owned()).expect("")
                        },
                        None => format!("No Standard Output"),
                    };

                    println!("Commands in \"{}\" section exited with the following output: {}", &section_name, &stdout);
                });
            }
        }
    }
}

fn child_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::new()));
    let vec = out.clone();

    thread::spawn(move || loop {
        let mut buf = [0];
        match stream.read(&mut buf) {
            Err(err) => {
                println!("{}] Error reading from stream: {}", line!(), err);
                break;
            }
            Ok(got) => {
                if got == 0 {
                    break;
                } else if got == 1 {
                    vec.lock().expect("!lock").push(buf[0])
                } else {
                    println!("{}] Unexpected number of bytes: {}", line!(), got);
                    break;
                }
            }
        }

        println!("Received from Stream: {}", buf[0]);
    });

    out
}