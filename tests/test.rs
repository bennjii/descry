use std::thread;
use run_script::{ScriptOptions, IoOptions};
use descry;

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn test_push() {
    let (svr, config) = descry::init("descry.yaml");

    let mut options = ScriptOptions::new();
    options.exit_on_error = config["settings"]["exit_on_error"]
        .as_bool()
        .unwrap_or(false);
    options.print_commands = config["settings"]["print_commands"]
        .as_bool()
        .unwrap_or(false);
    options.output_redirection = IoOptions::Pipe;
    let args = vec![];

    assert_eq!(config["settings"]["host"], "0.0.0.0:8443");

    thread::spawn(move || {
        let child = run_script::spawn_script!("push", &args, &options)
            .expect("Failed to execute command");
        
        let handler = match child.stdout {
            Some(a) => {
                child_stream_to_vec(a)
            },
            None => {
                panic!("Failed to detect .stdout on spawned child handler");
            },
        };

        let output = String::from_utf8(handler.lock().expect("Failed to obtain lock on output").to_owned()).expect("Failed to stringify output");

        println!("Commands in \"{}\" section exited with the following output: {}", "push", &output);
    });
}