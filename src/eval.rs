use ast::*;
use std::collections::HashMap;
use std::fmt::*;
use sym_tab::*;

//type Abstr = Fn(OutputValue) -> OutputValue + 'static;

#[derive(Clone, Debug, PartialEq)]
pub enum OutputValue<'a> {
    Nat(usize),
    Bool(bool),
    Func(String, Box<ASTNode<'a>>, SymbolTable<OutputValue<'a>>),
    Record(HashMap<String, OutputValue<'a>>),
    Variant(String, Box<OutputValue<'a>>),
    Fix(Box<OutputValue<'a>>),
}

impl<'a> Display for OutputValue<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            OutputValue::Nat(x) => write!(f, "{}", x),
            OutputValue::Bool(x) => write!(f, "{}", x),
            OutputValue::Record(records) => {
                let list: Vec<String> = records
                    .iter()
                    .map(|(name, val)| format!("{}={}", name, val))
                    .collect();
                write!(f, "{{{}}}", list.join(", "))
            }
            // TODO function
            OutputValue::Func(par, body, _) => write!(f, "@ {}. {:?}", par, body),
            OutputValue::Variant(ident, value) => write!(f, "<{}={}>", ident, value),
            OutputValue::Fix(body) => write!(f, "{:?}", body),
        }
    }
}

impl<'a> ASTNode<'a> {
    pub fn eval(&self) -> OutputValue {
        self.eval_node(&mut SymbolTable::new())
    }

    fn eval_node(&self, table: &mut SymbolTable<OutputValue<'a>>) -> OutputValue<'a> {
        match self {
            ASTNode::AbstractionNode {
                meta: _,
                ident,
                data_type: _,
                body,
            } => OutputValue::Func(ident.to_string(), body.clone(), table.clone()),
            ASTNode::ApplicationNode { meta, left, right } => {
                let left_val = left.eval_node(table);
                if let OutputValue::Func(ident, body, mut func_table) = left_val {
                    let right_val = right.eval_node(table);
                    func_table.push(Scope::new(ident, right_val));
                    body.eval_node(&mut func_table)
                } else if let OutputValue::Fix(func) = left_val {
                    let destr = *func;
                    if let OutputValue::Func(ident, body, mut _func_table) = destr {
                        table.remove(&ident);
                        let newnode = ASTNode::ApplicationNode {
                            meta: meta.clone(),
                            left: Box::new(ASTNode::FixNode {
                                meta: meta.clone(),
                                point: Box::new(ASTNode::AbstractionNode {
                                    meta: meta.clone(),
                                    ident,
                                    body,
                                    data_type: TypeAssignment::Single(Type::Bool),
                                }),
                            }),
                            right: right.clone(),
                        };
                        newnode.eval_node(table)
                    } else {
                        panic!("Bug in typechecker: in evaluation of application the left argument was not evaluated to a function");
                    }
                } else {
                    panic!("Bug in typechecker: in evaluation of application the left argument was not evaluated to a function");
                }
            }
            ASTNode::ArithmeticNode { meta: _, op, expr } => {
                if let OutputValue::Nat(x) = expr.eval_node(table) {
                    match op {
                        Operator::Pred => if x == 0 {
                            OutputValue::Nat(0)
                        } else {
                            OutputValue::Nat(x - 1)
                        },
                        Operator::Succ => OutputValue::Nat(x + 1),
                    }
                } else {
                    panic!("Bug in typechecker: in evaluation of pred/succ expr did not return variable of type Nat");
                }
            }
            ASTNode::ConditionNode {
                meta: _,
                clause,
                then_arm,
                else_arm,
            } => {
                if let OutputValue::Bool(x) = clause.eval_node(table) {
                    if x {
                        then_arm.eval_node(table)
                    } else {
                        else_arm.eval_node(table)
                    }
                } else {
                    panic!("Bug in typechecker: in evaluation of ifthenelse clause did not return variable of type Bool");
                }
            }
            ASTNode::IdentifierNode { meta: _, name } => {
                let value = table
                    .lookup(name)
                    .expect("Bug in typechecker: came across unknown variable");

                value.clone()
            }
            ASTNode::IsZeroNode { meta: _, expr } => {
                if let OutputValue::Nat(x) = expr.eval_node(table) {
                    OutputValue::Bool(x == 0)
                } else {
                    panic!("Bug in typechecker: in evaluation of iszero expr did not return variable of type Nat");
                }
            }
            ASTNode::ValueNode { meta: _, value } => match value {
                Value::True => OutputValue::Bool(true),
                Value::False => OutputValue::Bool(false),
                Value::Zero => OutputValue::Nat(0),
            },
            ASTNode::ProjectionNode {
                meta: _,
                target,
                attrib,
            } => {
                if let OutputValue::Record(records) = target.eval_node(table) {
                    if let Some(output) = records.get(attrib) {
                        output.clone()
                    } else {
                        panic!("Bug in typechecker: in evaluation of projection the attribute was not found")
                    }
                } else {
                    panic!("Bug in typechecker: in evaluation of projection type target type was not a record")
                }
            }
            ASTNode::RecordNode { meta: _, records } => {
                let mut map = HashMap::new();
                for (name, node) in records {
                    map.insert(name.to_string(), node.eval_node(table));
                }
                OutputValue::Record(map)
            }
            ASTNode::MatchingNode {
                meta: _,
                to_match,
                cases,
            } => {
                if let OutputValue::Variant(ident, value) = to_match.eval_node(table) {
                    if let Some((case, arm)) = cases.get(&ident) {
                        table.push(Scope::new(case.to_string(), *value));
                        arm.eval_node(table)
                    } else {
                        panic!("Bug in typechecker: argument of case has no corresponding arm")
                    }
                } else {
                    panic!("Bug in typechecker: argument of case was not a variant")
                }
            }
            ASTNode::TaggingNode {
                meta: _,
                ident,
                value,
                data_type: _,
            } => OutputValue::Variant(ident.to_string(), Box::new(value.eval_node(table))),
            ASTNode::FixNode { meta: _, point } => {
                let left_val = point.eval_node(table);
                if let OutputValue::Func(ident, body, mut table) = left_val.clone() {
                    table.push(Scope::new(ident, OutputValue::Fix(Box::new(left_val))));
                    body.eval_node(&mut table)
                } else {
                    panic!("Bug in typechecker: in evaluation of fixpoint the left argument was not evaluated to a function");
                }
            }
        }
    }
}
