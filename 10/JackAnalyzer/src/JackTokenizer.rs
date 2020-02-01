use std::io::{self, BufRead, Read};

pub struct JackTokenizer<R: io::Read> {
    fs: io::BufReader<R>,
    token_type: Option<TokenType>,
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

impl<R: io::Read> JackTokenizer<R> {
    pub fn new(reader: R) -> Self {
        JackTokenizer {
            fs: io::BufReader::new(reader),
            token_type: None,
        }
    }

    pub fn hasMoreTokens(mut self) -> bool {
        self.fs.bytes().next().is_some()
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
    fn hasMoreTokens() {
        let s = io::Cursor::new("// comment\r\n@aaaa");
        let t = JackTokenizer::new(s);
        assert_eq!(t.hasMoreTokens(), true);

        let s1 = io::Cursor::new("");
        let t1 = JackTokenizer::new(s1);
        assert_eq!(t1.hasMoreTokens(), false);
    }
}
