
use std::{process::{Command, Stdio}, io::{BufReader, BufRead, Error, ErrorKind}, thread};

use run_script::{ScriptOptions, IoOptions};
use descry::{self, child_stream_to_vec};
use colored::Colorize;

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn test_push() -> Result<(), Error> {
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

    let handler = thread::spawn(move || {
        println!("Spawned Thread to Handle PUSH script.");

        let stdout = Command::new("sh")
            .args(["-C", &format!("scripts/{}.sh", "push")])
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

    Ok(())
}

#[test]
fn alt_test() -> Result<(), Error> {
    let stdout = Command::new("sh")
        .args(["-C", "scripts/test.sh"])
        .stdout(Stdio::piped())
        .spawn().expect("Could not spawn child process")
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other,"Could not capture standard output."))?;

    let reader = BufReader::new(stdout);
    
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("hello {}", line));

    Ok(())
}