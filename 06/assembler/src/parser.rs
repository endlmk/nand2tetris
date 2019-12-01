use std::fs;
use std::io::{self, BufRead};

pub struct Parser<R: io::Read> {
    fs : io::BufReader<R>,
    cur_line : String,
    command_type : Option<CommandType>,
    symbol : String,
    dest : String,
    comp : String,
    jump : String,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum CommandType {
    A_Command,
    C_Command,
    L_Command,
}

impl<R: io::Read> Parser<R> {
    pub fn new(reader : R) -> Self {
        let file_stream = io::BufReader::new(reader);
        Parser {
            fs : file_stream, 
            cur_line : String::from(""),
            command_type : None,
            symbol : String::from(""),
            dest : String::from(""),
            comp : String::from(""),
            jump : String::from(""),
        }
    }
    pub fn hasMoreComments(&mut self) -> bool {
        let mut ln = String::new();
        let mut ln_bytes = self.fs.read_line(&mut ln).unwrap_or_default();
        while ln_bytes != 0 && (ln.starts_with("//") || ln.eq("\r\n")) {
            ln.clear();
            ln_bytes = self.fs.read_line(&mut ln).unwrap_or_default();
        }
        if ln_bytes == 0 {
            false
        }
        else {
            self.cur_line = ln.trim_end_matches("\r\n").to_string();
            true
        }
    }
    pub fn advance(&mut self) {
        let mut ln = self.cur_line.clone();
        if let Some(comment_pos) = ln.find("//") {
            ln.truncate(comment_pos);
        }
        ln.retain(|c| c != ' ');

        if ln.starts_with("(") && ln.ends_with(")") {
            self.command_type = Some(CommandType::L_Command);
            self.symbol = ln.drain(1..ln.len()-1).collect();
            self.dest.clear();
            self.comp.clear();
            self.jump.clear();
        }
        else if ln.starts_with("@") {
            self.command_type = Some(CommandType::A_Command);
            self.symbol = ln.drain(1..).collect();
            self.dest.clear();
            self.comp.clear();
            self.jump.clear();
        }
        else {
            let eq_pos_opt = ln.find("=");
            let sc_pos_opt = ln.find(";");
            if eq_pos_opt.is_some() || sc_pos_opt.is_some() {
                self.command_type = Some(CommandType::C_Command);
                let eq_pos = eq_pos_opt.unwrap_or(0);
                let sc_pos = sc_pos_opt.unwrap_or(ln.len());
                self.dest = ln[..eq_pos].to_string();
                self.comp = ln[eq_pos + if eq_pos_opt.is_some() {1} else {0} ..sc_pos].to_string();
                self.jump = ln[sc_pos + if sc_pos_opt.is_some() {1} else {0} ..].to_string();
                self.symbol.clear();
            }
            else {
                self.command_type = None;
                self.symbol.clear();
                self.dest.clear();
                self.comp.clear();
                self.jump.clear();
            }
        }
    }
    pub fn commandType(&self) -> Option<CommandType> {
        self.command_type.clone()
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    pub fn dest(&self) -> &str {
        &self.dest
    }
    pub fn comp(&self) -> &str {
        &self.comp
    }
    pub fn jump(&self) -> &str {
        &self.jump
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn new()
    {
        let p = fs::File::open("test.txt")
            .and_then(|f : fs::File| {Ok(Parser::new(f))});
        assert_eq!(p.is_ok(), true);
    }

    #[test]
    fn new_error()
    {
        // To be implemented.
        unimplemented!();
    }

    #[test]
    fn hasMoreComments_1() {
        let s = io::Cursor::new("// comment\r\n@aaaa");
        let mut p = Parser::new(s);
        {
            assert_eq!(p.hasMoreComments(), true);
            assert_eq!(p.cur_line, "@aaaa");
        }
        {
            assert_eq!(p.hasMoreComments(), false);
        }
    }

    #[test]
    fn hasMoreComments_2() {
        let s = io::Cursor::new("// comment\r\n\r\n@aaaa");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "@aaaa");
        
        assert_eq!(p.hasMoreComments(), false);
    }

    #[test]
    fn hasMoreComments_3() {
        let s = io::Cursor::new("DDD ssaf \r\n// comment\r\n@aaaa\r\n\r\n");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "DDD ssaf ");
        
        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "@aaaa");
        
        assert_eq!(p.hasMoreComments(), false);
    }

    #[test]
    fn advance_1() {
        let s = io::Cursor::new("@a");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "@a");
        
        p.advance();
        assert_eq!(p.commandType().unwrap(), CommandType::A_Command);
        assert_eq!(p.symbol, "a");
    }

    #[test]
    fn advance_2() {
        let s = io::Cursor::new("    @a // bbb");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "    @a // bbb");
        
        p.advance();
        assert_eq!(p.commandType().unwrap(), CommandType::A_Command);
        assert_eq!(p.symbol, "a");
    }

    #[test]
    fn advance_3() {
        let s = io::Cursor::new("(abc) // bbb");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "(abc) // bbb");
        
        p.advance();
        assert_eq!(p.commandType().unwrap(), CommandType::L_Command);
        assert_eq!(p.symbol, "abc");
    }

    #[test]
    fn advance_4() {
        let s = io::Cursor::new("D = D-A");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "D = D-A");
        
        p.advance();
        assert_eq!(p.commandType().unwrap(), CommandType::C_Command);
        assert_eq!(p.dest, "D");
        assert_eq!(p.comp, "D-A");
        assert_eq!(p.jump, "");
    }

    #[test]
    fn advance_5() {
        let s = io::Cursor::new("    0;JMP");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "    0;JMP");
        
        p.advance();
        assert_eq!(p.commandType().unwrap(), CommandType::C_Command);
        assert_eq!(p.dest, "");
        assert_eq!(p.comp, "0");
        assert_eq!(p.jump, "JMP");
    }

    #[test]
    fn advance_6() {
        let s = io::Cursor::new("AMD =    D +1;  JMP");
        let mut p = Parser::new(s);

        assert_eq!(p.hasMoreComments(), true);
        assert_eq!(p.cur_line, "AMD =    D +1;  JMP");
        
        p.advance();
        assert_eq!(p.commandType().unwrap(), CommandType::C_Command);
        assert_eq!(p.dest, "AMD");
        assert_eq!(p.comp, "D+1");
        assert_eq!(p.jump, "JMP");
    }
}
