use std::env;
use std::fs;
use std::io::{Read, Write};

pub fn load_history(history: &mut Vec<String>) {
    match env::var("HISTFILE") {
        Ok(path) => {
            let mut file = fs::File::open(path).unwrap();
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();

            for line in buffer.lines().filter(|l| !l.is_empty()) {
                history.push(line.to_string());
            }
        }
        Err(_) => ()
    }
}

pub fn save_history(history: &Vec<String>) {
    match env::var("HISTFILE") {
        Ok(path) => {
            let mut file = std::fs::File::create(path).unwrap();

            for line in history {
                file.write_all((line.to_string() + "\n").as_bytes()).unwrap();
            }
        }
        Err(_) => ()
    }
}