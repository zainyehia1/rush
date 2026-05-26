#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use rustyline::Config;
use rustyline::config::CompletionType;
use rustyline::Editor;
use rustyline::validate::Validator;
use rustyline::{Helper, highlight::Highlighter, hint::Hinter};
use rustyline::completion::{Completer, Pair};

struct Redirect {
    operator: String,
    file: String,
    position: usize,
}

struct LineCompleter;

impl Completer for LineCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self, 
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)>
    {
        let builtin_commands = ["echo", "type", "exit"];
        let mut commands: Vec<String> = builtin_commands.iter().map(|s| s.to_string()).collect();
        commands.extend(get_path_executables());
        
        let input = &line[..pos];
        let candidates = commands.iter().filter(|c| c.starts_with(input)).map(|c| Pair {display: c.to_string(), replacement: c.to_string() + " "}).collect();
        
        Ok((0, candidates))
    }
}
impl Validator for LineCompleter {
    
}

impl Hinter for LineCompleter {
    type Hint = String;
}

impl Highlighter for LineCompleter {
    
}

impl Helper for LineCompleter {
    
}

fn main() {
    let config = Config::builder().completion_type(CompletionType::List).build();
    
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(LineCompleter));
    
    loop {
        let read_line = rl.readline("$ ");
        match read_line {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap();
                let input = line.trim().to_string();
                let args = parse_args(&input);
                if args.is_empty() {
                    continue;
                }
                if args[0] == "exit" {
                    break;
                }
                evaluate_command(&args);
            }
            Err(_) => break
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

fn evaluate_command(args: &[String]) {
    let builtin_commands = ["echo", "type", "exit"];
    let path = env::var("PATH").unwrap_or_default();

    let redirect = redirect(args);
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


fn parse_args(input: &str) -> Vec<String>{
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut next_is_escaped = false;
    let mut args = Vec::new();
    let mut current = String::new();

    let mut chars = input.chars().peekable();
    
    while let Some(char) = chars.next() {
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
                    let operator = if chars.peek() == Some(&'>'){
                        chars.next();
                        ">>"
                    } else {
                        ">"
                    };
                    
                    current.push_str(operator);
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
    let operator_position = args.iter().position(|x| x == ">" || x == "1>" || x == "2>" || x == ">>" || x == "1>>" || x == "2>>")?;

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

fn get_path_executables() -> Vec<String> {
    let path = env::var("PATH").unwrap_or_default();
    env::split_paths(&path).flat_map(|dir| {
        fs::read_dir(dir).into_iter().flatten().filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
    }).collect()
}