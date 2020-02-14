use std::io::{self, BufRead, Read, Seek, Write};
use super::JackTokenizer::*;

pub struct CompilationEngine<R: io::Read + io::Seek, W: io::Write> {
    tokenizer: JackTokenizer<R>,
    fs: io::BufWriter<W>,
    current_token: Token,
    level: usize,
    is_lookahead: bool,
}

enum NodeType {
    CLASS,
    CLASS_VAR_DEC,
    SUBROUTINE_DEC,
    PARAMETER_LIST,
    SUBROUTINE_BODY,
    VAR_DEC,
    STATEMENTS,
    LET_STATEMENT,
    IF_STATEMENT,
    WHILE_STATEMENT,
    DO_STATEMENT,
    RETURN_STATEMENT,
    EXPRESSION_LIST,
    EXPRESSION,
    TERM
}

impl<R: io::Read + io::Seek, W: io::Write> CompilationEngine<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        CompilationEngine {
            tokenizer: JackTokenizer::new(reader),
            fs: io::BufWriter::new(writer),
            current_token: Token::Keyword(KeywordType::CLASS),
            level: 0,
            is_lookahead: false,
        }
    }

    fn consume(&mut self) {
        if self.is_lookahead {
            return;
        }
        self.current_token = self.tokenizer.next().unwrap();
        self.is_lookahead = true;
    }

    fn write_node_start(&mut self, node_type: NodeType) {
        let s = indentation(&create_open_tag(&convert_node(node_type)), self.level);
        self.fs.write_all(s.as_bytes());
        self.level += 2;
    }

    fn write_node_end(&mut self, node_type: NodeType) {
        self.level -= 2;
        let s = indentation(&create_close_tag(&convert_node(node_type)), self.level);
        self.fs.write_all(s.as_bytes());
    }

    fn write_current_token(&mut self) {
        if !self.is_lookahead {
            self.consume();
        }
        let s = to_xml_elem(&self.current_token, self.level);
        self.fs.write_all(s.as_bytes());
        self.is_lookahead = false;
    }

    pub fn compileClass(&mut self) {
        self.write_node_start(NodeType::CLASS);

        // class
        self.write_current_token();
        
        // className
        self.write_current_token();

        // {
        self.write_current_token();
        
        while {self.consume();
        self.current_token == Token::Keyword(KeywordType::STATIC) 
        || self.current_token == Token::Keyword(KeywordType::FIELD)} {
            self.compileClassVarDec();
        }
        
        while {self.consume();
        self.current_token == Token::Keyword(KeywordType::CONSTRUCTOR) 
        || self.current_token == Token::Keyword(KeywordType::FUNCTION)
        || self.current_token == Token::Keyword(KeywordType::METHOD)} {
            self.compileSubroutineDec();
        }

        // }
        self.write_current_token();

        self.write_node_end(NodeType::CLASS);
    }

    pub fn compileClassVarDec(&mut self) {
        self.write_node_start(NodeType::CLASS_VAR_DEC);

        // static/field
        self.write_current_token();

        // type
        self.write_current_token();

        // varName
        self.write_current_token();

        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            // ,
            self.write_current_token();
 
            // varName
            self.write_current_token();
        }

        // ;
        self.write_current_token();

        self.write_node_end(NodeType::CLASS_VAR_DEC);
    }

    pub fn compileSubroutineDec(&mut self) {
        self.write_node_start(NodeType::SUBROUTINE_DEC);

        // constructor/function/method
        self.write_current_token();

        // void|type
        self.write_current_token();

        // subroutineName
        self.write_current_token();

        // (
        self.write_current_token();

        // parameterList
        self.compileParameterList();

        // )
        self.write_current_token();

        // subroutineBody
        self.compileSubroutineBody();

        self.write_node_end(NodeType::SUBROUTINE_DEC);
    }

    pub fn compileParameterList(&mut self) {
        self.write_node_start(NodeType::PARAMETER_LIST);

        self.consume();
        // if not type then empty (should be ")")
        if !(self.current_token == Token::Keyword(KeywordType::INT)
        || self.current_token == Token::Keyword(KeywordType::CHAR)
        || self.current_token == Token::Keyword(KeywordType::BOOLEAN)
        || enum_eq(&self.current_token, &Token::Identifier("".to_string()))) {
            self.write_node_end(NodeType::PARAMETER_LIST);
            return;
        }

        // type
        self.write_current_token();

        //varName
        self.write_current_token();
        
        // , type varName
        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            // ,
            self.write_current_token();

            //type
            self.write_current_token();

            //varName
            self.write_current_token();
        }
        self.write_node_end(NodeType::PARAMETER_LIST);
    }

    pub fn compileSubroutineBody(&mut self) {
        self.write_node_start(NodeType::SUBROUTINE_BODY);

        // {
        self.write_current_token();

        // varDec*
        while {self.consume();
        self.current_token == Token::Keyword(KeywordType::VAR)} {
            self.compileVarDec();
        }

        // statements
        self.compileStatementes();

        // }
        self.write_current_token();
        
        self.write_node_end(NodeType::SUBROUTINE_BODY);
    }

    pub fn compileVarDec(&mut self) {
        self.write_node_start(NodeType::VAR_DEC);

        // var
        self.write_current_token();

        // type
        self.write_current_token();

        // varName
        self.write_current_token();

        // (, varName)*
        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            //,
            self.write_current_token();

            // varName
            self.write_current_token();
        }
        
        // ;
        self.write_current_token();

        self.write_node_end(NodeType::VAR_DEC);
    }

    pub fn compileStatementes(&mut self) {
        self.write_node_start(NodeType::STATEMENTS);
        
        loop {
            self.consume();
            match self.current_token {
                Token::Keyword(KeywordType::LET) => self.compileLet(),
                Token::Keyword(KeywordType::IF) => self.compileIf(),
                Token::Keyword(KeywordType::WHILE) => self.compileWhile(),
                Token::Keyword(KeywordType::DO) => self.compileDo(),
                Token::Keyword(KeywordType::RETURN) => self.compileReturn(),
                _ => break, // should be }
            }
        }

        self.write_node_end(NodeType::STATEMENTS);
    }

    pub fn compileLet(&mut self) {
        self.write_node_start(NodeType::LET_STATEMENT);

        // let
        self.write_current_token();

        // varName
        self.write_current_token();

        // [ or =
        self.consume();
        if self.current_token == Token::Symbol("[".to_string()) {
            // [
            self.write_current_token();
            
            self.compileExpression();

            // ] 
            self.write_current_token();
            
            // = 
            self.write_current_token();
        }
        else {
            // = 
            self.write_current_token();        
        }

        self.compileExpression();

        // ;
        self.write_current_token();
        
        self.write_node_end(NodeType::LET_STATEMENT);
    }

    pub fn compileIf(&mut self) {
        self.write_node_start(NodeType::IF_STATEMENT);

        // if
        self.write_current_token();

        // (
        self.write_current_token();
        
        self.compileExpression();

        // )
        self.write_current_token();

        // {
        self.write_current_token();

        self.compileStatementes();

        // }
        self.write_current_token();

        self.consume();
        if self.current_token == Token::Keyword(KeywordType::ELSE) {
            // else
            self.write_current_token();

            // {
            self.write_current_token();

            self.compileStatementes();

            // }
            self.write_current_token();
        }

        self.write_node_end(NodeType::IF_STATEMENT);
    }

    pub fn compileWhile(&mut self) {
        self.write_node_start(NodeType::WHILE_STATEMENT);

        // while
        self.write_current_token();

        // (
        self.write_current_token();

        self.compileExpression();

        // )
        self.write_current_token();

        // {
        self.write_current_token();

        self.compileStatementes();

        // }
        self.write_current_token();

        self.write_node_end(NodeType::WHILE_STATEMENT);
    }

    pub fn compileDo(&mut self) {
        self.write_node_start(NodeType::DO_STATEMENT);

        // do
        self.write_current_token();

        // identifier
        self.write_current_token();

        self.consume();
        if self.current_token == Token::Symbol("(".to_string()) {
            // (
            self.write_current_token();

            self.compileExpressionList();

            // )
            self.write_current_token();
        } 
        else if self.current_token == Token::Symbol(".".to_string()) {
            // .
            self.write_current_token();

            // subroutineName
            self.write_current_token();

            // (
            self.write_current_token();

            self.compileExpressionList();

            // )
            self.write_current_token();
        } 

        // ;
        self.write_current_token();
       
        self.write_node_end(NodeType::DO_STATEMENT);
    }

    pub fn compileReturn(&mut self) {
        self.write_node_start(NodeType::RETURN_STATEMENT);
        
        // return
        self.write_current_token();

        self.consume();
        if self.current_token != Token::Symbol(";".to_string())
        {
            self.compileExpression();
        }

        // ;
        self.write_current_token();
        
        self.write_node_end(NodeType::RETURN_STATEMENT);
    }

    pub fn compileExpressionList(&mut self) {
        self.write_node_start(NodeType::EXPRESSION_LIST);

        self.consume();
        if self.current_token == Token::Symbol(")".to_string()) {
            // Empty
            self.write_node_end(NodeType::EXPRESSION_LIST);
            return;
        }

        self.compileExpression();

        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            // ,
            self.write_current_token();

            self.compileExpression();
        }

        self.write_node_end(NodeType::EXPRESSION_LIST);
    }

    pub fn compileExpression(&mut self) {
        self.write_node_start(NodeType::EXPRESSION);

        self.compileTerm();

        while {self.consume();
        self.current_token == Token::Symbol("+".to_string())
        || self.current_token == Token::Symbol("-".to_string()) 
        || self.current_token == Token::Symbol("*".to_string()) 
        || self.current_token == Token::Symbol("/".to_string()) 
        || self.current_token == Token::Symbol("&".to_string()) 
        || self.current_token == Token::Symbol("|".to_string()) 
        || self.current_token == Token::Symbol("<".to_string()) 
        || self.current_token == Token::Symbol(">".to_string()) 
        || self.current_token == Token::Symbol("=".to_string())} {
            // op
            self.write_current_token();

            self.compileTerm();
        }

        self.write_node_end(NodeType::EXPRESSION);
    }

    pub fn compileTerm(&mut self) {
        self.write_node_start(NodeType::TERM);

        self.consume();
        if self.current_token == Token::Symbol("(".to_string()) {
            // (
            self.write_current_token();

            // expression
            self.compileExpression();

            // )
            self.write_current_token();
        }        
        else if self.current_token == Token::Symbol("-".to_string())
        || self.current_token == Token::Symbol("~".to_string()) {
            // unaryOp
            self.write_current_token();

            self.compileTerm();
        }
        else {
            // identifier
            self.write_current_token();

            self.consume();
            if self.current_token == Token::Symbol(".".to_string()) {
                // .
                self.write_current_token();

                // subroutineName
                self.write_current_token();

                // (
                self.write_current_token();

                self.compileExpressionList();

                // )
                self.write_current_token();
            } 
            else if self.current_token == Token::Symbol("(".to_string()) {
                // (
                self.write_current_token();

                self.compileExpressionList();

                // )
                self.write_current_token();
            } 
            else if self.current_token == Token::Symbol("[".to_string()) {
                // [
                self.write_current_token();

                self.compileExpression();
    
                // ]
                self.write_current_token();
            }
            else
            {
                // integerConst or StringConst or KeywordConst or varName
                // Nothing to do.
            }
        }

        self.write_node_end(NodeType::TERM);
    }
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

