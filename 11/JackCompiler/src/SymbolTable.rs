use std::collections::HashMap;

struct SymbolTable {
    class_symbols: HashMap<String, SymbolInfo>,
    subroutine_symbols: HashMap<String, SymbolInfo>,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
enum Scope {
    CLASS,
    SUBROUTINE,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
enum VarKind {
    STATIC,
    FIELD,
    ARG,
    VAR,
}

struct SymbolInfo {
    type_name: String,
    index: i32,
    kind: VarKind,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            class_symbols: HashMap::new(),
            subroutine_symbols: HashMap::new(),
        }
    }

    pub fn startSubroutine(&mut self) {
        self.subroutine_symbols.clear();
    }

    pub fn varCount(&self, kind: &VarKind) -> i32 {
        let t = match kind {
            VarKind::STATIC | VarKind::FIELD => &self.class_symbols,
            VarKind::ARG | VarKind::VAR => &self.subroutine_symbols,
        };
        t.iter().fold(0, |acc, (_, v)| if v.kind == *kind { acc + 1 } else { acc })   
    }

    pub fn define(&mut self, name: &str, type_name: &str, kind: &VarKind) {
        let i = self.varCount(kind);
        let table = match kind {
            VarKind::STATIC | VarKind::FIELD => &mut self.class_symbols,
            VarKind::ARG | VarKind::VAR => &mut self.subroutine_symbols,
        };
        
        table.insert(name.to_string(), SymbolInfo {type_name: type_name.to_string(), index: i, kind: kind.clone()});
    }

    pub fn kindOf(&self, name: &str) -> Option<VarKind> {
        match self.subroutine_symbols.get(name) {
            Some(item) => Some(item.kind.clone()),
            None => {
                match self.class_symbols.get(name) {
                    Some(item) => Some(item.kind.clone()),
                    None => None,
                }
            }
        }
    }

    pub fn typeOf(&self, name: &str) -> String {
        match self.subroutine_symbols.get(name) {
            Some(item) => item.type_name.clone(),
            None => {
                match self.class_symbols.get(name) {
                    Some(item) => item.type_name.clone(),
                    None => String::new(),
                }
            }
        }
    }

    pub fn indexOf(&self, name: &str) -> i32 {
        match self.subroutine_symbols.get(name) {
            Some(item) => item.index,
            None => {
                match self.class_symbols.get(name) {
                    Some(item) => item.index,
                    None => 0,
                }
            }
        }
    }

}