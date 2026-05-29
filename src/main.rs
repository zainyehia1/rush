use rustyline::config::CompletionType;
use rustyline::{Config, Editor};

use crate::history::{load_history, save_history};
use crate::evaluator::evaluate_command;
use crate::parser::parse_args;
use crate::completer::LineCompleter;

mod completer;
mod parser;
mod evaluator;
mod history;

fn main() {
    let config = Config::builder().completion_type(CompletionType::List).history_ignore_dups(false).unwrap().auto_add_history(true).build();
    
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(LineCompleter::new()));

    let mut history: Vec<String> = Vec::new();
    load_history(&mut history);
    
    loop {
        let read_line = rl.readline("$ ");
        match read_line {
            Ok(line) => {
                let input = line.trim().to_string();
                history.push(input.clone());
                let args = parse_args(&input);
                if args.is_empty() {
                    continue;
                }
                if args[0] == "exit" {
                    save_history(&history);
                    break;
                }
                evaluate_command(&args, &mut history);
            }
            Err(_) => break
        }
    }
}
