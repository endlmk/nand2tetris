use std::io::{self, Write};

pub struct VMWriter<W: Write> {
    fs: io::BufWriter<W>
} 

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Segment {
    CONST,
    ARG,
    LOCAL,
    STATIC,
    THIS,
    THAT,
    POINTER,
    TEMP,
} 

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            Segment::CONST => "constant",
            Segment::ARG => "argument",
            Segment::LOCAL => "local",
            Segment::STATIC => "static",
            Segment::THIS => "this",
            Segment::THAT => "that",
            Segment::POINTER => "pointer",
            Segment::TEMP => "temp",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Command {
    ADD,
    SUB,
    NEG,
    EQ,
    GT,
    LT,
    AND,
    OR,
    NOT,
} 

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            Command::ADD => "add",
            Command::SUB => "sub",
            Command::NEG => "neg",
            Command::EQ => "eq",
            Command::GT => "gt",
            Command::LT => "lt",
            Command::AND => "and",
            Command::OR => "or",
            Command::NOT => "not",
        };
        write!(f, "{}", s)
    }
}

impl<W: Write> VMWriter<W> {
    pub fn new(writer: W) -> Self {
        VMWriter {
            fs: io::BufWriter::new(writer),
        }
    }

    pub fn writePush(&mut self, seg: Segment, index: i32) {
        let s = format!("push {0} {1}\r\n", seg.to_string(), index);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writePop(&mut self, seg: Segment, index: i32) {
        let s = format!("pop {0} {1}\r\n", seg.to_string(), index);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeArithmetic(&mut self, command: Command) {
        let s = format!("{}\r\n", command.to_string());
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeLabel(&mut self, label: &str) {
        let s = format!("label {}\r\n", label);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeGoto(&mut self, label: &str) {
        let s = format!("goto {}\r\n", label);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeIf(&mut self, label: &str) {
        let s = format!("if-goto {}\r\n", label);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeCall(&mut self, name: &str, nArgs :i32) {
        let s = format!("call {0} {1}\r\n", name, nArgs);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeFunction(&mut self, name: &str, nLocals :i32) {
        let s = format!("function {0} {1}\r\n", name, nLocals);
        self.fs.write_all(s.as_bytes());
    }

    pub fn writeReturn(&mut self) {
        self.fs.write_all("return\r\n".as_bytes());
    }

    pub fn dump_string(&mut self) -> String {
        String::from_utf8(self.fs.buffer().to_vec()).unwrap()
    }
}