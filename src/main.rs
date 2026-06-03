use crate::shell::Shell;

mod completer;
mod parser;
mod utils;
mod history;
mod shell;

fn main() {
    Shell::new().run_repl();
}
