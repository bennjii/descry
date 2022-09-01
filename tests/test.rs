
use std::{process::{Command, Stdio}, fmt::Error, io::{BufReader, BufRead}};

use clap::ErrorKind;
use run_script::{ScriptOptions, IoOptions};
use descry::{self, child_stream_to_vec};
use colored::Colorize;

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn test_push() {
    let (_svr, config) = match descry::init("descry.yaml") {
        Ok(rtn) => {
            rtn
        },
        Err(err) => {
            panic!("{} Descry was unable to launch the server due to a error-handling related failure. Refer: {}", format!("Error:").red().bold(), err)
        },
    };

    let mut options = ScriptOptions::new();
    options.exit_on_error = config["settings"]["exit_on_error"]
        .as_bool()
        .unwrap_or(false);
    options.print_commands = config["settings"]["print_commands"]
        .as_bool()
        .unwrap_or(false);
    options.output_redirection = IoOptions::Pipe;
    // let args = vec![];

    // thread::spawn(move || {
        println!("Spawned Thread to Handle PUSH script.");

        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("!cat");

        // let mut child = run_script::spawn_script!(config["events"]["ping"].as_str().expect("Failed to obtain PING"), &args, &options)
        //     .expect("Failed to execute command");

        let out = child_stream_to_vec(child.stdout.take().expect("!stdout"));
        let err = child_stream_to_vec(child.stderr.take().expect("!stderr"));

        let output = String::from_utf8(out.lock().expect("Failed to obtain lock on output").to_owned()).expect("Failed to stringify output");
        let error = String::from_utf8(err.lock().expect("Failed to obtain lock on output").to_owned()).expect("Failed to stringify output");

        println!("Commands in \"{}\" section exited with the following output: {} \n and error: {}", "push", &output, &error);
    // });
}

#[test]
fn alt_test() -> Result<(), Error> {
    let stdout = Command::new("strace")
        .args(&["-p", ""])
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture standard output."))?;

    let reader = BufReader::new(stdout);
    
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("hello {}", line));

     Ok(())
}