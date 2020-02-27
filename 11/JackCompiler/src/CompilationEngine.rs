use std::io::{self, BufRead, Read, Seek, Write};
use super::JackTokenizer::*;
use super::SymbolTable::*;
use super::VMWriter::*;

pub struct CompilationEngine<R: io::Read + io::Seek, W: io::Write> {
    tokenizer: JackTokenizer<R>,
    fs: Option<io::BufWriter<W>>,
    current_token: Token,
    level: usize,
    is_lookahead: bool,
    table: SymbolTable,
    xml_mode: bool,
    vw: VMWriter<W>,
    class_name: String, 
    subroutine_name: String,
    next_if_label: i32,
    next_while_label: i32,
    is_constructor: bool,
    is_method: bool,
    fields_count: i32,
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
    pub fn new(reader: R, writer_vm: W, writer_xml: Option<W>) -> Self {
        CompilationEngine {
            tokenizer: JackTokenizer::new(reader),
            fs: match writer_xml {
                Some(w) => Some(io::BufWriter::new(w)),
                None => None,
            },
            current_token: Token::Keyword(KeywordType::CLASS),
            level: 0,
            is_lookahead: false,
            table: SymbolTable::new(),
            xml_mode: true,
            vw: VMWriter::new(writer_vm),
            class_name: "".to_string(),
            subroutine_name: "".to_string(),
            next_if_label: 0,
            next_while_label: 0,
            is_constructor: false,
            is_method: false,
            fields_count: 0,
        }
    }

    fn consume_eq(&mut self, tk: &Token) -> bool {
        if self.current_token != *tk {
            return false;
        }
        self.current_token = self.tokenizer.next().unwrap();
        true
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
        if let Some(w) = &mut self.fs { w.write_all(s.as_bytes()); }
        self.level += 2;
    }

    fn write_node_end(&mut self, node_type: NodeType) {
        self.level -= 2;
        let s = indentation(&create_close_tag(&convert_node(node_type)), self.level);
        if let Some(w) = &mut self.fs { w.write_all(s.as_bytes()); }
    }

    fn get_current_token(&mut self) -> &Token {
        if !self.is_lookahead {
            self.consume();
        }
        &self.current_token
    }

    fn get_current_token_name(&mut self) -> String {
        convert_token_to_strings(&self.current_token)[1].clone()
    }

    fn is_current_token_identifier(&mut self) -> bool {
        if let Token::Identifier(_) = &self.current_token { true } else { false }
    }

    fn write_identifier_info(&mut self, info: &IdentifierInfo) {
        let l = self.level;
        let s = to_identifier_xml_elem(info, l);
        if let Some(w) = &mut self.fs { w.write_all(s.as_bytes()); }
        self.is_lookahead = false;
    }

    fn write_token_with_consume(&mut self) {
        let l = self.level;
        let s = to_xml_elem(self.get_current_token(), l);
        if let Some(w) = &mut self.fs { w.write_all(s.as_bytes()); }
        self.is_lookahead = false;
    }

    pub fn compileClass(&mut self) {
        self.write_node_start(NodeType::CLASS);

        // first token.
        // TODO:Should be initialized.
        //self.current_token = self.tokenizer.next().unwrap();

        // class
        self.write_token_with_consume();
        
        // className
        self.consume();
        let class_name = self.get_current_token_name();
        let info = IdentifierInfo{
            name: class_name,
            cat: IdentifierCategory::CLASS,
            usage: IdentifierUsage::DEFIEND,
            varKind: None,
            index: None,   
        };
        self.write_identifier_info(&info);

        self.class_name = info.name;

        // {
        self.write_token_with_consume();
        
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
        self.write_token_with_consume();

        self.write_node_end(NodeType::CLASS);
    }

    pub fn compileClassVarDec(&mut self) {
        self.write_node_start(NodeType::CLASS_VAR_DEC);

        // static/field
        let var_kind_name = self.get_current_token_name();
        let mut varKind_t = None;
        if  var_kind_name == "static" {
            varKind_t = Some(VarKind::STATIC);
        }
        else if var_kind_name == "field"  {
            varKind_t = Some(VarKind::FIELD);
        } 
        let varKind = varKind_t.unwrap();
        self.write_token_with_consume();

        // type
        self.consume();
        let type_name = self.get_current_token_name();
        if self.current_token == Token::Keyword(KeywordType::INT)
        || self.current_token == Token::Keyword(KeywordType::CHAR)
        || self.current_token == Token::Keyword(KeywordType::BOOLEAN) {
            self.write_token_with_consume();
        }
        else if self.is_current_token_identifier() {
            let info = IdentifierInfo{
                name: type_name.clone(),
                cat: IdentifierCategory::CLASS,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,   
            };
            self.write_identifier_info(&info);
        }

        if varKind == VarKind::FIELD {
            self.fields_count += 1;
        }
        // varName
        self.consume();
        let var_name = self.get_current_token_name();
        self.table.define(&var_name, &type_name, &varKind);
        let info = IdentifierInfo{
            name: var_name.clone(),
            cat: convert_varKind_to_IdentifierCategory(&varKind),
            usage: IdentifierUsage::DEFIEND,
            varKind: Some(varKind.clone()),
            index: Some(self.table.indexOf(&var_name)),   
        };
        self.write_identifier_info(&info);

        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            // ,
            self.write_token_with_consume();

            if varKind == VarKind::FIELD {
                self.fields_count += 1;
            }
            // varName
            self.consume();
            let var_name = self.get_current_token_name();
            self.table.define(&var_name, &type_name, &varKind);
            let info = IdentifierInfo{
                name: var_name.clone(),
                cat: IdentifierCategory::FIELD,
                usage: IdentifierUsage::DEFIEND,
                varKind: Some(varKind.clone()),
                index: Some(self.table.indexOf(&var_name)),   
            };
            self.write_identifier_info(&info);
        }

