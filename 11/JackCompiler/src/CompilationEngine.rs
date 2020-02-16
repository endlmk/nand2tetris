use std::io::{self, BufRead, Read, Seek, Write};
use super::JackTokenizer::*;
use super::SymbolTable::*;

pub struct CompilationEngine<R: io::Read + io::Seek, W: io::Write> {
    tokenizer: JackTokenizer<R>,
    fs: io::BufWriter<W>,
    current_token: Token,
    level: usize,
    is_lookahead: bool,
    table: SymbolTable,
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

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
enum IdentifierCategory {
    VAR,
    ARG,
    STATIC,
    FIELD,
    CLASS,
    SUBROUTINE,
}

impl std::fmt::Display for IdentifierCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            IdentifierCategory::VAR => "var",
            IdentifierCategory::ARG => "argument",
            IdentifierCategory::STATIC => "static",
            IdentifierCategory::FIELD => "field",
            IdentifierCategory::CLASS => "class",
            IdentifierCategory::SUBROUTINE => "subroutine",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
enum IdentifierUsage {
    DEFIEND,
    USED,
}

impl std::fmt::Display for IdentifierUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            IdentifierUsage::DEFIEND => "defined",
            IdentifierUsage::USED => "used",
        };
        write!(f, "{}", s)
    }
}

struct IdentifierInfo {
    name: String,
    cat: IdentifierCategory,
    usage: IdentifierUsage,
    varKind: Option<VarKind>,
    index: Option<i32>,
}

impl std::fmt::Display for IdentifierInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut v = vec![self.name.clone(), self.cat.to_string(), self.usage.to_string()];
        if self.varKind.is_some() {
            v.push(self.varKind.as_ref().unwrap().to_string());
        }
        if self.index.is_some() {
            v.push(self.index.unwrap().to_string());
        }
        let s = v.join(", ");
        write!(f, "{}", s)
    }
}

impl<R: io::Read + io::Seek, W: io::Write> CompilationEngine<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        CompilationEngine {
            tokenizer: JackTokenizer::new(reader),
            fs: io::BufWriter::new(writer),
            current_token: Token::Keyword(KeywordType::CLASS),
            level: 0,
            is_lookahead: false,
            table: SymbolTable::new(),
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

    fn get_current_token(&mut self) -> &Token {
        if !self.is_lookahead {
            self.consume();
        }
        &self.current_token
    }

    fn write_identifier_info(&mut self, info: &IdentifierInfo) {
        let l = self.level;
        let s = to_identifier_xml_elem(info, l);
        self.fs.write_all(s.as_bytes());
        self.is_lookahead = false;
    }

    fn write_current_token(&mut self) {
        let l = self.level;
        let s = to_xml_elem(self.get_current_token(), l);
        self.fs.write_all(s.as_bytes());
        self.is_lookahead = false;
    }

    fn write_token(&mut self, tk: &Token) {
        let l = self.level;
        let s = to_xml_elem(tk, l);
        self.fs.write_all(s.as_bytes());
    }

