use ast::*;
use pest::Error;
use std::collections::HashMap;
use sym_tab::*;

impl<'a> ASTNode<'a> {
    /// Performs typechecking on the abstract syntax tree and returns the resulting type or an error
    /// specifying the problem encountered when type checking
    pub fn check<R: Copy>(&self) -> Result<TypeAssignment, Error<R>> {
        self.check_node(&mut SymbolTable::new())
    }

    fn check_node<R: Copy>(
        &self,
        table: &mut SymbolTable<TypeAssignment>,
    ) -> Result<TypeAssignment, Error<R>> {
        match self {
            ASTNode::ValueNode { meta: _, value } => match value {
                Value::True => Ok(TypeAssignment::Single(Type::Bool)),
                Value::False => Ok(TypeAssignment::Single(Type::Bool)),
                Value::Zero => Ok(TypeAssignment::Single(Type::Nat)),
            },
            ASTNode::IsZeroNode { meta, expr } => match expr.check_node(table)? {
                TypeAssignment::Single(Type::Nat) => Ok(TypeAssignment::Single(Type::Bool)),
                _ => Err(Error::CustomErrorSpan {
                    message: "The argument of a zero check should be of type Nat".to_string(),
                    span: meta.clone(),
                }),
            },
            ASTNode::IdentifierNode { meta, name } => {
                if let Some(type_ass) = table.lookup(name) {
                    Ok(type_ass.clone())
                } else {
                    Err(Error::CustomErrorSpan {
                        message: "Identifier is not defined".to_string(),
                        span: meta.clone(),
                    })
                }
            }
            ASTNode::ConditionNode {
                meta,
                clause,
                then_arm,
                else_arm,
            } => {
                let clause_type = clause.check_node(table)?;
                if clause_type != TypeAssignment::Single(Type::Bool) {
                    return Err(Error::CustomErrorSpan {
                        message: "The clause of an if expression should be of type Bool"
                            .to_string(),
                        span: meta.clone(),
                    });
                }
                let then_type = then_arm.check_node(table)?;
                let else_type = else_arm.check_node(table)?;
                if then_type != else_type {
                    return Err(Error::CustomErrorSpan {
                        message:
                            "The different outcomes of an if expression should have the same type"
                                .to_string(),
                        span: meta.clone(),
                    });
                }
                Ok(then_type)
            }
            ASTNode::ArithmeticNode { meta, op: _, expr } => match expr.check_node(table)? {
                TypeAssignment::Single(Type::Nat) => Ok(TypeAssignment::Single(Type::Nat)),
                _ => Err(Error::CustomErrorSpan {
                    message: "Arithmetic expression should have Nat as type".to_string(),
                    span: meta.clone(),
                }),
            },
            ASTNode::ApplicationNode { meta, left, right } => {
                let left_type = left.check_node(table)?;
                let right_type = right.check_node(table)?;
                if let TypeAssignment::Arrow(first, second) = left_type {
                    if *first == right_type {
                        Ok(*second)
                    } else {
                        Err(Error::CustomErrorSpan {
                        message: "Incorrect type of right argument in an application for the function type of the left argument".to_string(),
                        span: meta.clone(),
                    })
                    }
                } else {
                    Err(Error::CustomErrorSpan {
                        message: "Left argument of an application should be a function type"
                            .to_string(),
                        span: meta.clone(),
                    })
                }
            }
            ASTNode::AbstractionNode {
                meta: _,
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
            ASTNode::ProjectionNode {
                meta,
                target,
                attrib,
            } => {
                if let TypeAssignment::Record(types) = target.check_node(table)? {
                    if let Some(attrib_type) = types.get(attrib) {
                        Ok(attrib_type.clone())
                    } else {
                        Err(Error::CustomErrorSpan {
                            message: "Attribute of projection is not part of the record type"
                                .to_string(),
                            span: meta.clone(),
                        })
                    }
                } else {
                    Err(Error::CustomErrorSpan {
                        message: "Target of a projection should be of type record".to_string(),
                        span: meta.clone(),
                    })
                }
            }
            ASTNode::RecordNode { meta: _, records } => {
                let mut types: HashMap<String, TypeAssignment> = HashMap::new();
                for (name, node) in records {
                    types.insert(name.to_string(), node.check_node(table)?);
                }
                Ok(TypeAssignment::Record(types))
            }
            ASTNode::MatchingNode {
                meta,
                to_match,
                cases,
            } => {
                if let TypeAssignment::Variant(variant) = to_match.check_node(table)? {
                    let mut arm_type = None;
                    for (variant_name, variant_type) in variant {
                        if let Some((ident, arm)) = cases.get(&variant_name) {
                            table.push(Scope::new(ident.to_string(), variant_type.clone()));

                            if arm_type == None {
                                arm_type = Some(arm.check_node(table)?);
                            } else {
                                if arm_type != Some(arm.check_node(table)?) {
                                    return Err(Error::CustomErrorSpan {
                                        message: "All outcomes of a case expression should result in the same type".to_string(),
                                        span: meta.clone(),
                                    });
                                }
                            }
                        } else {
                            return Err(Error::CustomErrorSpan {
                                message: "Not all possible types in the variant are being handled"
                                    .to_string(),
                                span: meta.clone(),
                            });
                        }
                    }
                    if let Some(arm_type) = arm_type {
                        Ok(arm_type)
                    } else {
                        Err(Error::CustomErrorSpan {
                            message: "Variant can't be empty".to_string(),
                            span: meta.clone(),
                        })
                    }
                } else {
                    Err(Error::CustomErrorSpan {
                        message: "Argument of case expression should be a variant".to_string(),
                        span: meta.clone(),
                    })
                }
            }
            ASTNode::TaggingNode {
                meta,
                ident,
                value,
                data_type,
            } => {
                let value_type = value.check_node(table)?;
                if data_type.has_variant(ident, value_type) {
                    Ok(data_type.clone())
                } else {
                    Err(Error::CustomErrorSpan {
                        message: "Type of tagged value is not part of the variant".to_string(),
                        span: meta.clone(),
                    })
                }
            }
            ASTNode::FixNode { meta, point } => {
                if let TypeAssignment::Arrow(from, to) = point.check_node(table)? {
                    if from == to {
                        Ok(*to)
                    } else {
                        Err(Error::CustomErrorSpan {
                            message:
                                "Function argument of fixpoint does not result in the same type"
                                    .to_string(),
                            span: meta.clone(),
                        })
                    }
                } else {
                    Err(Error::CustomErrorSpan {
                        message: "Argument of fixpoint is not a function".to_string(),
                        span: meta.clone(),
                    })
                }
            }
        }
    }
}
