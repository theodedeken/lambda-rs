use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolTable<T> {
    table: Vec<Scope<T>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scope<T> {
    map: HashMap<String, T>,
}

impl<T> SymbolTable<T> {
    pub fn new() -> SymbolTable<T> {
        SymbolTable { table: Vec::new() }
    }

    pub fn lookup(&self, name: &str) -> Option<&T> {
        for scope in self.table.iter().rev() {
            if let Some(data_type) = scope.get(name) {
                return Some(data_type);
            }
        }
        None
    }

    pub fn push(&mut self, scope: Scope<T>) {
        self.table.push(scope);
    }

    pub fn remove(&mut self, name: &str) {
        let to_remove = self.table.iter().position(|el| el.get(name).is_some());
        if let Some(to_remove) = to_remove {
            self.table.remove(to_remove);
        }
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
