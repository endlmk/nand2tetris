use super::parser;
use std::io::{self, BufWriter, Write};

pub struct CodeWriter<W: io::Write> {
    os: io::BufWriter<W>,
    fileName_wo_ext: String,
}

impl<W: io::Write> CodeWriter<W> {
    pub fn new(writer: W) -> Self {
        CodeWriter {
            os: io::BufWriter::new(writer),
            fileName_wo_ext: String::from(""),
        }
    }
    pub fn setFileName(&mut self, file_name: &str) {
        let p = std::path::Path::new(file_name);
        self.fileName_wo_ext = p.file_stem().unwrap().to_str().unwrap().to_string();
    }

    pub fn writePushPop(&mut self, command: parser::CommandType, arg1: &str, arg2: i32) {
        match command {
            parser::CommandType::C_PUSH => {
                match arg1 {
                    "constant" => {
                        let push_const = format!("\
                        @{}\r\n\
                        D=A\r\n\
                        @SP\r\n\
                        A=M\r\n\
                        M=D\r\n\
                        @SP\r\n\
                        M=M+1\r\n\
                        ", 
                        arg2);
                        self.os.write(push_const.as_bytes());
                    },
                    _ => {}
                }
            },
            parser::CommandType::C_POP => {},
            _ => {}
        }
    }
    
    pub fn writeArithmetic(&mut self, arg1: &str) {
        match arg1 {
            "add" => {  
                let add = "\
                @SP\r\n\
                M=M-1\r\n\
                @SP\r\n\
                A=M\r\n\
                D=M\r\n\
                @SP\r\n\
                M=M-1\r\n\
                @SP\r\n\
                A=M\r\n\
                D=D+M\r\n\
                @SP\r\n\
                A=M\r\n\
                M=D\r\n\
                @SP\r\n\
                M=M+1\r\n\
                ";
                self.os.write(add.as_bytes());
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn new() {
        let s = io::Cursor::new(Vec::new());
        let cw = CodeWriter::new(s);
    }
    #[test]
    fn setFileName() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.setFileName("test.vm");
        assert_eq!(cw.fileName_wo_ext, "test")
    }

    #[test]
    fn const_plus() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_PUSH, "constant", 3);
        cw.writePushPop(parser::CommandType::C_PUSH, "constant", 4);
        cw.writeArithmetic("add");

        // push constant 3
        let push_const_3 = "\
        @3\r\n\
        D=A\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        // push constant 4
        let push_const_4 = "\
        @4\r\n\
        D=A\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        //add
        let add = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=D+M\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), [push_const_3, push_const_4, add].concat());
    }
}