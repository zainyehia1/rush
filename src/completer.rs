use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::validate::Validator;
use rustyline::{Helper, highlight::Highlighter, hint::Hinter};

use std::collections::HashMap;
use std::env;
use std::fs;
use crate::evaluator::BUILTIN_COMMANDS;
use crate::parser::parse_args;

pub struct LineCompleter {
    filename_completer: FilenameCompleter,
    pub registered_completions: HashMap<String, String>,
}

impl LineCompleter {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            registered_completions: HashMap::new()
        }
    }
}

impl Completer for LineCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self, 
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)>
    {
        let input = &line[..pos];
        
        if !input.contains(' ') { // if we're on the first word (command)
            let mut commands: Vec<String> = BUILTIN_COMMANDS.iter().map(|s| s.to_string()).collect();
            commands.extend(get_path_executables());
            
            let candidates = commands.iter().filter(|c| c.starts_with(input)).map(|c| Pair {display: c.to_string(), replacement: c.to_string() + " "}).collect();
            Ok((0, candidates))
        } else {
            let command = input.split_whitespace().next().unwrap_or("");
            if self.registered_completions.contains_key(command) {
                let argv = parse_args(input);
                let current_word = argv.last().map(|s| s.as_str()).unwrap_or("");
                let previous_word = argv.get(argv.len().saturating_sub(2)).map(|s| s.as_str()).unwrap_or("");
                
                if let Ok(script_output) = std::process::Command::new(self.registered_completions.get(command).unwrap())
                    .args(vec![command, current_word, previous_word])
                    .env("COMP_LINE", line)
                    .env("COMP_POINT", pos.to_string())
                    .output() {
                    let stdout = String::from_utf8_lossy(&script_output.stdout);
                    let candidates = stdout.lines().map(|line| Pair {
                        display: line.to_string(),
                        replacement: line.to_string() + " "
                    }).collect();

                    let start = line.rfind(' ').unwrap_or(pos) + 1;
                    return Ok((start, candidates));
                }
                
            }
            
            let (pos, pairs) = self.filename_completer.complete(line, pos, ctx)?;
            let pairs = pairs.into_iter().map(|p| Pair {
                display: if std::path::Path::new(&p.display).is_dir() {
                    p.display + "/"
                } else {
                    p.display + " "
                },
                replacement: if std::path::Path::new(&p.replacement).is_dir() {
                    p.replacement
                } else {
                    p.replacement + " "
                }
            }).collect();
            Ok((pos, pairs))
        }
                
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