        // ;
        self.write_token_with_consume();

        self.write_node_end(NodeType::CLASS_VAR_DEC);
    }

    pub fn compileSubroutineDec(&mut self) {
        self.write_node_start(NodeType::SUBROUTINE_DEC);

        // Clear Subroutine Symbol Table
        self.table.startSubroutine();

        // constructor/function/method
        self.write_token_with_consume();

        self.is_constructor = self.current_token == Token::Keyword(KeywordType::CONSTRUCTOR);
        self.is_method = self.current_token == Token::Keyword(KeywordType::METHOD);

        // void|type
        self.consume();
        if self.current_token == Token::Keyword(KeywordType::VOID)
        || self.current_token == Token::Keyword(KeywordType::INT)
        || self.current_token == Token::Keyword(KeywordType::CHAR)
        || self.current_token == Token::Keyword(KeywordType::BOOLEAN) {
            self.write_token_with_consume();
        }
        else if self.is_current_token_identifier() {
            let class_name = self.get_current_token_name();
            let info = IdentifierInfo{
                name: class_name.clone(),
                cat: IdentifierCategory::CLASS,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,   
            };
            self.write_identifier_info(&info);
        }

        // subroutineName
        self.consume();
        let sr_name = self.get_current_token_name();
        let sr_info = IdentifierInfo{
            name: sr_name.clone(),
            cat: IdentifierCategory::SUBROUTINE,
            usage: IdentifierUsage::DEFIEND,
            varKind: None,
            index: None,   
        };
        self.write_identifier_info(&sr_info);

        self.subroutine_name = sr_info.name.clone();

        // define instance this 
        if self.is_method {
            self.table.define("this", &self.class_name, &VarKind::ARG);
        }

        // (
        self.write_token_with_consume();

        // parameterList
        let args = self.compileParameterList();

        // )
        self.write_token_with_consume();

        // subroutineBody
        self.compileSubroutineBody();

        self.write_node_end(NodeType::SUBROUTINE_DEC);
    }

    pub fn compileParameterList(&mut self) -> i32 {
        self.write_node_start(NodeType::PARAMETER_LIST);

        self.consume();

        let mut args = 0;

        // if not type then empty (should be ")")
        if self.current_token == Token::Symbol(")".to_string()) {
            self.write_node_end(NodeType::PARAMETER_LIST);
            return args;
        }

        args += 1;

        // type
        let type_name = self.get_current_token_name();
        if self.current_token == Token::Keyword(KeywordType::INT)
        || self.current_token == Token::Keyword(KeywordType::CHAR)
        || self.current_token == Token::Keyword(KeywordType::BOOLEAN) {
            self.write_token_with_consume();
        }
        else if self.is_current_token_identifier() {
            let info = IdentifierInfo{
                name: type_name.clone(),
                cat: IdentifierCategory::CLASS,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,   
            };
            self.write_identifier_info(&info);
        }

        //varName
        self.consume();
        let var_name = self.get_current_token_name();
        let varKind = VarKind::ARG;
        self.table.define(&var_name, &type_name, &varKind);
        let info = IdentifierInfo{
            name: var_name.clone(),
            cat: convert_varKind_to_IdentifierCategory(&varKind),
            usage: IdentifierUsage::DEFIEND,
            varKind: Some(varKind.clone()),
            index: Some(self.table.indexOf(&var_name)),   
        };
        self.write_identifier_info(&info);
        
        // , type varName
        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {

            args += 1;

            // ,
            self.write_token_with_consume();

            // type
            self.consume();
            let type_name = self.get_current_token_name();
            if self.current_token == Token::Keyword(KeywordType::INT)
            || self.current_token == Token::Keyword(KeywordType::CHAR)
            || self.current_token == Token::Keyword(KeywordType::BOOLEAN) {
                self.write_token_with_consume();
            }
            else if self.is_current_token_identifier() {
                let info = IdentifierInfo{
                    name: type_name.clone(),
                    cat: IdentifierCategory::CLASS,
                    usage: IdentifierUsage::USED,
                    varKind: None,
                    index: None,   
                };
                self.write_identifier_info(&info);
            }

            //varName
            self.consume();
            let var_name = self.get_current_token_name();
            let varKind = VarKind::ARG;
            self.table.define(&var_name, &type_name, &varKind);
            let info = IdentifierInfo{
                name: var_name.clone(),
                cat: convert_varKind_to_IdentifierCategory(&varKind),
                usage: IdentifierUsage::DEFIEND,
                varKind: Some(varKind.clone()),
                index: Some(self.table.indexOf(&var_name)),   
            };
            self.write_identifier_info(&info);
        }
        self.write_node_end(NodeType::PARAMETER_LIST);
        return args;
    }

    pub fn compileSubroutineBody(&mut self) {
        self.write_node_start(NodeType::SUBROUTINE_BODY);

        // {
        self.write_token_with_consume();

        let mut locals = 0;
        // varDec*
        while {self.consume();
        self.current_token == Token::Keyword(KeywordType::VAR)} {
            locals += self.compileVarDec();
        }

        self.vw.writeFunction(&get_classfunc_name(&self.class_name, &self.subroutine_name), locals);

        // reset if/while label
        self.next_if_label = 0;
        self.next_while_label = 0;

        // insert Memory Alloc if constructor
        // this = Memory.alloc fields_count
        if self.is_constructor {
            self.vw.writePush(Segment::CONST, self.fields_count);
            self.vw.writeCall("Memory.alloc", 1);
            self.vw.writePop(Segment::POINTER, 0);
        }

        // insert instance 
        // this = arg0
        if self.is_method {
            self.vw.writePush(Segment::ARG, 0);
            self.vw.writePop(Segment::POINTER, 0);
        }

        // statements
        self.compileStatementes();

        // }
        self.write_token_with_consume();
        
        self.write_node_end(NodeType::SUBROUTINE_BODY);
    }

    pub fn compileVarDec(&mut self) -> i32 {
        self.write_node_start(NodeType::VAR_DEC);

        let mut vars = 1;
        // var
        let varKind = VarKind::VAR;
        self.write_token_with_consume();

        // type
        self.consume();
        let type_name = self.get_current_token_name();
        if self.is_current_token_identifier() {
            let info = IdentifierInfo{
                name: type_name.clone(),
                cat: IdentifierCategory::CLASS,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,   
            };
            self.write_identifier_info(&info);
        }
        else {
            self.write_token_with_consume();
        }

        // varName
        self.consume();
        let var_name = self.get_current_token_name();
        self.table.define(&var_name, &type_name, &varKind);
        let info = IdentifierInfo{
            name: var_name.clone(),
            cat: convert_varKind_to_IdentifierCategory(&varKind),
            usage: IdentifierUsage::DEFIEND,
            varKind: Some(varKind.clone()),
            index: Some(self.table.indexOf(&var_name)),   
        };
        self.write_identifier_info(&info);

        // (, varName)*
        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            //,
            self.write_token_with_consume();
            
            vars += 1;
            // varName
            self.consume();
            let var_name = self.get_current_token_name();
            self.table.define(&var_name, &type_name, &varKind);
            let info = IdentifierInfo{
                name: var_name.clone(),
                cat: IdentifierCategory::FIELD,
                usage: IdentifierUsage::DEFIEND,
                varKind: Some(varKind.clone()),
                index: Some(self.table.indexOf(&var_name)),   
            };
            self.write_identifier_info(&info);
        }
        
        // ;
        self.write_token_with_consume();

        self.write_node_end(NodeType::VAR_DEC);
        vars
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
        self.write_token_with_consume();

        // varName
        self.consume();
        let var_name = self.get_current_token_name();
        let varKind = self.table.kindOf(&var_name).unwrap();
        let info = IdentifierInfo{
            name: var_name.clone(),
            cat: convert_varKind_to_IdentifierCategory(&varKind),
            usage: IdentifierUsage::USED,
            varKind: Some(varKind.clone()),
            index: Some(self.table.indexOf(&var_name)),   
        };
        self.write_identifier_info(&info);

        // [ or =
        self.consume();
        let mut is_array = false;
        if self.current_token == Token::Symbol("[".to_string()) {
            is_array = true;

            // [
            self.write_token_with_consume();
            
            self.compileExpression();

            // ] 
            self.write_token_with_consume();

            // convert address calculation
            self.vw.writePush(convert_varKind_to_segment(&info.varKind.as_ref().unwrap()), info.index.clone().unwrap());
            self.vw.writeArithmetic(Command::ADD);

            // = 
            self.write_token_with_consume();
        }
        else {
            // = 
            self.write_token_with_consume();        
        }

        self.compileExpression();

        // ;
        self.write_token_with_consume();

        if is_array {
            // save expression result to temp 
            self.vw.writePop(Segment::TEMP, 0);
            // access address
            self.vw.writePop(Segment::POINTER, 1);
            // assign temp
            self.vw.writePush(Segment::TEMP, 0);
            self.vw.writePop(Segment::THAT, 0);
        }
        else {
            // assign
            self.vw.writePop(convert_varKind_to_segment(&info.varKind.unwrap()), info.index.unwrap());
        }
        
        self.write_node_end(NodeType::LET_STATEMENT);
    }

    pub fn compileIf(&mut self) {
        self.write_node_start(NodeType::IF_STATEMENT);
        let if_label = self.next_if_label;
        self.next_if_label += 1;

        // if
        self.write_token_with_consume();

       self.write_token_with_consume();
        
        self.compileExpression();

        // )
        self.write_token_with_consume();

        // vm
        // if-goto IF_TRUEX
        // goto IF_FALSEX
        // label IF_TRUEX
        self.vw.writeIf(&format!("IF_TRUE{}", if_label));
        self.vw.writeGoto(&format!("IF_FALSE{}", if_label)); 
        self.vw.writeLabel(&format!("IF_TRUE{}", if_label)); 

        // {
        self.write_token_with_consume();

        self.compileStatementes();

        // }
        self.write_token_with_consume();

        self.consume();
        if self.current_token == Token::Keyword(KeywordType::ELSE) {

            self.vw.writeGoto(&format!("IF_END{}", if_label));
            self.vw.writeLabel(&format!("IF_FALSE{}", if_label));

            // else
            self.write_token_with_consume();

            // {
            self.write_token_with_consume();

            self.compileStatementes();

            // }
            self.write_token_with_consume();

            self.vw.writeLabel(&format!("IF_END{}", if_label));
        }
        else {
            self.vw.writeLabel(&format!("IF_FALSE{}", if_label)); 
        }

        self.write_node_end(NodeType::IF_STATEMENT);
    }

    pub fn compileWhile(&mut self) {
        self.write_node_start(NodeType::WHILE_STATEMENT);
        let while_label = self.next_while_label;
        self.next_while_label += 1;

        // while
        self.write_token_with_consume();
        
        // (
        self.write_token_with_consume();
        
        self.vw.writeLabel(&format!("WHILE_EXP{}", while_label)); 
        self.compileExpression();
        self.vw.writeArithmetic(Command::NOT);
        self.vw.writeIf(&format!("WHILE_END{}", while_label));
        
        // )
        self.write_token_with_consume();

        // {
        self.write_token_with_consume();

        self.compileStatementes();
        self.vw.writeGoto(&format!("WHILE_EXP{}", while_label)); 

        // }
        self.write_token_with_consume();

        self.vw.writeLabel(&format!("WHILE_END{}", while_label)); 
        self.write_node_end(NodeType::WHILE_STATEMENT);
    }

    pub fn compileDo(&mut self) {
        self.write_node_start(NodeType::DO_STATEMENT);

        // do
        self.write_token_with_consume();

        // identifier
        self.consume();
        let name = self.get_current_token_name();
        self.is_lookahead = false;

        self.consume();
        if self.current_token == Token::Symbol("(".to_string()) {
            let info = IdentifierInfo {
                name: name.clone(),
                cat: IdentifierCategory::SUBROUTINE,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,
            };
            self.write_identifier_info(&info);
            // lookahead is not processed, turn on flag
            self.is_lookahead = true;

            // call of class member function, so add argument this.
            self.vw.writePush(Segment::POINTER, 0);

            // (
            self.write_token_with_consume();

            let args = self.compileExpressionList();

            // )
            self.write_token_with_consume();

            // argument conatins this.
            self.vw.writeCall(&get_classfunc_name(&self.class_name, &info.name), args + 1);
        } 
        else if self.current_token == Token::Symbol(".".to_string()) {
            let vk = self.table.kindOf(&name);
            let info = IdentifierInfo{
                name: name.clone(),
                cat: if vk.is_none() { IdentifierCategory::CLASS } else { convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()) },
                usage: IdentifierUsage::USED,
                varKind: vk.clone(),
                index: if vk.is_none() { None } else { Some(self.table.indexOf(&name)) },   
            };
            self.write_identifier_info(&info);
            // lookahead is not processed, turn on flag
            self.is_lookahead = true;

            // .
            self.write_token_with_consume();

            // subroutineName
            self.consume();
            let sr_name = self.get_current_token_name();
            let sr_info = IdentifierInfo {
                name: sr_name,
                cat: IdentifierCategory::SUBROUTINE,
                usage: IdentifierUsage::USED,
                varKind: None,
                index: None,
            };
            self.write_identifier_info(&sr_info);

            // (
            self.write_token_with_consume();

            if info.cat != IdentifierCategory::CLASS {
                // instance call
                self.vw.writePush(convert_varKind_to_segment(&info.varKind.unwrap()), info.index.unwrap());
            }
            let args = self.compileExpressionList();

            // )
            self.write_token_with_consume();

            if info.cat == IdentifierCategory::CLASS {
                self.vw.writeCall(&get_classfunc_name(&info.name, &sr_info.name), args);
            }
            else {
                // args must contain instance
                self.vw.writeCall(&get_classfunc_name(&self.table.typeOf(&info.name), &sr_info.name), args + 1);    
            }
        } 

        // ;
        self.write_token_with_consume();

        // pop 0 for void function.
        self.vw.writePop(Segment::TEMP, 0);
       
        self.write_node_end(NodeType::DO_STATEMENT);
    }

    pub fn compileReturn(&mut self) {
        self.write_node_start(NodeType::RETURN_STATEMENT);
        
        // return
        self.write_token_with_consume();

        self.consume();
        if self.current_token != Token::Symbol(";".to_string())
        {
            self.compileExpression();
        }
        else {
            // void return
            self.vw.writePush(Segment::CONST, 0);
        }

        // ;
        self.write_token_with_consume();
        
        self.write_node_end(NodeType::RETURN_STATEMENT);

        self.vw.writeReturn();
    }

    pub fn compileExpressionList(&mut self) -> i32 {
        self.write_node_start(NodeType::EXPRESSION_LIST);

        let mut args = 0;

        self.consume();
        if self.current_token == Token::Symbol(")".to_string()) {
            // Empty
            self.write_node_end(NodeType::EXPRESSION_LIST);
            return args;
        }

        args += 1;
        self.compileExpression();

        while {self.consume();
        self.current_token == Token::Symbol(",".to_string())} {
            // ,
            self.write_token_with_consume();

            args += 1;
            self.compileExpression();
        }

        self.write_node_end(NodeType::EXPRESSION_LIST);
        return args;
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
            let op_token = self.current_token.clone();
            // op
            self.write_token_with_consume();

            self.compileTerm();

            self.write_arithmetic(&op_token);
        }

        self.write_node_end(NodeType::EXPRESSION);
    }

    pub fn compileTerm(&mut self) {
        self.write_node_start(NodeType::TERM);

        self.consume();

        if let Token::IntConst(i) = &self.current_token {
            self.vw.writePush(Segment::CONST, *i);
            // integerConst
            self.write_token_with_consume();
        }
        else if let Token::StringConst(s) = &self.current_token {
            // StringConst
            
            // String.new(length)
            self.vw.writePush(Segment::CONST, s.len() as i32);
            self.vw.writeCall("String.new", 1);

            // String.appendChar(asciicode)
            for c in s.chars() {
                self.vw.writePush(Segment::CONST, c as i32);
                self.vw.writeCall("String.appendChar", 2);
            }

            self.write_token_with_consume();
        }
        else if let Token::Keyword(kw) = &self.current_token {
            // KeywordConst
            match kw {
                KeywordType::TRUE => {
                    self.vw.writePush(Segment::CONST, 0);
                    self.vw.writeArithmetic(Command::NOT);
                },
                KeywordType::FALSE | KeywordType::NULL => { self.vw.writePush(Segment::CONST, 0); },
                KeywordType::THIS => { self.vw.writePush(Segment::POINTER, 0); },
                _ => {},
            }
            self.write_token_with_consume();
        } 
        else if self.current_token == Token::Symbol("(".to_string()) {
            // (
            self.write_token_with_consume();

            // expression
            self.compileExpression();

            // )
            self.write_token_with_consume();
        }        
        else if self.current_token == Token::Symbol("-".to_string())
        || self.current_token == Token::Symbol("~".to_string()) {
            let op_token = self.current_token.clone();
            // unaryOp
            self.write_token_with_consume();

            self.compileTerm();

            self.write_unary_arithmetic(&op_token);
        }
        else {
            // identifier
            let name = self.get_current_token_name();
            self.is_lookahead = false;

            self.consume();
            let elem = convert_token_to_strings(&self.current_token);             
            if self.current_token == Token::Symbol(".".to_string()) {
                let vk = self.table.kindOf(&name);
                let info = IdentifierInfo{
                    name: name.clone(),
                    cat: if vk.is_none() { IdentifierCategory::CLASS } else { convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()) },
                    usage: IdentifierUsage::USED,
                    varKind: vk.clone(),
                    index: if vk.is_none() { None } else { Some(self.table.indexOf(&name)) },   
                };
                self.write_identifier_info(&info);
                // lookahead is not processed, turn on flag
                self.is_lookahead = true;

                // .
                self.write_token_with_consume();

                // subroutineName
                self.consume();
                let sr_name = self.get_current_token_name();
                let sr_info = IdentifierInfo {
                    name: sr_name,
                    cat: IdentifierCategory::SUBROUTINE,
                    usage: IdentifierUsage::USED,
                    varKind: None,
                    index: None,
                };
                self.write_identifier_info(&sr_info);

                // (
                self.write_token_with_consume();

                if info.cat != IdentifierCategory::CLASS {
                    // instance call
                    self.vw.writePush(convert_varKind_to_segment(&info.varKind.unwrap()), info.index.unwrap());
                }

                let args = self.compileExpressionList();

                // )
                self.write_token_with_consume();

                if info.cat == IdentifierCategory::CLASS {
                    self.vw.writeCall(&get_classfunc_name(&info.name, &sr_info.name), args);
                }
                else {
                    // args must contain instance
                    self.vw.writeCall(&get_classfunc_name(&self.table.typeOf(&info.name), &sr_info.name), args + 1);    
                }
            } 
            else if self.current_token == Token::Symbol("(".to_string()) {
                let info = IdentifierInfo {
                    name: name,
                    cat: IdentifierCategory::SUBROUTINE,
                    usage: IdentifierUsage::USED,
                    varKind: None,
                    index: None,
                };
                self.write_identifier_info(&info);
                // lookahead is not processed, turn on flag
                self.is_lookahead = true;

                // (
                self.write_token_with_consume();
                
                if self.is_method {
                    // call of class member function, so add argument this.
                    self.vw.writePush(Segment::POINTER, 0);
                }

                let args = self.compileExpressionList();

                // )
                self.write_token_with_consume();

                // argument conatins this.
                self.vw.writeCall(&get_classfunc_name(&self.class_name, &info.name), if self.is_method { args + 1 } else { args });
            } 
            else if self.current_token == Token::Symbol("[".to_string()) {
                let vk = self.table.kindOf(&name);
                let info = IdentifierInfo{
                    name: name.clone(),
                    cat: convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()),
                    usage: IdentifierUsage::USED,
                    varKind: vk,
                    index: Some(self.table.indexOf(&name)),   
                };
                self.write_identifier_info(&info);
                // lookahead is not processed, turn on flag
                self.is_lookahead = true;

                // [
                self.write_token_with_consume();

                self.compileExpression();
    
                // ]

                // convert address calculation
                self.vw.writePush(convert_varKind_to_segment(&info.varKind.unwrap()), info.index.unwrap());
                self.vw.writeArithmetic(Command::ADD);
                // access address
                self.vw.writePop(Segment::POINTER, 1);
                self.vw.writePush(Segment::THAT, 0);

                self.write_token_with_consume();
            }
            else {
                // varName
                let vk = self.table.kindOf(&name);
                let info = IdentifierInfo{
                    name: name.clone(),
                    cat: convert_varKind_to_IdentifierCategory(&vk.clone().unwrap()),
                    usage: IdentifierUsage::USED,
                    varKind: vk,
                    index: Some(self.table.indexOf(&name)),   
                };
                self.write_identifier_info(&info);

                self.vw.writePush(convert_varKind_to_segment(&info.varKind.unwrap()), info.index.unwrap());

                // lookahead is not processed, turn on flag
                self.is_lookahead = true;
            }
        }

        self.write_node_end(NodeType::TERM);
    }

    fn write_arithmetic(&mut self, tk: &Token) {
        match tk {
            Token::Symbol(s) => match s.as_str() {
                "+" => { self.vw.writeArithmetic(Command::ADD); },
                "-" => { self.vw.writeArithmetic(Command::SUB); },
                "*" => { self.vw.writeCall("Math.multiply", 2); },
                "/" => { self.vw.writeCall("Math.divide", 2); },
                "&" => { self.vw.writeArithmetic(Command::AND); },
                "|" => { self.vw.writeArithmetic(Command::OR); },
                "<" => { self.vw.writeArithmetic(Command::LT); },
                ">" => { self.vw.writeArithmetic(Command::GT); },
                "=" => { self.vw.writeArithmetic(Command::EQ); },
                _ => {},
            }
            _ => {},
        }
    }

    fn write_unary_arithmetic(&mut self, tk: &Token) {
        match tk {
            Token::Symbol(s) => match s.as_str() {
                "~" => { self.vw.writeArithmetic(Command::NOT); },
                "-" => { self.vw.writeArithmetic(Command::NEG); },
                _ => {},
            }
            _ => {},
        }
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
fn convert_varKind_to_segment(varKind: &VarKind) -> Segment {
    match varKind {
        VarKind::VAR => Segment::LOCAL,
        VarKind::ARG => Segment::ARG,
        VarKind::STATIC => Segment::STATIC,
        VarKind::FIELD => Segment::THIS,
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

fn get_classfunc_name(class_name: &str, func_name: &str) -> String {
    [class_name, func_name].join(".")
}

#[cfg(test)]
mod tests{
    use super::*;
    
    fn rawstr_to_code(s: &str) -> String {
        let mut sw = s.to_string();
        sw.remove(0);
        sw.replace("\n", "\r\n")
    }

    #[test]
    fn SimplestClass() {
        let s = io::Cursor::new("\
        class Main {}
        ");
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w.clone(), Some(w));

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
        assert_eq!(String::from_utf8(c.fs.unwrap().buffer().to_vec()).unwrap(), r);
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
        let mut c = CompilationEngine::new(s, w.clone(), Some(w));

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
        assert_eq!(String::from_utf8(c.fs.unwrap().buffer().to_vec()).unwrap(), r);
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
        let mut c = CompilationEngine::new(s, w.clone(), Some(w));

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
        assert_eq!(String::from_utf8(c.fs.unwrap().buffer().to_vec()).unwrap(), r);
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
        let mut c = CompilationEngine::new(s, w.clone(), Some(w));

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
        assert_eq!(String::from_utf8(c.fs.unwrap().buffer().to_vec()).unwrap(), r);
    }


    #[test]
    fn Subroutine_field_test() {
        let s = io::Cursor::new("\
        class Square {\r\n\
            static boolean test;\r\n\
            field int x, y; // screen location of the square's top-left corner\r\n\
            field int size; // length of this square, in pixels\r\n\
            /** Constructs a new square with a given location and size. */\r\n\
            constructor Square new(int Ax, int Ay, int Asize) {\r\n\
               let x = Ax;\r\n\
               let y = Ay;\r\n\
               let size = Asize;\r\n\
               do draw();\r\n\
               return this;\r\n\
            }\r\n\
        }\r\n\
        ");
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w.clone(), Some(w));

        c.compileClass();
        let mut r = r#"
<class>
  <keyword> class </keyword>
  <identifier> Square, class, defined </identifier>
  <symbol> { </symbol>
  <classVarDec>
    <keyword> static </keyword>
    <keyword> boolean </keyword>
    <identifier> test, static, defined, static, 0 </identifier>
    <symbol> ; </symbol>
  </classVarDec>
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
  <subroutineDec>
    <keyword> constructor </keyword>
    <identifier> Square, class, used </identifier>
    <identifier> new, subroutine, defined </identifier>
    <symbol> ( </symbol>
    <parameterList>
      <keyword> int </keyword>
      <identifier> Ax, argument, defined, argument, 0 </identifier>
      <symbol> , </symbol>
      <keyword> int </keyword>
      <identifier> Ay, argument, defined, argument, 1 </identifier>
      <symbol> , </symbol>
      <keyword> int </keyword>
      <identifier> Asize, argument, defined, argument, 2 </identifier>
    </parameterList>
    <symbol> ) </symbol>
    <subroutineBody>
      <symbol> { </symbol>
      <statements>
        <letStatement>
          <keyword> let </keyword>
          <identifier> x, field, used, field, 0 </identifier>
          <symbol> = </symbol>
          <expression>
            <term>
              <identifier> Ax, argument, used, argument, 0 </identifier>
            </term>
          </expression>
          <symbol> ; </symbol>
        </letStatement>
        <letStatement>
          <keyword> let </keyword>
          <identifier> y, field, used, field, 1 </identifier>
          <symbol> = </symbol>
          <expression>
            <term>
              <identifier> Ay, argument, used, argument, 1 </identifier>
            </term>
          </expression>
          <symbol> ; </symbol>
        </letStatement>
        <letStatement>
          <keyword> let </keyword>
          <identifier> size, field, used, field, 2 </identifier>
          <symbol> = </symbol>
          <expression>
            <term>
              <identifier> Asize, argument, used, argument, 2 </identifier>
            </term>
          </expression>
          <symbol> ; </symbol>
        </letStatement>
        <doStatement>
          <keyword> do </keyword>
          <identifier> draw, subroutine, used </identifier>
          <symbol> ( </symbol>
          <expressionList>
          </expressionList>
          <symbol> ) </symbol>
          <symbol> ; </symbol>
        </doStatement>
        <returnStatement>
          <keyword> return </keyword>
          <expression>
            <term>
              <keyword> this </keyword>
            </term>
          </expression>
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
        assert_eq!(String::from_utf8(c.fs.unwrap().buffer().to_vec()).unwrap(), r);
    }


    #[test]
    fn CodeGenaration_Seven() {
        let s = io::Cursor::new(rawstr_to_code(r#"
class Main {
    
    function void main() {
        do Output.printInt(1 + (2 * 3));
        return;
    }
    
}
"#));
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w, None);
        c.xml_mode = false;
        c.compileClass();

        let r = rawstr_to_code(r#"
function Main.main 0
push constant 1
push constant 2
push constant 3
call Math.multiply 2
add
call Output.printInt 1
pop temp 0
push constant 0
return
"#);
        assert_eq!(c.vw.dump_string(), r);
    }

    #[test]
    fn CodeGenaration_If() {
        let s = io::Cursor::new(rawstr_to_code(r#"
class Main {
    
    function int main(int mask) {
        if (mask = 0) {
            return 1;
        } 
        else {
            return mask * 2;
        }
    }
    
}
"#));
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w, None);
        c.xml_mode = false;
        c.compileClass();

        let r = rawstr_to_code(r#"
function Main.main 0
push argument 0
push constant 0
eq
if-goto IF_TRUE0
goto IF_FALSE0
label IF_TRUE0
push constant 1
return
goto IF_END0
label IF_FALSE0
push argument 0
push constant 2
call Math.multiply 2
return
label IF_END0
"#);
        assert_eq!(c.vw.dump_string(), r);
    }

    #[test]
    fn CodeGenaration_Multi_If() {
        let s = io::Cursor::new(rawstr_to_code(r#"
class Main {
    
    function void main(int mask) {
        if (mask < 1) {
            if (mask = 0) {
                return;
            } 
            else {
                return;
            }
        }

        if (mask > 0) {
        }
        return;
    }
    
}
"#));
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w, None);
        c.xml_mode = false;
        c.compileClass();

        let r = rawstr_to_code(r#"
function Main.main 0
push argument 0
push constant 1
lt
if-goto IF_TRUE0
goto IF_FALSE0
label IF_TRUE0
push argument 0
push constant 0
eq
if-goto IF_TRUE1
goto IF_FALSE1
label IF_TRUE1
push constant 0
return
goto IF_END1
label IF_FALSE1
push constant 0
return
label IF_END1
label IF_FALSE0
push argument 0
push constant 0
gt
if-goto IF_TRUE2
goto IF_FALSE2
label IF_TRUE2
label IF_FALSE2
push constant 0
return
"#);
        assert_eq!(c.vw.dump_string(), r);
    }

    #[test]
    fn CodeGenaration_While() {
        let s = io::Cursor::new(rawstr_to_code(r#"
class Main {
    
    function void main(int startAddress, int length, int value) {
        while (length > 0) {
            do Memory.poke(startAddress, value);
            let length = length - 1;
            let startAddress = startAddress + 1;
        }
        return;
    }
    
}
"#));
        let w = io::Cursor::new(Vec::new());
        let mut c = CompilationEngine::new(s, w, None);
        c.xml_mode = false;
        c.compileClass();

        let r = rawstr_to_code(r#"
function Main.main 0
label WHILE_EXP0
push argument 1
push constant 0
gt
not
if-goto WHILE_END0
push argument 0
push argument 2
call Memory.poke 2
pop temp 0
push argument 1
push constant 1
sub
pop argument 1
push argument 0
push constant 1
add
pop argument 0
goto WHILE_EXP0
label WHILE_END0
push constant 0
return
"#);
        assert_eq!(c.vw.dump_string(), r);
    }

    #[test]
    fn compile_Seven() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Seven/Main.jack");
            let w_xml = std::fs::File::create("Seven/Main_compile.xml");
            let w = std::fs::File::create("Seven/Main_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Seven/Main.vm").unwrap();
        let al = std::fs::read_to_string("Seven/Main_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_ConvertToBin() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("ConvertToBin/Main.jack");
            let w_xml = std::fs::File::create("ConvertToBin/Main_compile.xml");
            let w = std::fs::File::create("ConvertToBin/Main_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("ConvertToBin/Main.vm").unwrap();
        let al = std::fs::read_to_string("ConvertToBin/Main_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }


    #[test]
    fn compile_Square_Main() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Square/Main.jack");
            let w_xml = std::fs::File::create("Square/Main_compile.xml");
            let w = std::fs::File::create("Square/Main_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Square/Main.vm").unwrap();
        let al = std::fs::read_to_string("Square/Main_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Square_Square1() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Square/Square.jack");
            let w_xml = std::fs::File::create("Square/Square_compile.xml");
            let w = std::fs::File::create("Square/Square_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Square/Square.vm").unwrap();
        let al = std::fs::read_to_string("Square/Square_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Square_SquareGame() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Square/SquareGame.jack");
            let w_xml = std::fs::File::create("Square/SquareGame_compile.xml");
            let w = std::fs::File::create("Square/SquareGame_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Square/SquareGame.vm").unwrap();
        let al = std::fs::read_to_string("Square/SquareGame_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Average() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Average/Main.jack");
            let w_xml = std::fs::File::create("Average/Main_compile.xml");
            let w = std::fs::File::create("Average/Main_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Average/Main.vm").unwrap();
        let al = std::fs::read_to_string("Average/Main_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Pong_Main() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Pong/Main.jack");
            let w_xml = std::fs::File::create("Pong/Main_compile.xml");
            let w = std::fs::File::create("Pong/Main_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Pong/Main.vm").unwrap();
        let al = std::fs::read_to_string("Pong/Main_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Pong_Ball() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Pong/Ball.jack");
            let w_xml = std::fs::File::create("Pong/Ball_compile.xml");
            let w = std::fs::File::create("Pong/Ball_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Pong/Ball.vm").unwrap();
        let al = std::fs::read_to_string("Pong/Ball_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Pong_Bat() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Pong/Bat.jack");
            let w_xml = std::fs::File::create("Pong/Bat_compile.xml");
            let w = std::fs::File::create("Pong/Bat_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Pong/Bat.vm").unwrap();
        let al = std::fs::read_to_string("Pong/Bat_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_Pong_PongGame() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("Pong/PongGame.jack");
            let w_xml = std::fs::File::create("Pong/PongGame_compile.xml");
            let w = std::fs::File::create("Pong/PongGame_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("Pong/PongGame.vm").unwrap();
        let al = std::fs::read_to_string("Pong/PongGame_compile.vm").unwrap();
        assert_eq!(result_string, al);
    }

    #[test]
    fn compile_ComplexArrays() {
        // CompilationEngine must be scoped to drop.
        // If not, BufWriter will not be flushed. 
        {
            let s = std::fs::File::open("ComplexArrays/Main.jack");
            let w_xml = std::fs::File::create("ComplexArrays/Main_compile.xml");
            let w = std::fs::File::create("ComplexArrays/Main_compile.vm");

            let mut c = CompilationEngine::new(s.unwrap(), w.unwrap(), Some(w_xml.unwrap()));
            c.compileClass();
        }

        let result_string = std::fs::read_to_string("ComplexArrays/Main.vm").unwrap();
        let al = std::fs::read_to_string("ComplexArrays/Main.vm").unwrap();
        assert_eq!(result_string, al);
    }
}