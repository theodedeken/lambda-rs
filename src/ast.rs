use parser::*;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Span;
use std::collections::HashMap;
use std::fmt::*;

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
pub enum ASTNode<'a> {
    AbstractionNode {
        meta: Span<'a>,
        ident: String,
        data_type: TypeAssignment,
        body: Box<ASTNode<'a>>,
    },
    ApplicationNode {
        meta: Span<'a>,
        left: Box<ASTNode<'a>>,
        right: Box<ASTNode<'a>>,
    },
    IdentifierNode {
        meta: Span<'a>,
        name: String,
    },
    ConditionNode {
        meta: Span<'a>,
        clause: Box<ASTNode<'a>>,
        then_arm: Box<ASTNode<'a>>,
        else_arm: Box<ASTNode<'a>>,
    },
    ArithmeticNode {
        meta: Span<'a>,
        op: Operator,
        expr: Box<ASTNode<'a>>,
    },
    IsZeroNode {
        meta: Span<'a>,
        expr: Box<ASTNode<'a>>,
    },
    ValueNode {
        meta: Span<'a>,
        value: Value,
    },
    ProjectionNode {
        meta: Span<'a>,
        target: Box<ASTNode<'a>>,
        attrib: String,
    },
    RecordNode {
        meta: Span<'a>,
        records: HashMap<String, ASTNode<'a>>,
    },
    MatchingNode {
        meta: Span<'a>,
        to_match: Box<ASTNode<'a>>,
        cases: HashMap<String, (String, Box<ASTNode<'a>>)>,
    },
    TaggingNode {
        meta: Span<'a>,
        ident: String,
        value: Box<ASTNode<'a>>,
        data_type: TypeAssignment,
    },
    FixNode {
        meta: Span<'a>,
        point: Box<ASTNode<'a>>,
    },
}