fn convert_node(node_type: NodeType) -> String {
    match node_type {
        NodeType::CLASS => "class",
        NodeType::CLASS_VAR_DEC => "classVarDec",
        NodeType::SUBROUTINE_DEC => "subroutineDec",
        NodeType::PARAMETER_LIST => "parameterList",
        NodeType::SUBROUTINE_BODY => "subroutineBody",
        NodeType::VAR_DEC => "varDec",
        NodeType::STATEMENTS => "statements",
        NodeType::LET_STATEMENT => "letStatement",
        NodeType::IF_STATEMENT => "ifStatement",
        NodeType::WHILE_STATEMENT => "whileStatement",
        NodeType::DO_STATEMENT => "doStatement",
        NodeType::RETURN_STATEMENT => "returnStatement",
        NodeType::EXPRESSION_LIST => "expressionList",
        NodeType::EXPRESSION => "expression",
        NodeType::TERM => "term",
    }.to_string()
}

fn to_xml_elem(token: &Token, level: usize) -> String {
    let elem = match token {
        Token::Keyword(k) => ["keyword".to_string(), convert_keyword(k.clone())],
        Token::Symbol(s) => ["symbol".to_string(), escape_symbol(&s)],
        Token::Identifier(i) => ["identifier".to_string(), i.clone()],
        Token::IntConst(i) => ["integerConstant".to_string(), i.to_string()],
        Token::StringConst(s) => ["stringConstant".to_string(), s.clone()]
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

    #[test]
    fn ClassVarDec() {
        let s = io::Cursor::new("\
        class Main {\r\n\
            field int x, y;\r\n\
            field int size;\r\n\
        }\r\n\
        ");
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w);

        c.compileClass();
        let mut r = r#"
<class>
  <keyword> class </keyword>
  <identifier> Main </identifier>
  <symbol> { </symbol>
  <classVarDec>
    <keyword> field </keyword>
    <keyword> int </keyword>
    <identifier> x </identifier>
    <symbol> , </symbol>
    <identifier> y </identifier>
    <symbol> ; </symbol>
  </classVarDec>
  <classVarDec>
    <keyword> field </keyword>
    <keyword> int </keyword>
    <identifier> size </identifier>
    <symbol> ; </symbol>
  </classVarDec>
  <symbol> } </symbol>
</class>
"#.to_string();
        r.remove(0);
        r = r.replace("\n", "\r\n");
        assert_eq!(String::from_utf8(c.fs.buffer().to_vec()).unwrap(), r);
    }

    #[test]
    fn SubroutineDec_simple() {
        let s = io::Cursor::new("\
        class Main {\r\n\
            function void main() {\r\n\
                return;\r\n\
            }\r\n\
        }\r\n\
        ");
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w);

        c.compileClass();
        let mut r = r#"
<class>
  <keyword> class </keyword>
  <identifier> Main </identifier>
  <symbol> { </symbol>
  <subroutineDec>
    <keyword> function </keyword>
    <keyword> void </keyword>
    <identifier> main </identifier>
    <symbol> ( </symbol>
    <parameterList>
    </parameterList>
    <symbol> ) </symbol>
    <subroutineBody>
      <symbol> { </symbol>
      <statements>
        <returnStatement>
          <keyword> return </keyword>
          <symbol> ; </symbol>
        </returnStatement>
      </statements>
      <symbol> } </symbol>
    </subroutineBody>
  </subroutineDec>
  <symbol> } </symbol>
</class>
"#.to_string();
        r.remove(0);
        r = r.replace("\n", "\r\n");
        assert_eq!(String::from_utf8(c.fs.buffer().to_vec()).unwrap(), r);
    }
    #[test]
    fn Subroutine_do_statement() {
        let s = io::Cursor::new("\
        class Main {\r\n\
            function void main() {\r\n\
                var SquareGame game;\r\n\
                let game = game;\r\n\
                do game.run();\r\n\
            }\r\n\
        }\r\n\
        ");
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w);

        c.compileClass();
        let mut r = r#"
