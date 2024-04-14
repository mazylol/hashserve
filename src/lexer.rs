#[derive(Debug, PartialEq)]
pub enum Command {
    Add,
    Get,
    Delete,
    Invalid,
}

pub struct Lexer;

impl Lexer {
    pub fn parse(input: String) -> (Command, String, String) {
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
                return (command, key.to_string(), value.to_string());
            }
            Command::Get => {
                let key = split.next().unwrap();
                return (command, key.to_string(), "".to_string());
            }
            Command::Delete => {
                let key = split.next().unwrap();
                return (command, key.to_string(), "".to_string());
            }
            Command::Invalid => {
                return (command, "".to_string(), "".to_string());
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse() {
        assert_eq!(
            super::Lexer::parse("ADD key value".to_string()),
            (super::Command::Add, "key".to_string(), "value".to_string())
        );
        assert_eq!(
            super::Lexer::parse("GET key".to_string()),
            (super::Command::Get, "key".to_string(), "".to_string())
        );
        assert_eq!(
            super::Lexer::parse("DEL key".to_string()),
            (super::Command::Delete, "key".to_string(), "".to_string())
        );
    }
}
