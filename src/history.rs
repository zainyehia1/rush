use std::env;
use std::fs;
use std::io::Read;

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
