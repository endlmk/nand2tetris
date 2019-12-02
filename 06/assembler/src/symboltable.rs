use std::collections::HashMap;

pub struct SymbolTable {
    table : HashMap<String, i32>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { table: HashMap::new() }
    }
    pub fn addEntry(&mut self, symbol: String, address: i32) {
        self.table.insert(symbol, address);
    }
    pub fn contains(&self, symbol: &str) -> bool {
        self.table.contains_key(symbol)
    }
    pub fn getAddress(&self, symbol: &str) -> i32 {
        *self.table.get(symbol).unwrap_or(&0)
    }
}