    pub fn compileClass(&mut self) {
        self.write_node_start(NodeType::CLASS);

        // class
        self.write_current_token();
        
        // className
        let tk = self.get_current_token();
        let elem = convert_token_to_strings(tk);
        let info = IdentifierInfo{
            name: elem[1].clone(),
            cat: IdentifierCategory::CLASS,
            usage: IdentifierUsage::DEFIEND,
            varKind: None,
            index: None,   
        };
        self.write_identifier_info(&info);

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
        let tk_varKind = self.get_current_token();
        let mut varKind_t = None;
        if let Token::Keyword(kw) = tk_varKind {
            if *kw == KeywordType::STATIC {
                varKind_t = Some(VarKind::STATIC);
            }
            else if *kw == KeywordType::FIELD {
                varKind_t = Some(VarKind::FIELD);
            } 
        }
        let varKind = varKind_t.unwrap();
        self.write_current_token();

        // type
        let tk_type = self.get_current_token();
        let elem = convert_token_to_strings(tk_type);
        let type_name = &elem[1];
        self.write_current_token();

        // varName
        let tk = self.get_current_token();
        let elem = convert_token_to_strings(tk);
        self.table.define(&elem[1], type_name, &varKind.clone());
        let info = IdentifierInfo{
            name: elem[1].clone(),
            cat: IdentifierCategory::FIELD,
            usage: IdentifierUsage::DEFIEND,
            varKind: Some(varKind.clone()),
            index: Some(self.table.indexOf(&elem[1])),   
        };
        self.write_identifier_info(&info);

        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            // ,
            self.write_current_token();

            // varName
            let tk = self.get_current_token();
            let elem = convert_token_to_strings(tk);
            self.table.define(&elem[1], type_name, &varKind);
            let info = IdentifierInfo{
                name: elem[1].clone(),
                cat: IdentifierCategory::FIELD,
                usage: IdentifierUsage::DEFIEND,
                varKind: Some(varKind.clone()),
                index: Some(self.table.indexOf(&elem[1])),   
            };
            self.write_identifier_info(&info);
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
        let tk = self.get_current_token();
        let elem = convert_token_to_strings(tk);
        let info = IdentifierInfo{
            name: elem[1].clone(),
            cat: IdentifierCategory::SUBROUTINE,
            usage: IdentifierUsage::DEFIEND,
            varKind: None,
            index: None,   
        };
        self.write_identifier_info(&info);

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
        let tk_varKind = self.get_current_token();
        let mut varKind_t = None;
        if let Token::Keyword(kw) = tk_varKind {
            if *kw == KeywordType::VAR {
                varKind_t = Some(VarKind::VAR);
            }
        }
        let varKind = varKind_t.unwrap();
        self.write_current_token();

        // type
        let tk_type = self.get_current_token();
        let elem = convert_token_to_strings(tk_type);
        let type_name = &elem[1];
        if let Token::Identifier(_) = tk_type {
            let info = IdentifierInfo{
                name: elem[1].clone(),
                cat: IdentifierCategory::CLASS,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,   
            };
            self.write_identifier_info(&info);
        }
        else {
            self.write_current_token();
        }

        // varName
        let tk = self.get_current_token();
        let elem = convert_token_to_strings(tk);
        self.table.define(&elem[1], type_name, &varKind.clone());
        let info = IdentifierInfo{
            name: elem[1].clone(),
            cat: convert_varKind_to_IdentifierCategory(&varKind),
            usage: IdentifierUsage::DEFIEND,
            varKind: Some(varKind.clone()),
            index: Some(self.table.indexOf(&elem[1])),   
        };
        self.write_identifier_info(&info);

        // (, varName)*
        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            //,
            self.write_current_token();

            // varName
            let tk = self.get_current_token();
            let elem = convert_token_to_strings(tk);
            self.table.define(&elem[1], type_name, &varKind.clone());
            let info = IdentifierInfo{
                name: elem[1].clone(),
                cat: IdentifierCategory::FIELD,
                usage: IdentifierUsage::DEFIEND,
                varKind: Some(varKind.clone()),
                index: Some(self.table.indexOf(&elem[1])),   
            };
            self.write_identifier_info(&info);
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
        let tk = self.get_current_token();
        let elem = convert_token_to_strings(tk);
        let vk = self.table.kindOf(&elem[1]);
        let info = IdentifierInfo{
            name: elem[1].clone(),
            cat: convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()),
            usage: IdentifierUsage::USED,
            varKind: vk,
            index: Some(self.table.indexOf(&elem[1])),   
        };
        self.write_identifier_info(&info);

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
        let tk = self.get_current_token().clone();
        // To consume, turn off flag
        self.is_lookahead = false;

