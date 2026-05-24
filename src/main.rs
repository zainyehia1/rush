#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;

struct Redirect {
    operator: String,
    file: String,
    position: usize,
}

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
        // println!("{args:?}");
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

fn evaluate_command(args: &[String]) {
    let builtin_commands = vec!["echo", "type", "exit"];
    let path = env::var("PATH").unwrap();

    let redirect = redirect(args);
    let command_args = match redirect {
        Some(ref r) => &args[..r.position],
        None => &args
    };
    
    match command_args[0].as_str() {
        "echo" => {
            if let Some(r) = &redirect {
                if r.operator == "1>" || r.operator == ">" {
                    let mut file = std::fs::File::create(&r.file).unwrap();
                    file.write_all((command_args[1..].join(" ") + "\n").as_bytes()).unwrap();
                } 
            } else {
                println!("{}", command_args[1..].join(" "))
            }
        },
        "type" => {
            let output = if builtin_commands.contains(&command_args[1].as_str()) {
                    format!("{} is a shell builtin", &command_args[1])
                } else if let Some(path) = locate_executables(&command_args[1], &path) {
                    format!("{} is {}", &command_args[1], path.display())
                } else {
                    format!("{}: not found", &command_args[1])
                };
            
            if let Some(r) = &redirect {
                if r.operator == "1>" || r.operator == ">" {
                    let mut file = std::fs::File::create(&r.file).unwrap();
                    file.write_all(output.as_bytes()).unwrap();
                } 
            } else {            
                println!("{output}");   
            }
        },
        _ => {
            if locate_executables(command_args[0].as_str(), &path).is_some() {
                let mut command = std::process::Command::new(&command_args[0].as_str());
                command.args(&command_args[1..]);
                
                if let Some(r) = &redirect {
                    if r.operator == "1>" || r.operator == ">" {
                        let mut file = std::fs::File::create(&r.file).unwrap();
                        command.stdout(file);

                        command.spawn().unwrap().wait().unwrap();
                    }
                } else {
                    std::process::Command::new(&command_args[0].as_str()).args(&command_args[1..]).spawn().unwrap().wait().unwrap();
                }
            } else {            
                 println!("{}: command not found", command_args[0])
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
            if in_double_quotes {
                match char {
                    '"' | '$' | '\\' | '\n' => current.push(char),
                    _ => {
                        current.push('\\');
                        current.push(char);
                    }
                }
            } else {
                current.push(char);
            }
            next_is_escaped = false;
        } else {
            match char {
                '>' if !in_single_quotes && !in_double_quotes => {
                    current.push(char);
                    args.push(current.clone());
                    current.clear();
                }
                '\\' => {
                    if in_single_quotes {
                        current.push(char);
                    } else {
                        next_is_escaped = true;
                    }
                }, 
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

fn redirect(args: &[String]) -> Option<Redirect> {
    let operator_position = args.iter().position(|x| x == ">" || x == "1>" || x == "2>")?;

    if operator_position + 1 == args.len() {
        println!("syntax error near unexpected token `newline'");
        return None;
    }

    Some(Redirect { 
        operator: args[operator_position].clone(), 
        file: args[operator_position + 1].clone(), 
        position: operator_position,
    })
}