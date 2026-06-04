use std::io::{Read, Write};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::fs;
use std::env;

use rustyline::config::CompletionType;
use rustyline::{Config, Editor};

use crate::history::{load_history, save_history};
use crate::parser;
use crate::utils::locate_executables;
use crate::completer::LineCompleter;

pub const BUILTIN_COMMANDS: [&str; 9] = ["echo", "type", "exit", "pwd", "cd", "history", "complete", "jobs", "declare"];

pub struct Job {
    id: usize,
    child: Child,
    command: String,
}


pub struct Shell {
    history: Vec<String>,
    completions: HashMap<String, String>,
    jobs: Vec<Job>,
    variables: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            completions: HashMap::new(),
            jobs: Vec::new(),
            variables: HashMap::new()
        }
    }

    fn run_command(&mut self, background: bool, command: &mut Command, command_string: String) {
        if background {
            let child = command.spawn().unwrap();
            let job = Job {id: self.jobs.len() +  1, child, command: command_string};
            println!("[{}] {}", job.id, job.child.id());
            self.jobs.push(job);
        } else {
            command.spawn().unwrap().wait().unwrap();
        }
    }

    fn reap_finished(&mut self) {
        let len = self.jobs.len();
        
        for (i, job) in self.jobs.iter_mut().enumerate() {
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
        
        self.jobs.retain_mut(|job| job.child.try_wait().unwrap().is_none());
    }

    fn evaluate_command(&mut self, args: &[String]) {
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

        let mut expanded_command_args: Vec<String> = Vec::new();
        for arg in command_args {
            if arg.contains("${") && arg.contains('}'){
                let vars: Vec<&str> = arg.split("${").collect();
                let literal = vars[0];

                let mut expanded_var = String::new();
                expanded_var.push_str(literal);

                for var in &vars[1..] {
                    if var.contains('}') {
                        let (var_name, rest) = var.split_once('}').unwrap();
                        if let Some(value) = self.variables.get(var_name) {
                            expanded_var.push_str(value);
                        } else {
                            expanded_var.push_str("");
                        }
                        expanded_var.push_str(rest);
                    }
                }
                if !expanded_var.is_empty() {
                    expanded_command_args.push(expanded_var);
                }
            } else if arg.contains('$') {
                let vars: Vec<&str> = arg.split('$').collect();
                let literal = vars[0];

                let mut expanded_var = String::new();
                expanded_var.push_str(literal);
                
                for var in &vars[1..] { // all potential variables (prefixed with '$')
                    if self.variables.contains_key(*var) { 
                        expanded_var.push_str(self.variables.get(*var).unwrap());
                    }
                }
                if !expanded_var.is_empty() {
                    expanded_command_args.push(expanded_var);
                }
            } else {
                expanded_command_args.push(arg.to_owned());
            }
        }
        
        match expanded_command_args[0].as_str() {
            "echo" => {
                if let Some(r) = &redirect {
                    if r.operator == "1>" || r.operator == ">" {
                        let mut file = std::fs::File::create(&r.file).unwrap();
                        file.write_all((expanded_command_args[1..].join(" ") + "\n").as_bytes()).unwrap();
                    } else if r.operator == "2>" {
                        std::fs::File::create(&r.file).unwrap();
                        println!("{}", expanded_command_args[1..].join(" "))
                    } else if r.operator == "1>>" || r.operator == ">>" {
                       let mut file = fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap();
                       file.write_all((expanded_command_args[1..].join(" ") + "\n").as_bytes()).unwrap();
                    } else if r.operator == "2>>" {
                        println!("{}", expanded_command_args[1..].join(" "))
                    }
                } else {
                    println!("{}", expanded_command_args[1..].join(" "))
                }
            },
            "type" => {
                let output = if BUILTIN_COMMANDS.contains(&expanded_command_args[1].as_str()) {
                        format!("{} is a shell builtin", &expanded_command_args[1])
                    } else if let Some(path) = locate_executables(&command_args[1], &path) {
                        format!("{} is {}", &expanded_command_args[1], path.display())
                    } else {
                        format!("{}: not found", &expanded_command_args[1])
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
                if expanded_command_args[1].starts_with("/") {
                    env::set_current_dir(&expanded_command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", expanded_command_args[1]));
                } else if expanded_command_args[1].starts_with("./") {
                    env::set_current_dir(&expanded_command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", expanded_command_args[1]));
                } else if expanded_command_args[1].starts_with("../") {
                    env::set_current_dir(&expanded_command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", expanded_command_args[1]));
                } else if expanded_command_args[1] == "~" {
                    let home = env::var("HOME").unwrap_or_default();
                    env::set_current_dir(home).unwrap_or_else(|_| println!("cd: {}: No such file or directory", expanded_command_args[1]));
                } else if expanded_command_args[1].starts_with("~"){
                    let home = env::var("HOME").unwrap_or_default();
                    env::set_current_dir(expanded_command_args[1].replacen("~", &home, 1)).unwrap_or_else(|_| println!("cd: {}: No such file or directory", expanded_command_args[1]));
                } else {
                    env::set_current_dir("./".to_owned() + &expanded_command_args[1]).unwrap_or_else(|_| println!("cd: {}: No such file or directory", expanded_command_args[1]));
                }
            }
            "history" => {
                if expanded_command_args.len() == 1 {
                    for (i,line) in self.history.iter().enumerate() {
                        println!("{} {line}", i + 1);
                    }
                } else if expanded_command_args.len() == 2 {
                    let entries = expanded_command_args[1].parse::<usize>().unwrap_or(0);
                    if entries > self.history.len() {
                        for (i, line) in self.history.iter().enumerate() {
                            println!("{} {line}", i + 1);
                        }
                    } else {
                        let start = self.history.len() - entries;
                        for (i,line) in self.history[start..].iter().enumerate() {
                            println!("\t{} {line}", start + i + 1);
                        }
                    }
                } else if expanded_command_args.len() == 3 {
                    if expanded_command_args[1] == "-r" {
                        let mut file = fs::File::open(&expanded_command_args[2]).unwrap();
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer).unwrap();
    
                        for line in buffer.lines().filter(|l| !l.is_empty()) {
                            self.history.push(line.to_string());
                        }
                    } else if expanded_command_args[1] == "-w" {
                        let mut file = std::fs::File::create(&expanded_command_args[2]).unwrap();
                        for line in &self.history {
                            file.write_all((line.to_string() + "\n").as_bytes()).unwrap();
                        }
                    } else if expanded_command_args[1] == "-a" {
                        let mut file = fs::OpenOptions::new().append(true).create(true).open(&expanded_command_args[2]).unwrap();
    
                        let start = self.history[..self.history.len() - 1].iter().enumerate().rev()
                            .find(|(_pos, line)| *line == &format!("history -a {}", expanded_command_args[2]))
                            .map(|(pos, _line)| pos + 1)
                            .unwrap_or(0);
                        
                        for line in &self.history[start..] {
                            file.write_all((line.to_string() + "\n").as_bytes()).unwrap();
                        }
                    }
                }
            },
            "complete" => {
                if expanded_command_args.len() == 4 && expanded_command_args[1] == "-C" {
                    self.completions.insert(String::from(&expanded_command_args[3]), String::from(&expanded_command_args[2]));
                } else if expanded_command_args.len() == 3 {
                    if expanded_command_args[1] == "-p" {
                        if self.completions.contains_key(&expanded_command_args[2]) {
                            println!("complete -C '{}' {}", self.completions.get(&expanded_command_args[2]).unwrap(), expanded_command_args[2])
                        } else {
                            println!("complete: {}: no completion specification", expanded_command_args[2])
                        }
                    } else if expanded_command_args[1] == "-r" {
                        if self.completions.contains_key(&expanded_command_args[2]) {
                            self.completions.remove(&expanded_command_args[2]);
                        }
                    }
                }
            },
            "jobs" => {
                let len = self.jobs.len();
                for (i, job) in self.jobs.iter_mut().enumerate() {
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
                self.jobs.retain_mut(|job| job.child.try_wait().unwrap().is_none()); // remove finished commands
            },
            "declare" => {
                if expanded_command_args.len() == 2 && expanded_command_args[1].contains("=") {
                    let (variable_name, value) = expanded_command_args[1].split_once("=").unwrap();

                    if variable_name.starts_with(|c: char| c.is_digit(10)) || variable_name.chars().any(|c: char| !c.is_alphanumeric() && c != '_'){
                        println!("declare: `{}': not a valid identifier", expanded_command_args[1]);
                    } else {
                        self.variables.insert(variable_name.to_string(), value.to_string());
                    }
                } else if expanded_command_args.len() == 3 {
                    if expanded_command_args[1] == "-p" {
                        if self.variables.contains_key(&expanded_command_args[2]) {
                            println!("declare -- {}=\"{}\"", expanded_command_args[2], self.variables.get(&expanded_command_args[2]).unwrap())
                        } else {
                            println!("declare: {}: not found", expanded_command_args[2])
                        }
                    }
                }
            },
            _ => {
                if locate_executables(expanded_command_args[0].as_str(), &path).is_some() {
                    let mut command = std::process::Command::new(expanded_command_args[0].as_str());
                    command.args(&expanded_command_args[1..]);
                    let command_str = expanded_command_args.join(" ") + " &";
                    
                    if let Some(r) = &redirect {
                        if r.operator == "1>" || r.operator == ">" {
                            let file = std::fs::File::create(&r.file).unwrap();
                            command.stdout(file);
    
                            self.run_command(background, &mut command, command_str);
                        } else if r.operator == "2>" {
                            let file = std::fs::File::create(&r.file).unwrap();
                            command.stderr(file);
                            
                            self.run_command(background, &mut command, command_str);
                        } else if r.operator == "1>>" || r.operator == ">>" {
                           command.stdout(fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap());
    
                           self.run_command(background, &mut command, command_str);
                        } else if r.operator == "2>>" {
                           command.stderr(fs::OpenOptions::new().append(true).create(true).open(&r.file).unwrap());
    
                           self.run_command(background, &mut command, command_str);
                        }
                    } else {
                        self.run_command(background, &mut command, command_str);
                    }
                } else {            
                     println!("{}: command not found", expanded_command_args[0])
                }   
            }
        }
    }

    pub fn run_repl(&mut self) {
        let config = Config::builder().completion_type(CompletionType::List).history_ignore_dups(false).unwrap().auto_add_history(true).build();
        
        let mut rl = Editor::with_config(config).unwrap();
        rl.set_helper(Some(LineCompleter::new()));
    
        load_history(&mut self.history);
    
        loop {
            self.reap_finished();
            let read_line = rl.readline("$ ");
            match read_line {
                Ok(line) => {
                    let input = line.trim().to_string();
                    self.history.push(input.clone());
                    let args = parser::parse_args(&input);
                    if args.is_empty() {
                        continue;
                    }
                    if args[0] == "exit" {
                        save_history(&self.history);
                        break;
                    }
                    self.evaluate_command(&args);
                    
                    if let Some(completion) = rl.helper_mut() {
                        completion.registered_completions = self.completions.clone();
                    }
                }
                Err(_) => break
            }
        }
    }
}