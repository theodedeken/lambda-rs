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
    Variant(HashMap<String, TypeAssignment>),
}

impl TypeAssignment {
    pub fn has_variant(&self, ident: &str, data_type: TypeAssignment) -> bool {
        match self {
            TypeAssignment::Variant(variants) => {
                if let Some(value) = variants.get(ident) {
                    value == &data_type
                } else {
                    false
                }
            }
            _ => false,
        }
    }
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
    MatchingNode {
        to_match: Box<ASTNode>,
        cases: HashMap<String, (String, Box<ASTNode>)>,
    },
    TaggingNode {
        ident: String,
        value: Box<ASTNode>,
        data_type: TypeAssignment,
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
            ASTNode::MatchingNode { to_match, cases } => {
                println!(
                    "{}Match case of {:?} with elements:",
                    "\t".repeat(level),
                    to_match
                );
                for (variant, (ident, arm)) in cases {
                    println!("{}  {}={} => ", "\t".repeat(level), variant, ident);
                    arm.print_node(level + 1)
                }
            }
            ASTNode::TaggingNode {
                ident,
                value,
                data_type,
            } => {
                println!("{}Tag of {} to {:?}", "\t".repeat(level), ident, data_type);
            }
        }
    }
}

pub fn build_ast(mut parsed: Pairs<Rule>) -> ASTNode {
    let first = parsed.next().expect("Empty program");
    build_node(first)
}

fn build_node(pair: Pair<'_, Rule>) -> ASTNode {
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
        Rule::matching => build_matching(pair),
        Rule::tagging => build_tagging(pair),
        Rule::val_zero => ASTNode::ValueNode { value: Value::Zero },
        Rule::val_true => ASTNode::ValueNode { value: Value::True },
        Rule::val_false => ASTNode::ValueNode {
            value: Value::False,
        },
        _ => panic!(format!("Building of {:?} not implemented", rule)),
    }
}

fn build_program(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();
    build_node(inner.next().expect("Bug in parser: got empty program"))
}

fn build_application(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();

    let first = build_node(
        inner
            .next()
            .expect("Bug in parser: found an application with incorrect number of arguments"),
    );

    if let Some(second) = inner.next() {
        ASTNode::ApplicationNode {
            left: Box::new(first),
            right: Box::new(build_node(second)),
        }
    } else {
        first
    }
}

fn build_abstraction(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();

    let (ident, data_type) = build_type_term(
        inner
            .next()
            .expect("Bug in parser: found an abstraction with incorrect number of arguments"),
    );
    let body = Box::new(build_node(inner.next().expect(
        "Bug in parser: found an abstraction with incorrect number of arguments",
    )));

    ASTNode::AbstractionNode {
        ident,
        data_type,
        body,
    }
}

fn build_type_term(pair: Pair<'_, Rule>) -> (String, TypeAssignment) {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();
    let name = inner
        .next()
        .expect("Bug in parser: found a type term with incorrect number of arguments")
        .into_span()
        .as_str()
        .to_string();
    let data_type = build_type(
        inner
            .next()
            .expect("Bug in parser: found a type term with incorrect number of arguments"),
    );
    (name, data_type)
}

fn build_type(pair: Pair<'_, Rule>) -> TypeAssignment {
    match pair.as_rule() {
        Rule::type_nat => TypeAssignment::Single(Type::Nat),
        Rule::type_bool => TypeAssignment::Single(Type::Bool),
        Rule::type_arrow => {
            let mut arrow: Pairs<'_, Rule> = pair.into_inner();
            let left =
                build_type(arrow.next().expect(
                    "Bug in parser: found an arrow type with incorrect number of arguments",
                ));
            let right =
                build_type(arrow.next().expect(
                    "Bug in parser: found an arrow type with incorrect number of arguments",
                ));
            TypeAssignment::Arrow(Box::new(left), Box::new(right))
        }
        Rule::type_record => {
            let mut map = HashMap::new();
            for el in pair.into_inner() {
                let (ident, data_type) = build_type_term(el);
                map.insert(ident, data_type);
            }
            TypeAssignment::Record(map)
        }
        Rule::type_variant => {
            let mut map = HashMap::new();
            for el in pair.into_inner() {
                let (ident, data_type) = build_type_term(el);
                map.insert(ident, data_type);
            }
            TypeAssignment::Variant(map)
        }
        _ => panic!(format!("Incorrect type {:?}", pair)),
    }
}

