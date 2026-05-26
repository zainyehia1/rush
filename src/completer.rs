use rustyline::completion::{Completer, Pair};
use rustyline::validate::Validator;
use rustyline::{Helper, highlight::Highlighter, hint::Hinter};

use std::env;
use std::fs;
use crate::evaluator::BUILTIN_COMMANDS;

pub struct LineCompleter;

impl Completer for LineCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self, 
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)>
    {
        let mut commands: Vec<String> = BUILTIN_COMMANDS.iter().map(|s| s.to_string()).collect();
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

