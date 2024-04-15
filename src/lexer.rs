use std::fmt::Error;

#[derive(Debug, PartialEq)]
pub enum Command {
    Add,
    Get,
    Delete,
    Invalid,
}

pub struct Lexer;

impl Lexer {
    pub fn parse(mut input: String) -> Result<(Command, String, String), Error> {
        let command = match &input[0..3] {
            "ADD" => Command::Add,
            "GET" => Command::Get,
            "DEL" => Command::Delete,
            _ => Command::Invalid,
        };

        let input_cp = input.clone();
        let mut split = input_cp.split_whitespace();
        split.next();

        match command {
            Command::Add => {
                let key = split.next().unwrap();
                input.replace_range(0..3, "");
                input.replace_range(0..key.len() + 2, "");
                let value = input;
                return Ok((command, key.to_string(), value.to_string()));
            }
            Command::Get => {
                let key = split.next().unwrap();
                return Ok((command, key.to_string(), "".to_string()));
            }
            Command::Delete => {
                let key = split.next().unwrap();
                return Ok((command, key.to_string(), "".to_string()));
            }
            Command::Invalid => {
                return Err(Error);
            }
        }
    }
}
