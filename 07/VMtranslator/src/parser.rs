use std::fs;
use std::io::{self, BufRead, Read};

pub struct Parser<R: io::Read> {
    fs : io::BufReader<R>,
    cur_line : String,
    command_type : Option<CommandType>,
    arg1 : String,
    arg2 : i32,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum CommandType {
    C_ARITHMETIC,
    C_PUSH,
    C_POP,
    C_LABEL,
    C_GOTO,
    C_IF,
    C_FUNCTION,
    C_RETURN,
    C_CALL,
}

impl<R: io::Read> Parser<R> {
    pub fn new(reader : R) -> Self {
        Parser {
            fs : io::BufReader::new(reader),
            cur_line : String::from(""),
            command_type : None,
            arg1 : String::from(""),
            arg2 : 0
        }
    }
    pub fn hasMoreCommands(&mut self) -> bool {
        for line in self.fs.by_ref().lines() {
            let l = line.unwrap_or_default();
            if !(l.starts_with("//") || l.is_empty()) {
                self.cur_line = l;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn new() {
        let s = io::Cursor::new("// comment\r\n@aaaa");
        let p = Parser::new(s);
        assert_eq!(p.cur_line, "");
    }

    #[test]
    fn hasMoreCommands1() {
        let s = io::Cursor::new("// comment\r\n\r\npush constant 7");
        let mut p = Parser::new(s);
        assert_eq!(p.cur_line, "");
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "push constant 7");
        }
        {
            assert_eq!(p.hasMoreCommands(), false);
        }
    }

    #[test]
    fn hasMoreCommands2() {
        let s = io::Cursor::new("// comment\r\n\r\npush constant 17\r\npush constant 17\r\neq");
        let mut p = Parser::new(s);
        assert_eq!(p.cur_line, "");
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "push constant 17");
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "push constant 17");
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "eq");
        }
        {
            assert_eq!(p.hasMoreCommands(), false);
        }
    }
}