<class>
  <keyword> class </keyword>
  <identifier> Main </identifier>
  <symbol> { </symbol>
  <subroutineDec>
    <keyword> function </keyword>
    <keyword> void </keyword>
    <identifier> main </identifier>
    <symbol> ( </symbol>
    <parameterList>
    </parameterList>
    <symbol> ) </symbol>
    <subroutineBody>
      <symbol> { </symbol>
      <varDec>
        <keyword> var </keyword>
        <identifier> SquareGame </identifier>
        <identifier> game </identifier>
        <symbol> ; </symbol>
      </varDec>
      <statements>
        <letStatement>
          <keyword> let </keyword>
          <identifier> game </identifier>
          <symbol> = </symbol>
          <expression>
            <term>
              <identifier> game </identifier>
            </term>
          </expression>
          <symbol> ; </symbol>
        </letStatement>
        <doStatement>
          <keyword> do </keyword>
          <identifier> game </identifier>
          <symbol> . </symbol>
          <identifier> run </identifier>
          <symbol> ( </symbol>
          <expressionList>
          </expressionList>
          <symbol> ) </symbol>
          <symbol> ; </symbol>
        </doStatement>
      </statements>
      <symbol> } </symbol>
    </subroutineBody>
  </subroutineDec>
  <symbol> } </symbol>
