use ast::*;
use sym_tab::*;

//type Abstr = Fn(OutputValue) -> OutputValue + 'static;

#[derive(Clone, Debug, PartialEq)]
pub enum OutputValue {
    Nat(usize),
    Bool(bool),
    Func(String, Box<ASTNode>, SymbolTable<OutputValue>),
    //Func(Rc<Abstr>),
}

impl ASTNode {
    pub fn eval(&self) -> OutputValue {
        self.eval_node(&mut SymbolTable::new())
    }

    fn eval_node(&self, table: &mut SymbolTable<OutputValue>) -> OutputValue {
        match self {
            ASTNode::AbstractionNode {
                ident,
                data_type,
                body,
            } => OutputValue::Func(ident.to_string(), body.clone(), table.clone()),
            ASTNode::ApplicationNode { left, right } => {
                let left_val = left.eval_node(table);
                let right_val = right.eval_node(table);
                if let OutputValue::Func(ident, body, mut table) = left_val {
                    table.push(Scope::new(ident, right_val));
                    body.eval_node(&mut table)
                } else {
                    panic!("Bug in typechecker: in evaluation of application the left argument was not evaluated to a function");
                }
            }
            ASTNode::ArithmeticNode { op, expr } => {
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
            ASTNode::IdentifierNode { name } => {
                let value = table
                    .lookup(name)
                    .expect("Bug in typechecker: came across unknown variable");
                value.clone()
            }
            ASTNode::IsZeroNode { expr } => {
                if let OutputValue::Nat(x) = expr.eval_node(table) {
                    OutputValue::Bool(x == 0)
                } else {
                    panic!("Bug in typechecker: in evaluation of iszero expr did not return variable of type Nat");
                }
            }
            ASTNode::ValueNode { value } => match value {
                Value::True => OutputValue::Bool(true),
                Value::False => OutputValue::Bool(false),
                Value::Zero => OutputValue::Nat(0),
            },
        }
    }
}
