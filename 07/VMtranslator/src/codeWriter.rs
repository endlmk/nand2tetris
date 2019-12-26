use super::parser;
use std::io::{self, BufWriter, Write};

pub struct CodeWriter<W: io::Write> {
    os: io::BufWriter<W>,
    fileName_wo_ext: String,
    index_jmp: i32,
}

impl<W: io::Write> CodeWriter<W> {
    pub fn new(writer: W) -> Self {
        CodeWriter {
            os: io::BufWriter::new(writer),
            fileName_wo_ext: String::from(""),
            index_jmp: 0
        }
    }
    pub fn setFileName(&mut self, file_name: &str) {
        let p = std::path::Path::new(file_name);
        self.fileName_wo_ext = p.file_stem().unwrap().to_str().unwrap().to_string();
    }

    pub fn writePushPop(&mut self, command: parser::CommandType, arg1: &str, arg2: i32) {
        let pop_base_addr_template = |s :&str, pos: i32| format!("\
            @{0}\r\n\
            D=M\r\n\
            @{1}\r\n\
            D=D+A\r\n\
            @13\r\n\
            M=D\r\n\
            @SP\r\n\
            M=M-1\r\n\
            @SP\r\n\
            A=M\r\n\
            D=M\r\n\
            @13\r\n\
            A=M\r\n\
            M=D\r\n\
            ", s, pos);
        
        let pop_abs_addr_template = |addr: i32| format!("\
            @SP\r\n\
            M=M-1\r\n\
            @SP\r\n\
            A=M\r\n\
            D=M\r\n\
            @{}\r\n\
            M=D\r\n\
            ", addr);

        let push_base_addr_template = |s :&str, pos: i32| format!("\
            @{0}\r\n\
            D=M\r\n\
            @{1}\r\n\
            D=D+A\r\n\
            @13\r\n\
            M=D\r\n\
            @13\r\n\
            A=M\r\n\
            D=M\r\n\
            @SP\r\n\
            A=M\r\n\
            M=D\r\n\
            @SP\r\n\
            M=M+1\r\n\
            ", s, pos);

        let push_abs_addr_template = |addr: i32| format!("\
            @{}\r\n\
            D=M\r\n\
            @SP\r\n\
            A=M\r\n\
            M=D\r\n\
            @SP\r\n\
            M=M+1\r\n\
            ", addr);
    
        let c = match command {
            parser::CommandType::C_PUSH => {
                match arg1 {
                    "constant" => format!("\
                        @{}\r\n\
                        D=A\r\n\
                        @SP\r\n\
                        A=M\r\n\
                        M=D\r\n\
                        @SP\r\n\
                        M=M+1\r\n\
                        ", 
                        arg2),
                    "local" => push_base_addr_template("LCL", arg2),
                    "argument" => push_base_addr_template("ARG", arg2),
                    "this" => push_base_addr_template("THIS", arg2),
                    "that" => push_base_addr_template("THAT", arg2),
                    "temp" => push_abs_addr_template(arg2 + 5),
                    _ => String::from(""),
                }
            },
            parser::CommandType::C_POP => {
                match arg1 {
                    "local" => pop_base_addr_template("LCL", arg2),
                    "argument" => pop_base_addr_template("ARG", arg2),
                    "this" => pop_base_addr_template("THIS", arg2),
                    "that" => pop_base_addr_template("THAT", arg2),
                    "temp" => pop_abs_addr_template(arg2 + 5),
                    _ => String::from(""),
                }
            },
            _ => String::from("")
        };
        if !c.is_empty() {
            self.os.write(c.as_bytes());
        }
    }
    