fn build_ident(pair: Pair<'_, Rule>) -> ASTNode {
    let span = pair.into_span();
    let mut string = span.as_str().to_string();
    // hack because of bug in parser
    string = string.chars().filter(|chr| chr != &' ').collect();
    ASTNode::IdentifierNode { name: string }
}

fn build_arithmetic(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();

    let op = match inner
        .next()
        .expect("Bug in parser: found an arithmetic expression with incorrect number of arguments")
        .as_rule()
    {
        Rule::op_succ => Operator::Succ,
        Rule::op_pred => Operator::Pred,
        _ => panic!(format!("Incorrect operator")),
    };
    ASTNode::ArithmeticNode {
        op,
        expr: Box::new(build_node(inner.next().expect(
            "Bug in parser: found an arithmetic expression with incorrect number of arguments",
        ))),
    }
}

fn build_zero_check(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();
    ASTNode::IsZeroNode {
        expr: Box::new(build_node(inner.next().expect(
            "Bug in parser: found an iszero check with incorrect number of arguments",
        ))),
    }
}

fn build_if_then(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner = pair.into_inner().map(|el| Box::new(build_node(el)));
    ASTNode::ConditionNode {
        clause: inner
            .next()
            .expect("Bug in parser: found an ifthenelse with incorrect number of arguments"),
        then_arm: inner
            .next()
            .expect("Bug in parser: found an ifthenelse with incorrect number of arguments"),
        else_arm: inner
            .next()
            .expect("Bug in parser: found an ifthenelse with incorrect number of arguments"),
    }
}

fn build_projection(pair: Pair<'_, Rule>) -> ASTNode {
    let mut parts: Pairs<'_, Rule> = pair.into_inner();

    let target = build_node(
        parts
            .next()
            .expect("Bug in parser: found a projection with incorrect number of arguments"),
    );
    let attrib = parts
        .next()
        .expect("Bug in parser: found a projection with incorrect number of arguments")
        .into_span()
        .as_str()
        .to_string();
    ASTNode::ProjectionNode {
        target: Box::new(target),
        attrib,
    }
}

fn build_record(pair: Pair<'_, Rule>) -> ASTNode {
    let mut records = HashMap::new();
    for el in pair.into_inner() {
        let mut parts: Pairs<'_, Rule> = el.into_inner();
        let name = parts
            .next()
            .expect("Bug in parser: found a record element with incorrect number of arguments")
            .into_span()
            .as_str()
            .to_string();
        let result =
            build_node(parts.next().expect(
                "Bug in parser: found a record element with incorrect number of arguments",
            ));
        records.insert(name, result);
    }
    ASTNode::RecordNode { records }
}

fn build_matching(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();

    let to_match = build_node(
        inner
            .next()
            .expect("Bug in parser: got a matching expression without arguments"),
    );
    let mut cases = HashMap::new();
    for case_el in inner {
        let mut inner: Pairs<'_, Rule> = case_el.into_inner();

        let variant = inner
            .next()
            .expect("Bug in parser: got a case expression with incorrect number of arguments")
            .into_span()
            .as_str()
            .to_string();
        let ident = inner
            .next()
            .expect("Bug in parser: got a case expression with incorrect number of arguments")
            .into_span()
            .as_str()
            .to_string();
        let arm = Box::new(build_node(inner.next().expect(
            "Bug in parser: got a case expression with incorrect number of arguments",
        )));
        cases.insert(variant, (ident, arm));
    }
    ASTNode::MatchingNode {
        to_match: Box::new(to_match),
        cases,
    }
}

fn build_tagging(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.into_inner();
    let ident = inner
        .next()
        .expect("Bug in parser: got a tagging expression with incorrect number of arguments")
        .into_span()
        .as_str()
        .to_string();
    let value = build_node(
        inner
            .next()
            .expect("Bug in parser: got a tagging expression with incorrect number of arguments"),
    );
    let data_type = build_type(
        inner
            .next()
            .expect("Bug in parser: got a tagging expression with incorrect number of arguments"),
    );

    ASTNode::TaggingNode {
        ident,
        value: Box::new(value),
        data_type,
    }
}