impl<'a> Display for ASTNode<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.print_node(f, 0)
    }
}
impl<'a> ASTNode<'a> {
    fn print_node(&self, f: &mut Formatter, level: usize) -> Result {
        match self {
            ASTNode::AbstractionNode {
                meta: _,
                ident,
                data_type,
                body,
            } => {
                writeln!(
                    f,
                    "{}Abstraction with variable {} of type {:?}",
                    "\t".repeat(level),
                    ident,
                    data_type
                )?;
                body.print_node(f, level + 1)
            }
            ASTNode::ApplicationNode {
                meta: _,
                left,
                right,
            } => {
                writeln!(f, "{}Application", "\t".repeat(level))?;
                left.print_node(f, level + 1)?;
                right.print_node(f, level + 1)
            }
            ASTNode::IdentifierNode { meta: _, name } => {
                writeln!(f, "{}Identifier with name {}", "\t".repeat(level), name)
            }
            ASTNode::IsZeroNode { meta: _, expr } => {
                writeln!(f, "{}IsZero", "\t".repeat(level))?;
                expr.print_node(f, level + 1)
            }
            ASTNode::ValueNode { meta: _, value } => {
                writeln!(f, "{}Value {:?}", "\t".repeat(level), value)
            }
            ASTNode::ArithmeticNode { meta: _, op, expr } => {
                writeln!(f, "{}Arithmetic with operator {:?}", "\t".repeat(level), op)?;
                expr.print_node(f, level + 1)
            }
            ASTNode::ConditionNode {
                meta: _,
                clause,
                then_arm,
                else_arm,
            } => {
                writeln!(f, "{}Condition", "\t".repeat(level))?;
                clause.print_node(f, level + 1)?;
                then_arm.print_node(f, level + 1)?;
                else_arm.print_node(f, level + 1)
            }
            ASTNode::ProjectionNode {
                meta: _,
                target,
                attrib,
            } => {
                writeln!(f, "{}Projection to {} on", "\t".repeat(level), attrib)?;
                target.print_node(f, level + 1)
            }
            ASTNode::RecordNode { meta: _, records } => {
                writeln!(f, "{}Record with elements:", "\t".repeat(level))?;
                for (name, assign) in records {
                    writeln!(f, "{}  {} =", "\t".repeat(level), name)?;
                    assign.print_node(f, level + 1)?;
                }
                write!(f, "")
            }
            ASTNode::MatchingNode {
                meta: _,
                to_match,
                cases,
            } => {
                writeln!(
                    f,
                    "{}Match case of {:?} with elements:",
                    "\t".repeat(level),
                    to_match
                )?;
                for (variant, (ident, arm)) in cases {
                    writeln!(f, "{}  {}={} => ", "\t".repeat(level), variant, ident)?;
                    arm.print_node(f, level + 1)?;
                }
                write!(f, "")
            }
            ASTNode::TaggingNode {
                meta: _,
                ident,
                value,
                data_type,
            } => {
                writeln!(
                    f,
                    "{}Tag of {} to {:?}",
                    "\t".repeat(level),
                    ident,
                    data_type
                )?;
                value.print_node(f, level + 1)
            }
            ASTNode::FixNode { meta: _, point } => {
                writeln!(f, "{}Fixpoint", "\t".repeat(level))?;
                point.print_node(f, level + 1)
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
        Rule::fixpoint => build_fixpoint(pair),
        Rule::val_zero => ASTNode::ValueNode {
            meta: pair.into_span(),
            value: Value::Zero,
        },
        Rule::val_true => ASTNode::ValueNode {
            meta: pair.into_span(),
            value: Value::True,
        },
        Rule::val_false => ASTNode::ValueNode {
            meta: pair.into_span(),
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
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();

    let first = build_node(
        inner
            .next()
            .expect("Bug in parser: found an application with incorrect number of arguments"),
    );

    if let Some(second) = inner.next() {
        ASTNode::ApplicationNode {
            meta: pair.into_span(),
            left: Box::new(first),
            right: Box::new(build_node(second)),
        }
    } else {
        first
    }
}

fn build_abstraction(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();

    let (ident, data_type) = build_type_term(
        inner
            .next()
            .expect("Bug in parser: found an abstraction with incorrect number of arguments"),
    );
    let body = Box::new(build_node(inner.next().expect(
        "Bug in parser: found an abstraction with incorrect number of arguments",
    )));

    ASTNode::AbstractionNode {
        meta: pair.into_span(),
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
    let span = pair.clone().into_span();
    let mut string = span.as_str().to_string();
    // hack because of bug in parser
    string = string.chars().filter(|chr| chr != &' ').collect();
    ASTNode::IdentifierNode {
        meta: pair.into_span(),
        name: string,
    }
}

fn build_arithmetic(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();

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
        meta: pair.into_span(),
        op,
        expr: Box::new(build_node(inner.next().expect(
            "Bug in parser: found an arithmetic expression with incorrect number of arguments",
        ))),
    }
}

fn build_zero_check(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();
    ASTNode::IsZeroNode {
        meta: pair.into_span(),
        expr: Box::new(build_node(inner.next().expect(
            "Bug in parser: found an iszero check with incorrect number of arguments",
        ))),
    }
}

fn build_if_then(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner = pair.clone().into_inner().map(|el| Box::new(build_node(el)));
    ASTNode::ConditionNode {
        meta: pair.into_span(),
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
    let mut parts: Pairs<'_, Rule> = pair.clone().into_inner();

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
        meta: pair.into_span(),
        target: Box::new(target),
        attrib,
    }
}

fn build_record(pair: Pair<'_, Rule>) -> ASTNode {
    let mut records = HashMap::new();
    for el in pair.clone().into_inner() {
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
    ASTNode::RecordNode {
        meta: pair.into_span(),
        records,
    }
}

fn build_matching(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();

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
        meta: pair.into_span(),
        to_match: Box::new(to_match),
        cases,
    }
}

fn build_tagging(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();
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
        meta: pair.into_span(),
        ident,
        value: Box::new(value),
        data_type,
    }
}

fn build_fixpoint(pair: Pair<'_, Rule>) -> ASTNode {
    let mut inner: Pairs<'_, Rule> = pair.clone().into_inner();
    let point = build_node(
        inner
            .next()
            .expect("Bug in parser: got a fix without argument"),
    );
    ASTNode::FixNode {
        meta: pair.into_span(),
        point: Box::new(point),
    }
}
