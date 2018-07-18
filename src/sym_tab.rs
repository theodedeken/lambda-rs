use ast::Type;
use check::TypeAssignment;

pub struct SymbolTable {}

pub struct Scope {
    name: String,
    data_type: Type,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {}
    }

    pub fn lookup(&self, name: String) -> TypeAssignment {
        //TODO implement
        TypeAssignment::Single(Type::Bool)
    }

    pub fn add(&self, scope: Scope) {}
}

impl Scope {
    pub fn new(name: String, data_type: Type) -> Scope {
        Scope { name, data_type }
    }
}
