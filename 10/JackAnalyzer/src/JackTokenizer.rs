use std::io::{self, BufRead, Read, Seek};

pub struct JackTokenizer<R: io::Read + io::Seek> {
    fs: io::BufReader<R>,
    token_type: Option<TokenType>,
    cur_char: Option<u8>,
    token_buf: String,
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

impl<R: io::Read + io::Seek> JackTokenizer<R> {
    pub fn new(reader: R) -> Self {
        JackTokenizer {
            fs: io::BufReader::new(reader),
            token_type: None,
            cur_char: None,
            token_buf: String::new(),
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
}
