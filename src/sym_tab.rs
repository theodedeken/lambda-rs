use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SymbolTable<T> {
    table: Vec<Scope<T>>,
}

#[derive(Clone, Debug)]
pub struct Scope<T> {
    map: HashMap<String, T>,
}

impl<T> SymbolTable<T> {
    pub fn new() -> SymbolTable<T> {
        SymbolTable { table: Vec::new() }
    }

    pub fn lookup(&self, name: &str) -> Option<&T> {
        for scope in &self.table {
            if let Some(data_type) = scope.get(name) {
                return Some(data_type);
            }
        }
        None
    }

    pub fn push(&mut self, scope: Scope<T>) {
        self.table.push(scope);
    }
}

impl<T> Scope<T> {
    pub fn new(name: String, contents: T) -> Scope<T> {
        let mut map = HashMap::new();
        map.insert(name, contents);
        Scope { map }
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.map.get(name)
    }
}
