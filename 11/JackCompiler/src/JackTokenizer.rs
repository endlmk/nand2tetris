use std::io::{self, BufRead, Read, Seek, Write};

pub struct JackTokenizer<R: io::Read + io::Seek> {
    fs: io::BufReader<R>,
    token_type: Option<TokenType>,
    cur_char: Option<u8>,
    keyword_type: Option<KeywordType>,
    identifier: Option<String>,
    symbol: Option<String>,
    int_val: Option<i32>,
    string_val: Option<String>,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum TokenType {
    KEYWORD,
    SYMBOL,
    IDENTIFIER,
    INT_CONST,
    STRING_CONST,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum KeywordType {
    CLASS,
    METHOD,
    FUNCTION,
    CONSTRUCTOR,
    INT,
    BOOLEAN,
    CHAR,
    VOID,
    VAR,
    STATIC,
    FIELD,
    LET,
    DO,
    IF,
    ELSE,
    WHILE,
    RETURN,
    TRUE,
    FALSE,
    NULL,
    THIS,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Token {
    Keyword(KeywordType),
    Symbol(String),
    Identifier(String),
    IntConst(i32),
    StringConst(String),
}

impl<R: io::Read + io::Seek> JackTokenizer<R> {
    pub fn new(reader: R) -> Self {
        JackTokenizer {
            fs: io::BufReader::new(reader),
            token_type: None,
            cur_char: None,
            keyword_type: None,
            identifier: None,
            symbol: None,
            int_val: None,
            string_val: None,
        }
    }

    pub fn hasMoreTokens(&mut self) -> bool {
        let mut is_in_inline_comment = false;
        let mut is_in_block_comment = false;

        let get_next_char = |fs: &mut io::BufReader<R>| {
            let r = fs.bytes().next();
            if r.is_none() {
                None
            }
            else {
                let ch = r.unwrap().unwrap();
                Some(ch.clone())
            }};

        loop {
            // インラインコメントなら次の行まですすめる                
            if is_in_inline_comment {
                let mut s = String::new();
                self.fs.read_line(&mut s);
                is_in_inline_comment = false;
                continue;
            }
            
            let r = get_next_char(&mut self.fs);
            if r.is_none() {
                self.cur_char = None;
                break;
            }

            let c = r.unwrap();
            
            // ブロックコメントなら"*/"まですすめる 
            if is_in_block_comment {
                if c == b'*' {
                    let r1 = get_next_char(&mut self.fs);
                    if r1.is_none() {
                        self.cur_char = None;
                        break;
                    }
                    let c1 = r1.unwrap();
                    if c1 == b'/' {
                        is_in_block_comment = false;
                    }
                    continue;
                }
                continue;
            }
            
            // 空白 or 改行ならすすめる
            if c.is_ascii_whitespace() 
            || c == b'\r'
            || c == b'\n' {
                continue;
            }

            // コメント開始かどうか判定する
            if c == b'/' {
                match get_next_char(&mut self.fs) {
                    None => {
                        // EOFを読んだときはカーソルが進まない
                        self.cur_char = Some(b'/');
                        break;
                    }
                    Some(c1) => {
                        if c1 == b'/' {
                            is_in_inline_comment = true;
                            continue;
                        } else if c1 == b'*' {
                            is_in_block_comment = true;
                            continue;
                        } else {
                            // コメントでないので一つ戻してカレントにする
                            self.fs.seek(std::io::SeekFrom::Current(-1));
                            self.cur_char = Some(b'/');
                            break;
                        }
                    }
                }                   
            }
            
            // いずれにも該当しないのでカレントとする
            self.cur_char = Some(c);
            break;
        }

        self.cur_char.is_some()
    }

    pub fn advance(&mut self) {
        let cur_char = self.cur_char.unwrap();
        
        let read_word = |cur_char_ref: &mut Option<u8>, until_cond: fn(u8) -> bool, fs: &mut io::BufReader<R>| {
            let mut buf = vec![];
            buf.push(cur_char_ref.unwrap());
            // read until delimter
            loop {
                let r = fs.by_ref().bytes().next();
                if r.is_none() {
                    *cur_char_ref = Some(buf.last().unwrap().clone());
                    break;
                }
                let c = r.unwrap().unwrap();
                if !until_cond(c) {
                    fs.seek(std::io::SeekFrom::Current(-1));
                    *cur_char_ref = Some(buf.last().unwrap().clone());
                    break;
                }
                buf.push(c)
            }
            String::from_utf8(buf).unwrap()
        };

        let read_word_to_end_cond = |cur_char_ref: &mut Option<u8>, end_cond: fn(u8) -> bool, fs: &mut io::BufReader<R>| {
            let mut buf = vec![];
            buf.push(cur_char_ref.unwrap());
            // read until delimter
            loop {
                let r = fs.by_ref().bytes().next();
                if r.is_none() {
                    *cur_char_ref = Some(buf.last().unwrap().clone());
                    break;
                }
                let c = r.unwrap().unwrap();
                buf.push(c);
                if end_cond(c) {
                    *cur_char_ref = Some(buf.last().unwrap().clone());
                    break;
                }
            }
            String::from_utf8(buf).unwrap()
        };

        // try symbol
        match cur_char {
            b';' | b'=' | b'.' | b'(' | b')' | b'[' | b']' | b'{' | b'}' |
            b',' | b'+' | b'-' | b'*' | b'/' | b'&' | b'|' | b'<' | b'>' | b'~' => {
                self.token_type = Some(TokenType::SYMBOL);
                self.symbol = Some(String::from_utf8(vec![cur_char]).unwrap());
                return;
            } 
            _ => {}
        }
        // try integer
        if cur_char.is_ascii_digit() {
            let word = read_word(&mut self.cur_char, |c: u8| { c.is_ascii_digit() }, &mut self.fs);
            let i:i32 = word.parse().unwrap();
            self.token_type = Some(TokenType::INT_CONST);
            self.int_val = Some(i);
            return;
        }

        // try string
        if cur_char == b'"' {
            let mut word = read_word_to_end_cond(&mut self.cur_char, |c: u8| { c == b'"' }, &mut self.fs);
            // remove double quaotes
            word.remove(0);
            word.pop();
            self.token_type = Some(TokenType::STRING_CONST);
            self.string_val = Some(word);
            return;
        }

        // try keyword or identifier
        if cur_char.is_ascii_alphabetic() 
        || cur_char == b'_' {
            let word = read_word(&mut self.cur_char, |c: u8| { c.is_ascii_alphabetic() || c.is_ascii_digit() || (c == b'_') }, &mut self.fs);
            self.token_type = Some(TokenType::KEYWORD);
            match &*word {
                "class" => self.keyword_type = Some(KeywordType::CLASS),
                "var" => self.keyword_type = Some(KeywordType::VAR),
                "int" => self.keyword_type = Some(KeywordType::INT),
                "let" => self.keyword_type = Some(KeywordType::LET),
                "constructor" => self.keyword_type = Some(KeywordType::CONSTRUCTOR),
                "function" => self.keyword_type = Some(KeywordType::FUNCTION),
                "method" => self.keyword_type = Some(KeywordType::METHOD),
                "field" => self.keyword_type = Some(KeywordType::FIELD),
                "static" => self.keyword_type = Some(KeywordType::STATIC),
                "char" => self.keyword_type = Some(KeywordType::CHAR),
                "boolean" => self.keyword_type = Some(KeywordType::BOOLEAN),
                "void" => self.keyword_type = Some(KeywordType::VOID),
                "true" => self.keyword_type = Some(KeywordType::TRUE),
                "false" => self.keyword_type = Some(KeywordType::FALSE),
                "null" => self.keyword_type = Some(KeywordType::NULL),
                "this" => self.keyword_type = Some(KeywordType::THIS),
                "do" => self.keyword_type = Some(KeywordType::DO),
                "if" => self.keyword_type = Some(KeywordType::IF),
                "else" => self.keyword_type = Some(KeywordType::ELSE),
                "while" => self.keyword_type = Some(KeywordType::WHILE),
                "return" => self.keyword_type = Some(KeywordType::RETURN),
                _ => {
                    self.token_type = Some(TokenType::IDENTIFIER);
                    self.identifier = Some(word);
                },
            }
        }
    }


    pub fn tokenType(&self) -> Option<TokenType> {
        self.token_type.clone()
    }

    pub fn keywordType(&self) -> Option<KeywordType> {
        self.keyword_type.clone()
    }

    pub fn identifier(&self) -> Option<String> {
        self.identifier.clone()
    }

    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }

    pub fn intVal(&self) -> Option<i32> {
        self.int_val.clone()
    }

    pub fn stringVal(&self) -> Option<String> {
        self.string_val.clone()
    }

    pub fn to_xml(&mut self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}\r\n", create_open_tag("tokens")));
        for token in self {
            let elem = match token {
                Token::Keyword(k) => ["keyword".to_string(), convert_keyword(k)],
                Token::Symbol(s) => ["symbol".to_string(), escape_symbol(&s)],
                Token::Identifier(i) => ["identifier".to_string(), i],
                Token::IntConst(i) => ["integerConstant".to_string(), i.to_string()],
                Token::StringConst(s) => ["stringConstant".to_string(), s]
            };
            let line = format!("{0} {1} {2}\r\n", create_open_tag(&elem[0]), elem[1], create_close_tag(&elem[0]));
            s.push_str(&line);
        };
        s.push_str(&format!("{}\r\n", create_close_tag("tokens")));
        s
    }
}
fn create_open_tag(name: &str) -> String {
    format!("<{}>", name) 
}

fn create_close_tag(name: &str) -> String {
    format!("</{}>", name) 
}

fn convert_keyword(keyword_type: KeywordType) -> String {
    match keyword_type {
        KeywordType::CLASS => "class",
        KeywordType::METHOD => "method",
        KeywordType::FUNCTION => "function",
        KeywordType::CONSTRUCTOR => "constructor",
        KeywordType::INT => "int",
        KeywordType::BOOLEAN => "boolean",
        KeywordType::CHAR => "char",
        KeywordType::VOID => "void",
        KeywordType::VAR => "var",
        KeywordType::STATIC => "static",
        KeywordType::FIELD => "field",
        KeywordType::LET => "let",
        KeywordType::DO => "do",
        KeywordType::IF => "if",
        KeywordType::ELSE => "else",
        KeywordType::WHILE => "while",
        KeywordType::RETURN => "return",
        KeywordType::TRUE => "true",
        KeywordType::FALSE => "false",
        KeywordType::NULL => "null",
        KeywordType::THIS => "this",
    }.to_string()
}

fn escape_symbol(s: &str) -> String {
    match s {
        "&" => "&amp;",
        "<" => "&lt;",
        ">" => "&gt;",
        _ => s
    }.to_string()
}

impl<R: io::Read + io::Seek> Iterator for JackTokenizer<R> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        if self.hasMoreTokens() {
            self.advance();
            match self.tokenType() {
                Some(TokenType::KEYWORD) => Some(Token::Keyword(self.keywordType().unwrap())),
                Some(TokenType::SYMBOL) => Some(Token::Symbol(self.symbol().unwrap())),
                Some(TokenType::IDENTIFIER) => Some(Token::Identifier(self.identifier().unwrap())),
                Some(TokenType::INT_CONST) => Some(Token::IntConst(self.intVal().unwrap())),
                Some(TokenType::STRING_CONST) => Some(Token::StringConst(self.stringVal().unwrap())),
                _ => None
            }
        }
        else
        {
            None
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn new () {
        let s = io::Cursor::new("// comment\r\n@aaaa");
        let t = JackTokenizer::new(s);
        assert_eq!(t.token_type, None);
    }

    #[test]
    fn hasMoreTokens_empty() {
        let s = io::Cursor::new("");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn hasMoreTokens_simple() {
        let s = io::Cursor::new("a b c");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'a'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'b'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'c'));
        assert_eq!(t.hasMoreTokens(), false);
    }


