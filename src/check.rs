use ast::*;
use std::collections::HashMap;
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

impl ASTNode {
    pub fn check(&self) -> Result<TypeAssignment, Box<Error>> {
        self.check_node(&mut SymbolTable::new())
    }

    fn check_node(
        &self,
        table: &mut SymbolTable<TypeAssignment>,
    ) -> Result<TypeAssignment, Box<Error>> {
        match self {
            ASTNode::ValueNode { value } => match value {
                Value::True => Ok(TypeAssignment::Single(Type::Bool)),
                Value::False => Ok(TypeAssignment::Single(Type::Bool)),
                Value::Zero => Ok(TypeAssignment::Single(Type::Nat)),
            },
            ASTNode::IsZeroNode { expr } => match expr.check_node(table)? {
                TypeAssignment::Single(Type::Nat) => Ok(TypeAssignment::Single(Type::Bool)),
                _ => Err(Box::new(TypeError::new(format!(
                    "Found a zero check with inner expression not of type Nat"
                )))),
            },
            ASTNode::IdentifierNode { name } => {
                if let Some(type_ass) = table.lookup(name) {
                    Ok(type_ass.clone())
                } else {
                    Err(Box::new(TypeError::new(format!(
                        "Unknown identifier found: {}",
                        name
                    ))))
                }
            }
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
                table.push(Scope::new(ident.to_string(), data_type.clone()));
                let body_type = body.check_node(table)?;
                Ok(TypeAssignment::Arrow(
                    Box::new(data_type.clone()),
                    Box::new(body_type),
                ))
            }
            ASTNode::ProjectionNode { target, attrib } => {
                if let TypeAssignment::Record(types) = target.check_node(table)? {
                    if let Some(attrib_type) = types.get(attrib) {
                        Ok(attrib_type.clone())
                    } else {
                        Err(Box::new(TypeError::new(format!(
                            "Projection attribute is not defined"
                        ))))
                    }
                } else {
                    Err(Box::new(TypeError::new(format!(
                        "Found a projection where the target argument was not of type record"
                    ))))
                }
            }
            ASTNode::RecordNode { records } => {
                let mut types: HashMap<String, TypeAssignment> = HashMap::new();
                for (name, node) in records {
                    types.insert(name.to_string(), node.check_node(table)?);
                }
                Ok(TypeAssignment::Record(types))
            }
            ASTNode::MatchingNode { to_match, cases } => {
                if let TypeAssignment::Variant(variant) = to_match.check_node(table)? {
                    let mut arm_type = None;
                    for (variant_name, variant_type) in variant {
                        if let Some((ident, arm)) = cases.get(&variant_name) {
                            table.push(Scope::new(ident.to_string(), variant_type.clone()));

                            if arm_type == None {
                                arm_type = Some(arm.check_node(table)?);
                            } else {
                                if arm_type != Some(arm.check_node(table)?) {
                                    return Err(Box::new(TypeError::new(
                                        "Not all case arms return the same type".to_string(),
                                    )));
                                }
                            }
                        } else {
                            return Err(Box::new(TypeError::new(
                                "Not all cases are being handled".to_string(),
                            )));
                        }
                    }
                    arm_type.ok_or(Box::new(TypeError::new("Empty variant".to_string())))
                } else {
                    Err(Box::new(TypeError::new(
                        "Found a match case where the argument to be matched was not a variant"
                            .to_string(),
                    )))
                }
            }
            ASTNode::TaggingNode {
                ident,
                value,
                data_type,
            } => {
                let value_type = value.check_node(table)?;
                if data_type.has_variant(ident, value_type) {
                    Ok(data_type.clone())
                } else {
                    Err(Box::new(TypeError::new(
                        "Found a tag of a value, but its type is not part of the variant"
                            .to_string(),
                    )))
                }
            }
            ASTNode::FixNode { point } => panic!("not implemented"),
        }
    }
}