</class>
"#.to_string();
        r.remove(0);
        r = r.replace("\n", "\r\n");
        assert_eq!(String::from_utf8(c.fs.buffer().to_vec()).unwrap(), r);
    }

    #[test]
    fn analyze_arraytest() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("ArrayTest/Main.jack");
            let w = std::fs::File::create("ArrayTest/Main_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("ArrayTest/Main.xml").unwrap();
        let al = std::fs::read_to_string("ArrayTest/Main_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn analyze_ExpressionLessSquare_Main() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("ExpressionLessSquare/Main.jack");
            let w = std::fs::File::create("ExpressionLessSquare/Main_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("ExpressionLessSquare/Main.xml").unwrap();
        let al = std::fs::read_to_string("ExpressionLessSquare/Main_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn analyze_ExpressionLessSquare_Square1() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("ExpressionLessSquare/Square.jack");
            let w = std::fs::File::create("ExpressionLessSquare/Square_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("ExpressionLessSquare/Square.xml").unwrap();
        let al = std::fs::read_to_string("ExpressionLessSquare/Square_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn analyze_ExpressionLessSquare_SquareGame() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("ExpressionLessSquare/SquareGame.jack");
            let w = std::fs::File::create("ExpressionLessSquare/SquareGame_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("ExpressionLessSquare/SquareGame.xml").unwrap();
        let al = std::fs::read_to_string("ExpressionLessSquare/SquareGame_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn analyze_Square_Main() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Square/Main.jack");
            let w = std::fs::File::create("Square/Main_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Square/Main.xml").unwrap();
        let al = std::fs::read_to_string("Square/Main_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }


    #[test]
    fn analyze_Square_Square1() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Square/Square.jack");
            let w = std::fs::File::create("Square/Square_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Square/Square.xml").unwrap();
        let al = std::fs::read_to_string("Square/Square_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }


    #[test]
    fn analyze_Square_SquareGame() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Square/SquareGame.jack");
            let w = std::fs::File::create("Square/SquareGame_analyzer.xml");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap());
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Square/SquareGame.xml").unwrap();
        let al = std::fs::read_to_string("Square/SquareGame_analyzer.xml").unwrap();
        assert_eq!(result_string, al);
    }
}