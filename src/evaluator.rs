use std::io::Write;
use std::env;
use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::parser;
// use crate::parser::

pub fn evaluate_command(args: &[String]) {
    let builtin_commands = ["echo", "type", "exit", "pwd", "cd"];
    let path = env::var("PATH").unwrap_or_default();

    let redirect = parser::redirect(args);
    let command_args = match redirect {
        Some(ref r) => &args[..r.position],
        None => args
    };
    
    match command_args[0].as_str() {
        "echo" => {
            if let Some(r) = &redirect {
                if r.operator == "1>" || r.operator == ">" {
                    let mut file = std::fs::File::create(&r.file).unwrap();
                    file.write_all((command_args[1..].join(" ") + "\n").as_bytes()).unwrap();
                } else if r.operator == "2>" {
                    std::fs::File::create(&r.file).unwrap();
                    println!("{}", command_args[1..].join(" "))
                } else if r.operator == "1>>" || r.operator == ">>" {
                   let mut file = fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap();
                   file.write_all((command_args[1..].join(" ") + "\n").as_bytes()).unwrap();
                } else if r.operator == "2>>" {
                    println!("{}", command_args[1..].join(" "))
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
                } else if r.operator == "1>>" || r.operator == ">>" {
                   let mut file = fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap();
                   file.write_all(output.as_bytes()).unwrap();
                } 
            } else {            
                println!("{output}");   
            }
        },
        "pwd" => println!("{}", env::current_dir().unwrap().display()),
        "cd" => {
            if command_args[1].starts_with("/") {
                env::set_current_dir(&command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", command_args[1]));
            } else if command_args[1].starts_with("./") {
                env::set_current_dir(&command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", command_args[1]));
            } else if command_args[1].starts_with("../") {
                env::set_current_dir(&command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", command_args[1]));
            } else if command_args[1] == "~" {
                let home = env::var("HOME").unwrap_or_default();
                env::set_current_dir(home).unwrap_or_else(|_| println!("cd: {}: No such file or directory", command_args[1]));
            } else if command_args[1].starts_with("~"){
                let home = env::var("HOME").unwrap_or_default();
                env::set_current_dir(command_args[1].replacen("~", &home, 1)).unwrap_or_else(|_| println!("cd: {}: No such file or directory", command_args[1]));
            } else {
                env::set_current_dir("./".to_owned() + &command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", command_args[1]));
            }
        }
        _ => {
            if locate_executables(command_args[0].as_str(), &path).is_some() {
                let mut command = std::process::Command::new(command_args[0].as_str());
                command.args(&command_args[1..]);
                
                if let Some(r) = &redirect {
                    if r.operator == "1>" || r.operator == ">" {
                        let file = std::fs::File::create(&r.file).unwrap();
                        command.stdout(file);

                        command.spawn().unwrap().wait().unwrap();
                    } else if r.operator == "2>" {
                        let file = std::fs::File::create(&r.file).unwrap();
                        command.stderr(file);
                        command.spawn().unwrap().wait().unwrap();
                    } else if r.operator == "1>>" || r.operator == ">>" {
                       command.stdout(fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap());

                       command.spawn().unwrap().wait().unwrap();
                    } else if r.operator == "2>>" {
                       command.stderr(fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap());

                       command.spawn().unwrap().wait().unwrap();
                    }
                } else {
                    command.spawn().unwrap().wait().unwrap();
                }
            } else {            
                 println!("{}: command not found", command_args[0])
            }   
        }
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
