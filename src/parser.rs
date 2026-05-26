pub struct Redirect {
    pub operator: String,
    pub file: String,
    pub position: usize,
}

pub fn parse_args(input: &str) -> Vec<String>{
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

pub fn redirect(args: &[String]) -> Option<Redirect> {
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