    #[test]
    fn hasMoreTokens_simple2() {
        let s = io::Cursor::new("\
        ab = 1/2;
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'a'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'b'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'='));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'1'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'/'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'2'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b';'));
        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn hasMoreTokens_comment1() {
        let s = io::Cursor::new("\
        // comment\r\n\
        @aaaa\
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'@'));
    }

    #[test]
    fn hasMoreTokens_comment2() {
        let s = io::Cursor::new("\
        /* comment\r\n\
        comment aaa */\r\n\
        /");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'/'));
    }


    #[test]
    fn hasMoreTokens_comment3() {
        let s = io::Cursor::new("\
        /** API comment/**\r\n\
        comment aaa */a\r\n\
        /");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'a'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'/'));
        assert_eq!(t.hasMoreTokens(), false);
    }
    #[test]
    fn hasMoreTokens_comment4() {
        let s = io::Cursor::new("\
        / * /* API comment// **\r\n\
        comment aaa */
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'/'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'*'));
        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn hasMoreTokens_comment5() {
        let s = io::Cursor::new("\
        ab // comment\r\n\
        \r\n\
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'a'));
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'b'));
        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_keyword1() {
        let s = io::Cursor::new("\
        class \
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'c'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::CLASS));
        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_identifier1() {
        let s = io::Cursor::new("\
        test\r\n\
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b't'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("test")));
        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_vardec() {
        let s = io::Cursor::new("\
        // var declaration\r\n\
           var int  sample1;\r\n\
        ");
        let mut t = JackTokenizer::new(s);

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'v'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::VAR));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'i'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::INT));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b's'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("sample1")));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b';'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(";")));

        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_letstatement1() {
        let s = io::Cursor::new("\
        let c = 33;\r\n\
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'l'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::LET));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'c'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("c")));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'='));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("=")));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'3'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::INT_CONST));
        assert_eq!(t.intVal(), Some(33));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b';'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(";")));

        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_letstatement2() {
        let s = io::Cursor::new("\
        let string_test1 = \"あああ 　　aaa ;:/=\";\r\n\
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'l'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::LET));
        assert_eq!(t.cur_char, Some(b't'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b's'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("string_test1")));
        assert_eq!(t.cur_char, Some(b'1'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'='));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("=")));
        assert_eq!(t.cur_char, Some(b'='));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'"'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::STRING_CONST));
        assert_eq!(t.stringVal(), Some(String::from("あああ 　　aaa ;:/=")));
        assert_eq!(t.cur_char, Some(b'"'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b';'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(";")));
        assert_eq!(t.cur_char, Some(b';'));

        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_letstatement3() {
        let s = io::Cursor::new("\
        let s = \"A\"; let c = s.charAt(0);\r\n\
        ");
        let mut t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'l'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::LET));
        assert_eq!(t.cur_char, Some(b't'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b's'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("s")));
        assert_eq!(t.cur_char, Some(b's'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'='));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("=")));
        assert_eq!(t.cur_char, Some(b'='));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'"'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::STRING_CONST));
        assert_eq!(t.stringVal(), Some(String::from("A")));
        assert_eq!(t.cur_char, Some(b'"'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b';'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(";")));
        assert_eq!(t.cur_char, Some(b';'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'l'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::LET));
        assert_eq!(t.cur_char, Some(b't'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'c'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("c")));
        assert_eq!(t.cur_char, Some(b'c'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'='));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("=")));
        assert_eq!(t.cur_char, Some(b'='));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b's'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("s")));
        assert_eq!(t.cur_char, Some(b's'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'.'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(".")));
        assert_eq!(t.cur_char, Some(b'.'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'c'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("charAt")));
        assert_eq!(t.cur_char, Some(b't'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b'('));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("(")));
        assert_eq!(t.cur_char, Some(b'('));
        assert_eq!(t.hasMoreTokens(), true);

        assert_eq!(t.cur_char, Some(b'0'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::INT_CONST));
        assert_eq!(t.intVal(), Some(0));
        assert_eq!(t.cur_char, Some(b'0'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b')'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(")")));
        assert_eq!(t.cur_char, Some(b')'));

        assert_eq!(t.hasMoreTokens(), true);
        assert_eq!(t.cur_char, Some(b';'));
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(";")));
        assert_eq!(t.cur_char, Some(b';'));

        assert_eq!(t.hasMoreTokens(), false);
    }

    #[test]
    fn advance_letstatement4() {
        let s = io::Cursor::new("\
        let a[100] = 77; \r\n\
        ");
        let mut t = JackTokenizer::new(s);

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::KEYWORD));
        assert_eq!(t.keywordType(), Some(KeywordType::LET));

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::IDENTIFIER));
        assert_eq!(t.identifier(), Some(String::from("a")));
        
        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("[")));

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::INT_CONST));
        assert_eq!(t.intVal(), Some(100));

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("]")));

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from("=")));

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::INT_CONST));
        assert_eq!(t.intVal(), Some(77));

        assert_eq!(t.hasMoreTokens(), true);
        t.advance();
        assert_eq!(t.tokenType(), Some(TokenType::SYMBOL));
        assert_eq!(t.symbol(), Some(String::from(";")));

        assert_eq!(t.hasMoreTokens(), false);
    }

    // #[test]
    fn tokenizer_square_main() {
        let s = std::fs::File::open("Square/Main.jack");
        let mut t = JackTokenizer::new(s.unwrap());
        let tr = t.to_xml();
        let mut result_string = String::new();
        std::fs::File::open("Square/MainT.xml").unwrap().read_to_string(&mut result_string);
        std::fs::File::create("Square/MainT_tokenizer.xml").unwrap().write_all(tr.as_bytes());

        assert_eq!(result_string, tr);
    }

    // #[test]
    fn tokenizer_square_square() {
        let s = std::fs::File::open("Square/Square.jack");
        let mut t = JackTokenizer::new(s.unwrap());
        let tr = t.to_xml();
        let mut result_string = String::new();
        std::fs::File::open("Square/SquareT.xml").unwrap().read_to_string(&mut result_string);
        std::fs::File::create("Square/SquareT_tokenizer.xml").unwrap().write_all(tr.as_bytes());

        assert_eq!(result_string, tr);
    }

    // #[test]
    fn tokenizer_square_squaregame() {
        let s = std::fs::File::open("Square/SquareGame.jack");
        let mut t = JackTokenizer::new(s.unwrap());
        let tr = t.to_xml();
        let mut result_string = String::new();
        std::fs::File::open("Square/SquareGameT.xml").unwrap().read_to_string(&mut result_string);
        std::fs::File::create("Square/SquareGameT_tokenizer.xml").unwrap().write_all(tr.as_bytes());

        assert_eq!(result_string, tr);
    }

    // #[test]
    fn tokenizer_arraytest() {
        let s = std::fs::File::open("ArrayTest/Main.jack");
        let mut t = JackTokenizer::new(s.unwrap());
        let tr = t.to_xml();
        let mut result_string = String::new();
        std::fs::File::open("ArrayTest/MainT.xml").unwrap().read_to_string(&mut result_string);
        std::fs::File::create("ArrayTest/MainT_tokenizer.xml").unwrap().write_all(tr.as_bytes());

        assert_eq!(result_string, tr);
    }
}
