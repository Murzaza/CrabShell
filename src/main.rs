use std::io::{self, Write};
use std::ffi::{CStr};
use std::convert::TryInto;
use std::process;
use std::path::Path;

use nix::unistd::{fork, execvp, chdir, ForkResult, Pid};
use nix::sys::wait::*;
use nix::sys::signal::Signal::SIGKILL;
use nix::NixPath;

struct CrabShell {
    name: String,
    version: String
}

fn setup() -> CrabShell {
   CrabShell{name: String::from("CrabShell🦀"), version: String::from("0.1.0")}
}

fn print_prompt() {
    print!("> ");
    io::stdout().flush().expect("Could not flush STDOUT");
}

fn wait_for_child(child: Pid) {
    loop {
        match waitpid(child, Some(WaitPidFlag::WUNTRACED)) {
            Ok(status) => {
                if status == WaitStatus::Exited(child, 0) || status == WaitStatus::Signaled(child, SIGKILL, false) {
                   break;
                }
            },
            Err(_) => {
                eprintln!("Unable to wait for pid {}", child);
                break;
            }
        }
    }
}

fn exec_command(command: &str, arguments: &[&str]) {
    // Convert command to CStr
    let cmd = CStr::from_bytes_with_nul(command.as_bytes()).expect("Unable to convert cmd to CStr");
    let mut args:&[&CStr] = &[CStr::from_bytes_with_nul(b"\0").unwrap()];

    // Convert args to CStr
    let accumulator: Vec<String> = arguments.iter().map(|&e| String::from(e) + "\0").collect();
    let mut arg_holder: Vec<&CStr> = accumulator.iter().map(|e| CStr::from_bytes_with_nul(e.as_bytes()).expect("Unable to convert args to CStr")).collect();

    if arguments.len() != 0 {
        args = arg_holder.as_slice();
    }

    match execvp(cmd, args) {
        Ok(_) => {},
        Err(error) => eprintln!("Unable to call execvp: {:?}", error)
    };
}

fn launch(config: &CrabShell, parts: Vec<&str>) {
    let cmd = parts.first().unwrap();
    let command = String::from(*cmd) + "\0";
    match *cmd {
        "exit" => exit(),
        "help" => print_help(&config),
        "cd" => cd(parts.as_slice()),
        _ => {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child, .. }) => { wait_for_child(child) },
                Ok(ForkResult::Child) => { exec_command(command.as_ref(), parts.as_slice()) },
                Err(_) => { eprintln!("Failure forking process") }
            };
        }
    };
}

fn run_loop(config: CrabShell) {
    let mut line = String::from("");

    loop {
        // Print the prompt.
        print_prompt();

        // Read the line from stdin.
        io::stdin().read_line(&mut line);
        let parsed_line = line.trim_end().split(" ");
        launch(&config, parsed_line.collect());
        line.clear();
    }
}

fn cd(args: &[&str]) {
    if args.len() < 2 {
        eprintln!("Please specify a directory");
        return;
    }

    let cd_path = Path::new(args.get(1).unwrap());
    match chdir(cd_path) {
        Ok(_) => {},
        Err(error) => eprintln!("Unable to change directories: {:?}", error)
    }
}

fn print_help(config: &CrabShell) {
    println!("Welcome to {},\n version: {},\n A crappy little shell", config.name, config.version);
}

fn exit() {
    process::exit(0);
}

fn main() {
    // Load config.
    let rsh_config = setup();

    // Run command loop.
    run_loop(rsh_config);

    // Teardown.
    exit();
}