        self.consume();
        if self.current_token == Token::Symbol("(".to_string()) {
            let elem = convert_token_to_strings(&tk);
            let info = IdentifierInfo {
                name: elem[1].clone(),
                cat: IdentifierCategory::SUBROUTINE,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,
            };
            self.write_identifier_info(&info);
            // lookahead is not processed, turn on flag
            self.is_lookahead = true;

            // (
            self.write_current_token();

            self.compileExpressionList();

            // )
            self.write_current_token();
        } 
        else if self.current_token == Token::Symbol(".".to_string()) {
            let elem = convert_token_to_strings(&tk);
            let vk = self.table.kindOf(&elem[1]);
            let info = IdentifierInfo{
                name: elem[1].clone(),
                cat: if vk.is_none() { IdentifierCategory::CLASS } else { convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()) },
                usage: IdentifierUsage::USED,
                varKind: vk.clone(),
                index: if vk.is_none() { None } else { Some(self.table.indexOf(&elem[1])) },   
            };
            self.write_identifier_info(&info);
            // lookahead is not processed, turn on flag
            self.is_lookahead = true;

            // .
            self.write_current_token();

            // subroutineName
            let tk = self.get_current_token();
            let elem = convert_token_to_strings(&tk);
            let info = IdentifierInfo {
                name: elem[1].clone(),
                cat: IdentifierCategory::SUBROUTINE,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,
            };
            self.write_identifier_info(&info);

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
            let tk = self.get_current_token().clone();
            // To consume, turn off flag
            self.is_lookahead = false;

            self.consume();
            let elem = convert_token_to_strings(&self.current_token);
            assert_eq!(elem[0], "symbol".to_string());
            assert_eq!(elem[1], ";".to_string());                
            if self.current_token == Token::Symbol(".".to_string()) {
                let elem = convert_token_to_strings(&tk);
                let info = IdentifierInfo {
                    name: elem[1].clone(),
                    cat: IdentifierCategory::CLASS,
                    usage: IdentifierUsage::USED,
                    varKind: None,
                    index: None,
                };
                self.write_identifier_info(&info);
                // lookahead is not processed, turn on flag
                self.is_lookahead = true;

                // .
                self.write_current_token();

                // subroutineName
                let tk = self.get_current_token().clone();
                let elem = convert_token_to_strings(&tk);
                let info = IdentifierInfo {
                    name: elem[1].clone(),
                    cat: IdentifierCategory::SUBROUTINE,
                    usage: IdentifierUsage::USED,
                    varKind: None,
                    index: None,
                };
                self.write_identifier_info(&info);

                // (
                self.write_current_token();

                self.compileExpressionList();

                // )
                self.write_current_token();
            } 
            else if self.current_token == Token::Symbol("(".to_string()) {
                let elem = convert_token_to_strings(&tk);
                let info = IdentifierInfo {
                    name: elem[1].clone(),
                    cat: IdentifierCategory::SUBROUTINE,
                    usage: IdentifierUsage::USED,
                    varKind: None,
                    index: None,
                };
                self.write_identifier_info(&info);
                // lookahead is not processed, turn on flag
                self.is_lookahead = true;

                // (
                self.write_current_token();

                self.compileExpressionList();

                // )
                self.write_current_token();
            } 
            else if self.current_token == Token::Symbol("[".to_string()) {
                let elem = convert_token_to_strings(&tk);
                let vk = self.table.kindOf(&elem[1]);
                let info = IdentifierInfo{
                    name: elem[1].clone(),
                    cat: convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()),
                    usage: IdentifierUsage::USED,
                    varKind: vk,
                    index: Some(self.table.indexOf(&elem[1])),   
                };
                self.write_identifier_info(&info);
                // lookahead is not processed, turn on flag
                self.is_lookahead = true;
                
                // [
                self.write_current_token();

                self.compileExpression();
    
