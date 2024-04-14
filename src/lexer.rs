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
    pub fn parse(input: String) -> Result<(Command, String, String), Error> {
        let command = match &input[0..3] {
            "ADD" => Command::Add,
            "GET" => Command::Get,
            "DEL" => Command::Delete,
            _ => Command::Invalid,
        };

        let mut split = input.split_whitespace();
        split.next();

        match command {
            Command::Add => {
                let key = split.next().unwrap();
                let value = split.next().unwrap();
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