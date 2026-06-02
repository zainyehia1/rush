use std::collections::HashMap;
use std::io::{Read, Write};
use std::env;
use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command};

use crate::parser;

pub const BUILTIN_COMMANDS: [&str; 9] = ["echo", "type", "exit", "pwd", "cd", "history", "complete", "jobs", "declare"];

pub struct Job {
    id: usize,
    child: Child,
    command: String,
}

pub fn evaluate_command(args: &[String], history: &mut Vec<String>, completions: &mut HashMap<String, String>, jobs: &mut Vec<Job>) {
    let path = env::var("PATH").unwrap_or_default();

    let redirect = parser::redirect(args);
    let mut command_args = match redirect {
        Some(ref r) => &args[..r.position],
        None => args
    };

    let background = command_args.last().map(|s| s.as_str()) == Some("&");
    if background {
        command_args = &command_args[..command_args.len() - 1];
    }
    
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
            let output = if BUILTIN_COMMANDS.contains(&command_args[1].as_str()) {
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
        "history" => {
            if command_args.len() == 1 {
                for (i,line) in history.iter().enumerate() {
                    println!("{} {line}", i + 1);
                }
            } else if command_args.len() == 2 {
                let entries = command_args[1].parse::<usize>().unwrap_or(0);
                if entries > history.len() {
                    let mut i = 1;
                    for line in history {
                        println!("{} {line}", i + 1);
                        i += 1;
                    }
                } else {
                    let start = history.len() - entries;
                    for (i,line) in history[start..].iter().enumerate() {
                        println!("\t{} {line}", start + i + 1);
                    }
                }
            } else if command_args.len() == 3 {
                if command_args[1] == "-r" {
                    let mut file = fs::File::open(&command_args[2]).unwrap();
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer).unwrap();

                    for line in buffer.lines().filter(|l| !l.is_empty()) {
                        history.push(line.to_string());
                    }
                } else if command_args[1] == "-w" {
                    let mut file = std::fs::File::create(&command_args[2]).unwrap();
                    for line in history {
                        file.write_all((line.to_string() + "\n").as_bytes()).unwrap();
                    }
                } else if command_args[1] == "-a" {
                    let mut file = fs::OpenOptions::new().append(true).create(true).open(&command_args[2]).unwrap();

                    let start = history[..history.len() - 1].iter().enumerate().rev()
                        .find(|(_pos, line)| *line == &format!("history -a {}", command_args[2]))
                        .map(|(pos, _line)| pos + 1)
                        .unwrap_or(0);
                    
                    for line in &history[start..] {
                        file.write_all((line.to_string() + "\n").as_bytes()).unwrap();
                    }
                }
            }
        },
        "complete" => {
            if command_args.len() == 4 && command_args[1] == "-C" {
                completions.insert(String::from(&command_args[3]), String::from(&command_args[2]));
            } else if command_args.len() == 3 {
                if command_args[1] == "-p" {
                    if completions.contains_key(&command_args[2]) {
                        println!("complete -C '{}' {}", completions.get(&command_args[2]).unwrap(), command_args[2])
                    } else {
                        println!("complete: {}: no completion specification", command_args[2])
                    }
                } else if command_args[1] == "-r" {
                    if completions.contains_key(&command_args[2]) {
                        completions.remove(&command_args[2]);
                    }
                }
            }
        },
        "jobs" => {
            let len = jobs.len();
            for (i, job) in jobs.iter_mut().enumerate() {
                let status = job.child.try_wait().unwrap();
                let marker = if i == len.saturating_sub(1) {
                    '+'
                } else if i == len.saturating_sub(2) {
                    '-'
                } else {
                    ' '
                };
                if status.is_none() {
                    println!("[{}]{}  Running                 {}", job.id, marker, job.command);
                } else if status.is_some() {
                    println!("[{}]{}  Done                    {}", job.id, marker, job.command.trim_end_matches(" &"));
                }
            }
            jobs.retain_mut(|job| job.child.try_wait().unwrap().is_none()); // remove finished commands
        },
        _ => {
            if locate_executables(command_args[0].as_str(), &path).is_some() {
                let mut command = std::process::Command::new(command_args[0].as_str());
                command.args(&command_args[1..]);
                let command_str = command_args.join(" ") + " &";
                
                if let Some(r) = &redirect {
                    if r.operator == "1>" || r.operator == ">" {
                        let file = std::fs::File::create(&r.file).unwrap();
                        command.stdout(file);

                        run_command(&mut command, background, jobs, &command_str);
                    } else if r.operator == "2>" {
                        let file = std::fs::File::create(&r.file).unwrap();
                        command.stderr(file);
                        
                        run_command(&mut command, background, jobs, &command_str);
                    } else if r.operator == "1>>" || r.operator == ">>" {
                       command.stdout(fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap());

                       run_command(&mut command, background, jobs, &command_str);
                    } else if r.operator == "2>>" {
                       command.stderr(fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap());

                       run_command(&mut command, background, jobs, &command_str);
                    }
                } else {
                    run_command(&mut command, background, jobs, &command_str);
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

fn run_command(command: &mut Command, background: bool, jobs: &mut Vec<Job>, command_str: &String) {
    if background {
        let child = command.spawn().unwrap();
        let job = Job {id: jobs.len() +  1, child: child, command: command_str.to_string()};
        println!("[{}] {}", job.id, job.child.id());
        jobs.push(job);
    } else {
        command.spawn().unwrap().wait().unwrap();
    }
}

pub fn reap_finished(jobs: &mut Vec<Job>) {
    let len = jobs.len();
    
    for (i, job) in jobs.iter_mut().enumerate() {
        let marker = if i == len.saturating_sub(1) {
            '+'
        } else if i == len.saturating_sub(2) {
            '-'
        } else {
            ' '
        };
        if job.child.try_wait().unwrap().is_some(){
            println!("[{}]{}  Done                    {}", job.id, marker, job.command.trim_end_matches(" &"));
        }
    }
    
    jobs.retain_mut(|job| job.child.try_wait().unwrap().is_none()); // remove finished commands
}