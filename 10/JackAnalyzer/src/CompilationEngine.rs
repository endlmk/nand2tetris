use std::io::{self, BufRead, Read, Seek, Write};
use super::JackTokenizer;

pub struct CompilationEngine<R: io::Read + io::Seek, W: io::Write> {
    tokenizer: JackTokenizer::JackTokenizer<R>,
    fs: io::BufWriter<W>,

}

enum NodeType {
    CLASS,
    CLASS_VAR_DEC,
    SUBROUTINE_DEC,
    PARAMETER_LIST,
    SUBROUTINE_BODY,
    VAR_DEC,
}

impl<R: io::Read + io::Seek, W: io::Write> CompilationEngine<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        CompilationEngine {
            tokenizer: JackTokenizer::JackTokenizer::new(reader),
            fs: io::BufWriter::new(writer),
        }
    }

    pub fn compileClass(&mut self) {
        let t = self.tokenizer.next().unwrap();
        assert!(t == JackTokenizer::Token::Keyword(JackTokenizer::KeywordType::CLASS));
        self.fs.write_all(create_open_tag("class").as_bytes());
        self.fs.write_all(indentation(&create_xml_elem("keyword", "class"), 2).as_bytes());
        
        let t = self.tokenizer.next().unwrap();
        assert!(enum_eq(&t, &JackTokenizer::Token::Identifier("".to_string())));
        self.fs.write_all(to_xml_elem(t, 2).as_bytes());

        let t = self.tokenizer.next().unwrap();
        assert!(t == JackTokenizer::Token::Symbol("{".to_string()));
        self.fs.write_all(to_xml_elem(t, 2).as_bytes());
        
        let t = self.tokenizer.next().unwrap();
        assert!(t == JackTokenizer::Token::Symbol("}".to_string()));
        self.fs.write_all(to_xml_elem(t, 2).as_bytes());

        assert!(self.tokenizer.next().is_none());
        self.fs.write_all(create_close_tag("class").as_bytes());
    }
}

fn convert_keyword(keyword_type: JackTokenizer::KeywordType) -> String {
    match keyword_type {
        JackTokenizer::KeywordType::CLASS => "class",
        JackTokenizer::KeywordType::METHOD => "method",
        JackTokenizer::KeywordType::FUNCTION => "function",
        JackTokenizer::KeywordType::CONSTRUCTOR => "constructor",
        JackTokenizer::KeywordType::INT => "int",
        JackTokenizer::KeywordType::BOOLEAN => "boolean",
        JackTokenizer::KeywordType::CHAR => "char",
        JackTokenizer::KeywordType::VOID => "void",
        JackTokenizer::KeywordType::VAR => "var",
        JackTokenizer::KeywordType::STATIC => "static",
        JackTokenizer::KeywordType::FIELD => "field",
        JackTokenizer::KeywordType::LET => "let",
        JackTokenizer::KeywordType::DO => "do",
        JackTokenizer::KeywordType::IF => "if",
        JackTokenizer::KeywordType::ELSE => "else",
        JackTokenizer::KeywordType::WHILE => "while",
        JackTokenizer::KeywordType::RETURN => "return",
        JackTokenizer::KeywordType::TRUE => "true",
        JackTokenizer::KeywordType::FALSE => "false",
        JackTokenizer::KeywordType::NULL => "null",
        JackTokenizer::KeywordType::THIS => "this",
    }.to_string()
}

fn to_xml_elem(token: JackTokenizer::Token, level: usize) -> String {
    let elem = match token {
        JackTokenizer::Token::Keyword(k) => ["keyword".to_string(), convert_keyword(k)],
        JackTokenizer::Token::Symbol(s) => ["symbol".to_string(), escape_symbol(&s)],
        JackTokenizer::Token::Identifier(i) => ["identifier".to_string(), i],
        JackTokenizer::Token::IntConst(i) => ["integerConstant".to_string(), i.to_string()],
        JackTokenizer::Token::StringConst(s) => ["stringConstant".to_string(), s]
    };
    indentation(&create_xml_elem(&elem[0], &elem[1]), level)
}

fn escape_symbol(s: &str) -> String {
    match s {
        "&" => "&amp;",
        "<" => "&lt;",
        ">" => "&gt;",
        _ => s
    }.to_string()
}

fn create_open_tag(name: &str) -> String {
    format!("<{}>\r\n", name) 
}

fn create_close_tag(name: &str) -> String {
    format!("</{}>\r\n", name) 
}

fn create_xml_elem(tag: &str, value: &str) -> String {
    format!("<{0}> {1} </{0}>\r\n", tag, escape_symbol(value)) 
}

fn indentation(s: &str, level: usize) -> String {
    format!("{:indent$}{}", "", s, indent=level)
}

fn enum_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn SimplestClass() {
        let s = io::Cursor::new("\
        class Main {}
        ");
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w);

        c.compileClass();
        let mut r = r#"
<class>
  <keyword> class </keyword>
  <identifier> Main </identifier>
  <symbol> { </symbol>
  <symbol> } </symbol>
</class>
"#.to_string();
        r.remove(0);
        r = r.replace("\n", "\r\n");
        assert_eq!(String::from_utf8(c.fs.buffer().to_vec()).unwrap(), r);
    }
}