                // ]
                self.write_current_token();
            }
            else {
                // varName
                if let Token::Identifier(_) = tk {
                    let elem = convert_token_to_strings(&tk);
                    let vk = self.table.kindOf(&elem[1]);
                    let info = IdentifierInfo{
                        name: elem[1].clone(),
                        cat: convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()),
                        usage: IdentifierUsage::USED,
                        varKind: vk,
                        index: Some(self.table.indexOf(&elem[1])),   
                    };
                    self.write_identifier_info(&info);
                    let elem = convert_token_to_strings(&self.current_token);
                    assert_eq!(elem[0], "symbol".to_string());         
                }
                else {
                    // integerConst or StringConst or KeywordConst
                    self.write_token(&tk);
                }

                // lookahead is not processed, turn on flag
                self.is_lookahead = true;
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

fn convert_token_to_strings(token: &Token) -> [String; 2]
{
    match token {
        Token::Keyword(k) => ["keyword".to_string(), convert_keyword(k.clone())],
        Token::Symbol(s) => ["symbol".to_string(), escape_symbol(&s)],
        Token::Identifier(i) => ["identifier".to_string(), i.clone()],
        Token::IntConst(i) => ["integerConstant".to_string(), i.to_string()],
        Token::StringConst(s) => ["stringConstant".to_string(), s.clone()]
    }
}

fn convert_varKind_to_IdentifierCategory(varKind: &VarKind) -> IdentifierCategory {
    match varKind {
        VarKind::VAR => IdentifierCategory::VAR,
        VarKind::ARG => IdentifierCategory::ARG,
        VarKind::STATIC => IdentifierCategory::STATIC,
        VarKind::FIELD => IdentifierCategory::FIELD,
    }
}

fn to_xml_elem(token: &Token, level: usize) -> String {
    let elem = convert_token_to_strings(token);
    indentation(&create_xml_elem(&elem[0], &elem[1]), level)
}

fn to_identifier_xml_elem(info: &IdentifierInfo, level: usize) -> String {
    indentation(&create_xml_elem("identifier", &info.to_string()), level)
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
  <identifier> Main, class, defined </identifier>
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
  <identifier> Main, class, defined </identifier>
  <symbol> { </symbol>
  <classVarDec>
    <keyword> field </keyword>
    <keyword> int </keyword>
    <identifier> x, field, defined, field, 0 </identifier>
    <symbol> , </symbol>
    <identifier> y, field, defined, field, 1 </identifier>
    <symbol> ; </symbol>
  </classVarDec>
  <classVarDec>
    <keyword> field </keyword>
    <keyword> int </keyword>
    <identifier> size, field, defined, field, 2 </identifier>
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
  <identifier> Main, class, defined </identifier>
  <symbol> { </symbol>
  <subroutineDec>
    <keyword> function </keyword>
    <keyword> void </keyword>
    <identifier> main, subroutine, defined </identifier>
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
  <identifier> Main, class, defined </identifier>
  <symbol> { </symbol>
  <subroutineDec>
    <keyword> function </keyword>
    <keyword> void </keyword>
    <identifier> main, subroutine, defined </identifier>
    <symbol> ( </symbol>
    <parameterList>
    </parameterList>
    <symbol> ) </symbol>
    <subroutineBody>
      <symbol> { </symbol>
      <varDec>
        <keyword> var </keyword>
        <identifier> SquareGame, class, used </identifier>
        <identifier> game, var, defined, var, 0 </identifier>
        <symbol> ; </symbol>
      </varDec>
      <statements>
        <letStatement>
          <keyword> let </keyword>
          <identifier> game, var, used, var, 0 </identifier>
          <symbol> = </symbol>
          <expression>
            <term>
              <identifier> game, var, used, var, 0 </identifier>
            </term>
          </expression>
          <symbol> ; </symbol>
        </letStatement>
        <doStatement>
          <keyword> do </keyword>
          <identifier> game, var, used, var, 0 </identifier>
          <symbol> . </symbol>
          <identifier> run, subroutine, used </identifier>
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
}