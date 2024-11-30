use std::{
    collections::HashMap,
    fs,
    io::{stdout, Write}, task::Wake,
};

use anyhow::{Error, Result};

#[derive(Debug, Clone)]
pub enum Token {
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    Number(f32),
    Variable(String),
    Power,
    Ln,
    Log,
    Sin,
    Cos,
    Tan,
}

pub fn produce_tokens(expr: String) -> Result<Vec<Token>> {
    use Token::*;

    let mut chars = expr.chars().peekable();
    let mut tokens = vec![];
    while let Some(c) = chars.next() {
        match c {
            '+' => tokens.push(Plus),
            '-' => tokens.push(Minus),
            '*' => tokens.push(Star),
            '/' => tokens.push(Slash),
            '^' => tokens.push(Power),
            '(' | '[' => tokens.push(LeftParen),
            ')' | ']' => tokens.push(RightParen),
            d if d.is_ascii_digit() => {
                let mut num = d.to_string();
                while let Some(n_char) = chars.peek() {
                    if n_char.is_ascii_digit() || n_char == &'.' {
                        num.push(*n_char);
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(Number(num.parse::<f32>()?));
            }
            c if c.is_ascii_alphanumeric() => {
                let mut string = c.to_string();
                while let Some(n_char) = chars.peek() {
                    if n_char.is_ascii_alphanumeric() {
                        string.push(*n_char);
                        chars.next();
                    } else {
                        break;
                    }
                }

                match string.as_str() {
                    "ln" => tokens.push(Ln),
                    "sin" => tokens.push(Sin),
                    "cos" => tokens.push(Cos),
                    "tan" => tokens.push(Tan),
                    "log" => tokens.push(Log),
                    _ => {
                        if string.len() > 1 {
                            return Err(Error::msg("Variable length cannot exceed 1"));
                        }
                        tokens.push(Variable(string))
                    }
                }
            }
            ' ' | '\n' | '\r' => {}
            _ => return Err(Error::msg("bad expr")),
        }
    }

    Ok(tokens)
}

// E = T + E
// E = T - E
// E = T
// T = F * T
// T = F / T
// T = F
// F = F^G
// G = Variable
// G = Number
// G = (E)
// G = ( - | ln | log | sin | cos | tan ) G
fn parse_tokens(){

}

// parser functions
 fn expression(){
     return term()
 }

fn term(){
    let expr = factor();
}
fn populate_variables(tokens: Vec<Token>) -> Result<Vec<Token>> {
    let mut map: HashMap<String, f32> = HashMap::new();
    let mut populated = vec![];
    for token in tokens.iter() {
        if let Token::Variable(identifer) = token {
            let value = map.get(identifer);
            if let Some(value) = value {
                populated.push(Token::Number(*value));
            } else {
                let mut var_value = String::new();
                print!("Set value [{}]: ", identifer);
                stdout().flush()?;
                std::io::stdin().read_line(&mut var_value)?;
                let parsed = var_value.trim().parse::<f32>()?;
                map.insert(identifer.to_string(), parsed);
                populated.push(Token::Number(parsed))
            }
        } else {
            populated.push(token.clone())
        }
    }

    Ok(populated)
}
fn evaluate(expr: String) -> Result<()> {
    let tokens = produce_tokens(expr);
    let populated = populate_variables(tokens?)?;
    println!("{:?}", populated);
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let filepath = &args[1];
        if std::path::Path::new(filepath).exists() {
            let contents = fs::read_to_string(filepath)?;
            evaluate(contents)?;
        } else {
            println!("file does not exist. usage: cord [filename]")
        }
    } else {
        println!("usage: cord [filename]")
    }
    Ok(())
}
