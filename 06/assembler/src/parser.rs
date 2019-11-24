use std::fs::File;

pub struct Parser {
    iostream : std::io::Result<File>,
}

pub enum CommandType {
    A_Command,
    C_Command,
    L_Command,
}

impl Parser {
    pub fn new(filename : String) -> Self {
        Parser{iostream : std::io::Result::Err(std::io::Error::from(std::io::ErrorKind::NotFound)), }
    }
    pub fn hasMoreComments() -> bool {
        false
    }
    pub fn advance() {

    }
    pub fn commandType() -> CommandType {
        CommandType::A_Command
    }
    pub fn symbol() -> String {
        "".to_string()
    }
    pub fn dest() -> String {
        "".to_string()
    }
    pub fn comp() -> String {
        "".to_string()
    }
    pub fn jump() -> String {
        "".to_string()
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn test_new()
    {
        let p = Parser::new("test.txt".to_string());
        assert_eq!(p.iostream.is_ok(), true);
    }
}
