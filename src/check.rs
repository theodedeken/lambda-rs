use ast::*;
use std::error::Error;
use std::fmt;
use sym_tab::*;

#[derive(Debug)]
pub struct TypeError {
    message: String,
}

impl TypeError {
    pub fn new(message: String) -> TypeError {
        TypeError { message }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TypeError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(PartialEq)]
pub enum TypeAssignment {
    Single(Type),
    Arrow(Box<TypeAssignment>, Box<TypeAssignment>),
}
//TODO if let more
impl ASTNode {
    pub fn check(&self) -> Result<TypeAssignment, Box<Error>> {
        self.check_node(&SymbolTable::new())
    }

    fn check_node(&self, table: &SymbolTable) -> Result<TypeAssignment, Box<Error>> {
        match self {
            ASTNode::ValueNode { value } => match value {
                Value::True => Ok(TypeAssignment::Single(Type::Bool)),
                Value::False => Ok(TypeAssignment::Single(Type::Bool)),
                Value::Zero => Ok(TypeAssignment::Single(Type::Nat)),
            },
            ASTNode::IsZeroNode { expr } => match expr.check_node(table)? {
                TypeAssignment::Single(Type::Bool) => Ok(TypeAssignment::Single(Type::Bool)),
                _ => Err(Box::new(TypeError::new(format!(
                    "Found a zero check with inner expression not of type Nat"
                )))),
            },
            ASTNode::IdentifierNode { name } => Ok(table.lookup(name.to_string())),
            ASTNode::ConditionNode {
                clause,
                then_arm,
                else_arm,
            } => {
                let clause_type = clause.check_node(table)?;
                if clause_type != TypeAssignment::Single(Type::Bool) {
                    return Err(Box::new(TypeError::new(format!(
                        "Found a ifthenelse with clause of invalid type"
                    ))));
                }
                let then_type = then_arm.check_node(table)?;
                let else_type = else_arm.check_node(table)?;
                if then_type != else_type {
                    return Err(Box::new(TypeError::new(format!(
                        "Found a ifthenelse with arms of different type"
                    ))));
                }
                Ok(then_type)
            }
            ASTNode::ArithmeticNode { op: _, expr } => match expr.check_node(table)? {
                TypeAssignment::Single(Type::Nat) => Ok(TypeAssignment::Single(Type::Nat)),
                _ => Err(Box::new(TypeError::new(format!(
                    "Found a zero check with inner expression of invalid type"
                )))),
            },
            ASTNode::ApplicationNode { left, right } => {
                let left_type = left.check_node(table)?;
                let right_type = right.check_node(table)?;
                if let TypeAssignment::Arrow(first, second) = left_type {
                    if *first == right_type {
                        Ok(*second)
                    } else {
                        Err(Box::new(TypeError::new(format!(
                        "Found an application where first argument of function type does not equal the type of the right argument"
                    ))))
                    }
                } else {
                    Err(Box::new(TypeError::new(format!(
                        "Found an application where left argument is not a function type"
                    ))))
                }
            }
            ASTNode::AbstractionNode {
                ident,
                data_type,
                body,
            } => {
                table.add(Scope::new(ident.to_string(), data_type.clone()));
                let body_type = body.check_node(table)?;
                Ok(TypeAssignment::Arrow(
                    Box::new(TypeAssignment::Single(data_type.clone())),
                    Box::new(body_type),
                ))
            }
        }
    }
}
