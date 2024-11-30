use std::{
    collections::HashMap,
    fs,
    io::{stdout, Write},
    iter::Peekable,
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
    Variable(String), // never seen by parser
    Power,
    Ln,
    Log,
    Sin,
    Cos,
    Tan,
}

// expr -> term
// term -> factor (("-" | "+") factor)*;
// factor -> power ( ( "/" | "*") power )*;
// power -> unary ( "^" unary)*;
// unary -> ("-" | "+" | "ln") | primary;
// primary -> Number | "(" expr ")"

#[derive(Debug, Clone)]
pub enum ParseExpr {
    Binary(Box<ParseExpr>, Token, Box<ParseExpr>),
    Unary(Token, Box<ParseExpr>),
    Value(Token),
}

impl ParseExpr {
    fn expr<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        Ok(Self::term(tokens)?)
    }

    fn term<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        let mut expr = Self::factor(tokens)?;
        while let Some(token) = tokens.peek() {
            if matches!(token, Token::Plus | Token::Minus) {
                let op = tokens.next();
                let right = Self::factor(tokens)?;
                expr = Self::Binary(Box::new(expr), op.unwrap(), Box::new(right))
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn factor<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        let mut expr = Self::power(tokens)?;
        while let Some(token) = tokens.peek() {
            if matches!(token, Token::Star | Token::Slash) {
                let op = tokens.next();
                let right = Self::power(tokens)?;
                expr = Self::Binary(Box::new(expr), op.unwrap(), Box::new(right))
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn power<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        let mut expr = Self::unary(tokens)?;
        while let Some(token) = tokens.peek() {
            if matches!(token, Token::Power) {
                let op = tokens.next();
                let right = Self::unary(tokens)?;
                expr = Self::Binary(Box::new(expr), op.unwrap(), Box::new(right))
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn unary<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        if let Some(token) = tokens.peek() {
            if matches!(
                token,
                Token::Minus
                    | Token::Plus
                    | Token::Ln
                    | Token::Log
                    | Token::Sin
                    | Token::Cos
                    | Token::Tan
            ) {
                let op = tokens.next();
                let right = Self::unary(tokens)?;
                return Ok(Self::Unary(op.unwrap(), Box::new(right)));
            }
        }
        Ok(Self::primary(tokens)?)
    }

    fn primary<I>(tokens: &mut Peekable<I>) -> Result<Self>
    where
        I: Iterator<Item = Token>,
    {
        if let Some(token) = tokens.peek() {
            match token {
                Token::Number(_) => return Ok(Self::Value(tokens.next().unwrap())),
                Token::LeftParen => {
                    tokens.next();
                    let expr = Self::expr(tokens)?;
                    if !matches!(tokens.next(), Some(Token::RightParen)) {
                        return Err(Error::msg("Unclosed bracket"));
                    }
                    return Ok(expr);
                }
                _ => {}
            }
        }
        Err(Error::msg("parser failed"))
    }

    fn eval(&self) -> Result<f32> {
        Ok(match self {
            Self::Binary(left, o, right) => match o {
                Token::Plus => left.eval()? + right.eval()?,
                Token::Star => left.eval()? * right.eval()?,
                Token::Slash => left.eval()? / right.eval()?,
                Token::Minus => left.eval()? - right.eval()?,
                Token::Power => left.eval()?.powf(right.eval()?),
                _ => return Err(Error::msg("Invalid binary operand.")),
            },
            Self::Unary(o, expr) => match o {
                Token::Minus => -expr.eval()?,
                Token::Plus => expr.eval()?,
                Token::Sin => expr.eval()?.sin(),
                Token::Cos => expr.eval()?.cos(),
                Token::Tan => expr.eval()?.tan(),
                Token::Ln => expr.eval()?.ln(),
                Token::Log => expr.eval()?.log10(),
                _ => return Err(Error::msg("Invalid unary operand.")),
            },
            Self::Value(token) => match token {
                Token::Number(n) => *n,
                _ => return Err(Error::msg("Invalid value")),
            },
        })
    }
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
                    if n_char.is_ascii_alphabetic() {
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
fn evaluate(expr: String) -> Result<f32> {
    let tokens = produce_tokens(expr)?;
    let populated = populate_variables(tokens)?;

    let result = ParseExpr::expr(&mut populated.into_iter().peekable())?.eval()?;
    Ok(result)
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let filepath = &args[1];
        if std::path::Path::new(filepath).exists() {
            let contents = fs::read_to_string(filepath)?;
            let result = evaluate(contents)?;

            println!("[Result] {}", result);
        } else {
            println!("file does not exist. usage: cord [filename]")
        }
    } else {
        println!("usage: cord [filename]")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::evaluate;
    #[test]
    fn test_complex_arithmetic_expressions() {
        assert_eq!(evaluate("(3 * 4) + (2 * 5) - 6".to_string()).unwrap(), 16.0);
        assert_eq!(
            evaluate("((10 + 5) * 2 - 3) / 4".to_string()).unwrap(),
            6.75
        );
        assert_eq!(evaluate("3 + 4 * 2 / ( 1 - 5 )".to_string()).unwrap(), 1.0);
    }

    #[test]
    fn test_nested_expressions() {
        assert_eq!(evaluate("(2 + 3) * (4 - 1)".to_string()).unwrap(), 15.0);
        assert_eq!(evaluate("10 + (5 * (3 - 1))".to_string()).unwrap(), 20.0);
    }

    #[test]
    fn test_trigonometric_expressions() {
        assert_eq!(
            evaluate("(3 * 4) + sin(45)".to_string()).unwrap(),
            12.8509035
        );
        assert_eq!(evaluate("cos(0) + (2 * 3)".to_string()).unwrap(), 7.0);
    }

    #[test]
    fn test_error_cases() {
        assert!(evaluate("invalid expression".to_string()).is_err());
        assert!(evaluate("".to_string()).is_err());
    }
}