    pub fn writeArithmetic(&mut self, arg1: &str) {
        let unary_template = |op: &str| format!("\
            @SP\r\n\
            M=M-1\r\n\
            @SP\r\n\
            A=M\r\n\
            M={}M\r\n\
            @SP\r\n\
            M=M+1\r\n\
            ", op);

        let binary_template = |op: &str| format!("\
                @SP\r\n\
                M=M-1\r\n\
                @SP\r\n\
                A=M\r\n\
                D=M\r\n\
                @SP\r\n\
                M=M-1\r\n\
                @SP\r\n\
                A=M\r\n\
                D=M{}D\r\n\
                @SP\r\n\
                A=M\r\n\
                M=D\r\n\
                @SP\r\n\
                M=M+1\r\n\
                ", op);

        let cmp_template = |op: &str, i: i32| format!("\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @IFZERO.{1}\r\n\
        D;{0}\r\n\
        @SP\r\n\
        A=M\r\n\
        M=0\r\n\
        @END.{1}\r\n\
        0;JMP\r\n\
        (IFZERO.{1})\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-1\r\n\
        (END.{1})\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ", op, i);
        
        let asm = match arg1 {
            "add" => binary_template("+"),
            "sub" => binary_template("-"),
            "neg" => unary_template("-"),
            "eq" => { let s = cmp_template("JEQ", self.index_jmp); self.index_jmp += 1; s },
            "gt" => { let s = cmp_template("JGT", self.index_jmp); self.index_jmp += 1; s },
            "lt" => { let s = cmp_template("JLT", self.index_jmp); self.index_jmp += 1; s },
            "and" => binary_template("&"),
            "or" => binary_template("|"),
            "not" => unary_template("!"),
            _ => String::from(""),
        };

        if !asm.is_empty() {
            self.os.write(asm.as_bytes());
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
    fn push_const() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_PUSH, "constant", 3);

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

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), push_const_3);
    }

    #[test]
    fn add() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("add");

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
        D=M+D\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), add);
    }

    #[test]
    fn sub() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("sub");
        
        let sub = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), sub);
    }

    #[test]
    fn neg() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("neg");
        
        let neg = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-M\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), neg);
    }

    #[test]
    fn eq() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("eq");
        
        let eq = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @IFZERO.0\r\n\
        D;JEQ\r\n\
        @SP\r\n\
        A=M\r\n\
        M=0\r\n\
        @END.0\r\n\
        0;JMP\r\n\
        (IFZERO.0)\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-1\r\n\
        (END.0)\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), eq);
    }

    #[test]
    fn gt() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("gt");
        
        let gt = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @IFZERO.0\r\n\
        D;JGT\r\n\
        @SP\r\n\
        A=M\r\n\
        M=0\r\n\
        @END.0\r\n\
        0;JMP\r\n\
        (IFZERO.0)\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-1\r\n\
        (END.0)\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), gt);
    }

    #[test]
    fn lt() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("lt");
        
        let lt = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @IFZERO.0\r\n\
        D;JLT\r\n\
        @SP\r\n\
        A=M\r\n\
        M=0\r\n\
        @END.0\r\n\
        0;JMP\r\n\
        (IFZERO.0)\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-1\r\n\
        (END.0)\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), lt);
    }

    #[test]
    fn and() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("and");

        let and = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M&D\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), and);
    }

    #[test]
    fn or() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("or");

        let or = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M|D\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), or);
    }

    #[test]
    fn not() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("not");
        
        let not = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        M=!M\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), not);
    }


    #[test]
    fn rep_cmp() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writeArithmetic("lt");
        cw.writeArithmetic("lt");
        
        let lt1 = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @IFZERO.0\r\n\
        D;JLT\r\n\
        @SP\r\n\
        A=M\r\n\
        M=0\r\n\
        @END.0\r\n\
        0;JMP\r\n\
        (IFZERO.0)\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-1\r\n\
        (END.0)\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        let lt2 = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M-D\r\n\
        @IFZERO.1\r\n\
        D;JLT\r\n\
        @SP\r\n\
        A=M\r\n\
        M=0\r\n\
        @END.1\r\n\
        0;JMP\r\n\
        (IFZERO.1)\r\n\
        @SP\r\n\
        A=M\r\n\
        M=-1\r\n\
        (END.1)\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), [lt1, lt2].concat());
    }

    #[test]
    fn pop_local() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_POP, "local", 0);

        let pop_local_0 = "\
        @LCL\r\n\
        D=M\r\n\
        @0\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @13\r\n\
        A=M\r\n\
        M=D\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), pop_local_0);
    }

    #[test]
    fn pop_arg() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_POP, "argument", 2);

        let c = "\
        @ARG\r\n\
        D=M\r\n\
        @2\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @13\r\n\
        A=M\r\n\
        M=D\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn pop_this() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_POP, "this", 6);

        let c = "\
        @THIS\r\n\
        D=M\r\n\
        @6\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @13\r\n\
        A=M\r\n\
        M=D\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn pop_that() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_POP, "that", 5);

        let c = "\
        @THAT\r\n\
        D=M\r\n\
        @5\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @13\r\n\
        A=M\r\n\
        M=D\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn pop_temp() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_POP, "temp", 6);

        let c = "\
        @SP\r\n\
        M=M-1\r\n\
        @SP\r\n\
        A=M\r\n\
        D=M\r\n\
        @11\r\n\
        M=D\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn push_that() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_PUSH, "that", 5);

        let c = "\
        @THAT\r\n\
        D=M\r\n\
        @5\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @13\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn push_arg() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_PUSH, "argument", 1);

        let c = "\
        @ARG\r\n\
        D=M\r\n\
        @1\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @13\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn push_this() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_PUSH, "this", 6);

        let c = "\
        @THIS\r\n\
        D=M\r\n\
        @6\r\n\
        D=D+A\r\n\
        @13\r\n\
        M=D\r\n\
        @13\r\n\
        A=M\r\n\
        D=M\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }

    #[test]
    fn push_temp() {
        let s = io::Cursor::new(Vec::new());
        let mut cw = CodeWriter::new(s);
        cw.writePushPop(parser::CommandType::C_PUSH, "temp", 6);

        let c = "\
        @11\r\n\
        D=M\r\n\
        @SP\r\n\
        A=M\r\n\
        M=D\r\n\
        @SP\r\n\
        M=M+1\r\n\
        ";

        assert_eq!(String::from_utf8(cw.os.buffer().to_vec()).unwrap(), c);
    }
}

