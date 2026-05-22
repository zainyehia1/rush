#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        input = input.trim().to_string();

        let args = parse_args(&input);
        let command = &args[0];
        
        if command == "exit" {
            break;
        }
        evaluate_command(&args);
    }
}

fn locate_executables(command: &str, path: &str) -> Option<path::PathBuf> {
    env::split_paths(&path).map(|dir| dir.join(command)).find(|path| path.is_file() && {
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    })
}

fn evaluate_command(args: &Vec<String>) {
    let builtin_commands = vec!["echo", "type", "exit"];
    let path = env::var("PATH").unwrap();
     
    match args[0].as_str() {
        "echo" => println!("{}", args[1..].join(" ")),
        "type" => {
            if builtin_commands.contains(&args[1].as_str()) {
                println!("{} is a shell builtin", &args[1])
            } else if let Some(path) = locate_executables(&args[1], &path) {
                println!("{} is {}", &args[1], path.display())
            } else {
                println!("{}: not found", &args[1])
            }
        },
        _ => {
            if locate_executables(args[0].as_str(), &path).is_some() {
                std::process::Command::new(&args[0].as_str()).args(&args[1..]).spawn().unwrap().wait().unwrap();
            } else {
                println!("{}: command not found", args[0])
            }
        }
    }
}

fn parse_args(input: &str) -> Vec<String>{
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut next_is_escaped = false;
    let mut args = Vec::new();
    let mut current = String::new();
    
    for char in input.chars() {
        if next_is_escaped {
            current.push(char);
            next_is_escaped = false;
        } else {
            match char {
                '\\' => next_is_escaped = true, 
                '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
                '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
                char if char.is_whitespace() && !in_single_quotes && !in_double_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                },
                
                char => current.push(char),
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }
    
    args
}
