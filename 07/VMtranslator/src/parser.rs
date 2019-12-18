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
    pub fn advance(&mut self) {
        let strs : Vec<&str>= self.cur_line.split_whitespace().collect();
 
        //init
        self.command_type = None;
        self.arg1.clear();
        self.arg2 = 0;

        match strs.get(0) {
            Some(command) => match command {
                &"add" | &"sub" | &"neg" | &"eq" | &"gt" | &"lt" | &"and" | &"or" | &"not" => {
                    self.command_type = Some(CommandType::C_ARITHMETIC);
                    self.arg1 = command.to_string();
                },
                &"push" => {
                    self.command_type = Some(CommandType::C_PUSH);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                    if strs.len() > 2 { self.arg2 = strs[2].parse().unwrap_or_default(); }
                },
                &"pop" => {
                    self.command_type = Some(CommandType::C_POP);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                    if strs.len() > 2 { self.arg2 = strs[2].parse().unwrap_or_default(); }
                },
                &"label" => {
                    self.command_type = Some(CommandType::C_LABEL);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                },
                &"goto" => {
                    self.command_type = Some(CommandType::C_GOTO);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                },
                &"if-goto" => {
                    self.command_type = Some(CommandType::C_IF);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                },
                &"function" => {
                    self.command_type = Some(CommandType::C_FUNCTION);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                    if strs.len() > 2 { self.arg2 = strs[2].parse().unwrap_or_default(); }
                },
                &"call" => {
                    self.command_type = Some(CommandType::C_CALL);
                    if strs.len() > 1 { self.arg1 = strs[1].to_string(); }
                    if strs.len() > 2 { self.arg2 = strs[2].parse().unwrap_or_default(); }
                },
                &"return" => {
                    self.command_type = Some(CommandType::C_RETURN);
                },
                _ => return,
            },
            None => return,
        }
    }

    pub fn commandType(&self) -> Option<CommandType> {
        self.command_type.clone()
    }
    pub fn arg1(&self) -> &str {
        &self.arg1
    }
    pub fn arg2(&self) -> i32 {
        self.arg2
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

    #[test]
    fn advance1() {
        let s = io::Cursor::new("// comment\r\n\r\npush local 23 // aaaa\r\npush constant 17\r\neq");
        let mut p = Parser::new(s);
        assert_eq!(p.cur_line, "");
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "push local 23 // aaaa");
            
            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_PUSH));
            assert_eq!(p.arg1(), "local");
            assert_eq!(p.arg2(), 23);
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "push constant 17");
        
            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_PUSH));
            assert_eq!(p.arg1(), "constant");
            assert_eq!(p.arg2(), 17);    
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "eq");

            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_ARITHMETIC));
            assert_eq!(p.arg1(), "eq");
        }
        {
            assert_eq!(p.hasMoreCommands(), false);
        }
    }
    #[test]
    fn advance2() {
        let s = io::Cursor::new("\r\nfunction   mult 2 // 2 locals\r\npop local 0\r\n   add\r\nlabel LOOP\r\n    if-goto END\r\ngoto LOOP\r\nreturn");
        let mut p = Parser::new(s);
        assert_eq!(p.cur_line, "");
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "function   mult 2 // 2 locals");
            
            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_FUNCTION));
            assert_eq!(p.arg1(), "mult");
            assert_eq!(p.arg2(), 2);
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "pop local 0");
        
            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_POP));
            assert_eq!(p.arg1(), "local");
            assert_eq!(p.arg2(), 0);    
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "   add");

            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_ARITHMETIC));
            assert_eq!(p.arg1(), "add");
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "label LOOP");

            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_LABEL));
            assert_eq!(p.arg1(), "LOOP");
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "    if-goto END");

            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_IF));
            assert_eq!(p.arg1(), "END");
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "goto LOOP");

            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_GOTO));
            assert_eq!(p.arg1(), "LOOP");
        }
        {
            assert_eq!(p.hasMoreCommands(), true);
            assert_eq!(p.cur_line, "return");

            p.advance();
            assert_eq!(p.commandType(), Some(CommandType::C_RETURN));
        }
        {
            assert_eq!(p.hasMoreCommands(), false);
        }
    }
}