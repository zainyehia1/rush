use std::io::{self, Write};
use std::env;
use std::path;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let builtin_commands = vec!["echo", "type", "exit"];
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        input = input.trim().to_string();
        let path = env::var("PATH").unwrap();

        let args: Vec<&str> = input.split_whitespace().collect();
        
        if args[0] == "exit" {
            break;
        } else if args[0] == "echo" {
            println!("{}", &input[5..]);
        } else if args[0] == "type" {
            if builtin_commands.contains(&args[1]) {
                println!("{} is a shell builtin", &args[1]);
            } else if let Some(path) = locate_executables(&args[1], &path){
                println!("{} is {}", &args[1], path.display())
            } else {
                println!("{}: not found", &args[1]);
            }
        } else if locate_executables(args[0], &path).is_some() {
            std::process::Command::new(args[0]).args(&args[1..]).spawn().unwrap().wait().unwrap();
        }
            else {
            println!("{}: command not found", input.trim());
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