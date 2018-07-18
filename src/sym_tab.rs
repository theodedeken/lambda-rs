use ast::Type;
use check::TypeAssignment;
use std::collections::HashMap;

pub struct SymbolTable {
    table: Vec<Scope>,
}

pub struct Scope {
    map: HashMap<String, Type>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { table: Vec::new() }
    }

    pub fn lookup(&self, name: &str) -> Option<TypeAssignment> {
        for scope in &self.table {
            if let Some(data_type) = scope.get(name) {
                return Some(TypeAssignment::Single(data_type.clone()));
            }
        }
        None
    }

    pub fn push(&mut self, scope: Scope) {
        self.table.push(scope);
    }
}

impl Scope {
    pub fn new(name: String, data_type: Type) -> Scope {
        let mut map = HashMap::new();
        map.insert(name, data_type);
        Scope { map }
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.map.get(name)
    }
}
