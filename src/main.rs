use rustyline::config::CompletionType;
use rustyline::{Config, Editor};

mod completer;
mod parser;
mod evaluator;

fn main() {
    let config = Config::builder().completion_type(CompletionType::List).history_ignore_dups(false).unwrap().auto_add_history(true).build();
    
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(completer::LineCompleter));
    
    loop {
        let read_line = rl.readline("$ ");
        match read_line {
            Ok(line) => {
                let input = line.trim().to_string();
                let args = parser::parse_args(&input);
                if args.is_empty() {
                    continue;
                }
                if args[0] == "exit" {
                    break;
                }
                let history: Vec<String> = rl.history().iter().map(|s| s.to_string()).collect();
                evaluator::evaluate_command(&args, &history);
            }
            Err(_) => break
        }
    }
}
