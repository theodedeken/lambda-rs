use parser::*;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ASTError {
    message: String,
}

impl ASTError {
    pub fn new(message: String) -> ASTError {
        ASTError { message }
    }
}

impl fmt::Display for ASTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ASTError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    True,
    False,
    Zero,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Succ,
    Pred,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeAssignment {
    Single(Type),
    Arrow(Box<TypeAssignment>, Box<TypeAssignment>),
    Record(HashMap<String, TypeAssignment>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Bool,
    Nat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    AbstractionNode {
        ident: String,
        data_type: TypeAssignment,
        body: Box<ASTNode>,
    },
    ApplicationNode {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    IdentifierNode {
        name: String,
    },
    ConditionNode {
        clause: Box<ASTNode>,
        then_arm: Box<ASTNode>,
        else_arm: Box<ASTNode>,
    },
    ArithmeticNode {
        op: Operator,
        expr: Box<ASTNode>,
    },
    IsZeroNode {
        expr: Box<ASTNode>,
    },
    ValueNode {
        value: Value,
    },
    ProjectionNode {
        target: Box<ASTNode>,
        attrib: String,
    },
    RecordNode {
        records: HashMap<String, ASTNode>,
    },
}

impl ASTNode {
    pub fn print(&self) {
        self.print_node(0)
    }

    fn print_node(&self, level: usize) {
        match self {
            ASTNode::AbstractionNode {
                ident,
                data_type,
                body,
            } => {
                println!(
                    "{}Abstraction with variable {} of type {:?}",
                    "\t".repeat(level),
                    ident,
                    data_type
                );
                body.print_node(level + 1);
            }
            ASTNode::ApplicationNode { left, right } => {
                println!("{}Application", "\t".repeat(level));
                left.print_node(level + 1);
                right.print_node(level + 1);
            }
            ASTNode::IdentifierNode { name } => {
                println!("{}Identifier with name {}", "\t".repeat(level), name);
            }
            ASTNode::IsZeroNode { expr } => {
                println!("{}IsZero", "\t".repeat(level));
                expr.print_node(level + 1);
            }
            ASTNode::ValueNode { value } => {
                println!("{}Value {:?}", "\t".repeat(level), value);
            }
            ASTNode::ArithmeticNode { op, expr } => {
                println!("{}Arithmetic with operator {:?}", "\t".repeat(level), op);
                expr.print_node(level + 1);
            }
            ASTNode::ConditionNode {
                clause,
                then_arm,
                else_arm,
            } => {
                println!("{}Condition", "\t".repeat(level));
                clause.print_node(level + 1);
                then_arm.print_node(level + 1);
                else_arm.print_node(level + 1);
            }
            ASTNode::ProjectionNode { target, attrib } => {
                println!("{}Projection to {} on", "\t".repeat(level), attrib);
                target.print_node(level + 1);
            }
            ASTNode::RecordNode { records } => {
                println!("{}Record with elements:", "\t".repeat(level));
                for (name, assign) in records {
                    println!("{}  {} =", "\t".repeat(level), name);
                    assign.print_node(level + 1);
                }
            }
        }
    }
}

pub fn build_ast(mut parsed: Pairs<Rule>) -> Result<ASTNode, Box<Error>> {
    let first = parsed
        .next()
        .ok_or_else(|| Box::new(ASTError::new(format!("Invalid program"))))?;
    build_node(first)
}

fn build_node(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let rule = pair.as_rule();
    match rule {
        Rule::program => build_program(pair),
        Rule::application => build_application(pair),
        Rule::abstraction => build_abstraction(pair),
        Rule::ident => build_ident(pair),
        Rule::p_ident => build_ident(pair),
        Rule::arithmetic => build_arithmetic(pair),
        Rule::zero_check => build_zero_check(pair),
        Rule::if_then => build_if_then(pair),
        Rule::projection => build_projection(pair),
        Rule::record => build_record(pair),
        Rule::val_zero => Ok(ASTNode::ValueNode { value: Value::Zero }),
        Rule::val_true => Ok(ASTNode::ValueNode { value: Value::True }),
        Rule::val_false => Ok(ASTNode::ValueNode {
            value: Value::False,
        }),
        _ => Err(Box::new(ASTError::new(format!(
            "Not implemented: {:?}",
            rule
        )))),
    }
}

fn build_program(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    match inner.get(0) {
        Some(x) => build_node(x.clone()),
        None => Err(Box::new(ASTError::new(format!(
            "No application body in program"
        )))),
    }
}

fn build_application(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 1 {
        build_node(inner[0].clone())
    } else if inner.len() == 2 {
        Ok(ASTNode::ApplicationNode {
            left: Box::new(build_node(inner[0].clone())?),
            right: Box::new(build_node(inner[1].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found application with incorrect number of arguments"
        ))))
    }
}

fn build_abstraction(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 3 {
        let data_type = build_type(inner[1].clone());

        let span = inner[0].clone().into_span();
        Ok(ASTNode::AbstractionNode {
            ident: span.as_str().to_string(),
            data_type: data_type?,
            body: Box::new(build_node(inner[2].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found abstraction with incorrect number of arguments"
        ))))
    }
}

fn build_type(pair: Pair<'_, Rule>) -> Result<TypeAssignment, Box<Error>> {
    match pair.as_rule() {
        Rule::type_nat => Ok(TypeAssignment::Single(Type::Nat)),
        Rule::type_bool => Ok(TypeAssignment::Single(Type::Bool)),
        Rule::type_arrow => {
            let arrow: Vec<Pair<'_, Rule>> = pair.into_inner().collect();
            let left = build_type(arrow[0].clone())?;
            let right = build_type(arrow[1].clone())?;
            Ok(TypeAssignment::Arrow(Box::new(left), Box::new(right)))
        }
        Rule::type_record => {
            let mut map = HashMap::new();
            for el in pair.into_inner() {
                let attribs: Vec<Pair<'_, Rule>> = el.into_inner().collect();
                let record_type = build_type(attribs[1].clone())?;
                map.insert(
                    attribs[0].clone().into_span().as_str().to_string(),
                    record_type,
                );
            }
            Ok(TypeAssignment::Record(map))
        }
        _ => Err(Box::new(ASTError::new(format!("Incorrect type")))),
    }
}

fn build_ident(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let span = pair.into_span();
    let mut string = span.as_str().to_string();
    // hack because of bug in parser
    string = string.chars().filter(|chr| chr != &' ').collect();
    Ok(ASTNode::IdentifierNode { name: string })
}

fn build_arithmetic(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 2 {
        let op = match inner[0].as_rule() {
            Rule::op_succ => Ok(Operator::Succ),
            Rule::op_pred => Ok(Operator::Pred),
            _ => Err(Box::new(ASTError::new(format!("Incorrect operator")))),
        };
        Ok(ASTNode::ArithmeticNode {
            op: op?,
            expr: Box::new(build_node(inner[1].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found arithmetic with incorrect number of arguments"
        ))))
    }
}

fn build_zero_check(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();
    if inner.len() == 1 {
        Ok(ASTNode::IsZeroNode {
            expr: Box::new(build_node(inner[0].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found zero check with incorrect number of arguments"
        ))))
    }
}

fn build_if_then(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 3 {
        Ok(ASTNode::ConditionNode {
            clause: Box::new(build_node(inner[0].clone())?),
            then_arm: Box::new(build_node(inner[1].clone())?),
            else_arm: Box::new(build_node(inner[2].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found ifthenelse with incorrect number of arguments"
        ))))
    }
}

fn build_projection(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let parts: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    let target = build_node(parts[0].clone())?;
    let attrib = parts[1].clone().into_span().as_str().to_string();
    Ok(ASTNode::ProjectionNode {
        target: Box::new(target),
        attrib,
    })
}

fn build_record(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let mut records = HashMap::new();
    for el in pair.into_inner() {
        let parts: Vec<Pair<'_, Rule>> = el.into_inner().collect();
        let name = parts[0].clone().into_span().as_str().to_string();
        let result = build_node(parts[1].clone())?;
        records.insert(name, result);
    }
    Ok(ASTNode::RecordNode { records })
}
