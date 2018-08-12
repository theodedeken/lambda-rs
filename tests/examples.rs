extern crate lambda_rs;
extern crate pest;

use lambda_rs::{ast::build_ast, eval::OutputValue, parser::parse_file, read_file};
use std::collections::HashMap;

fn run_file<'a>(filename: &'a str, expected: OutputValue) {
    let contents = read_file(&filename).unwrap_or_else(|_e| panic!("Cant read file"));
    let pairs = parse_file(&contents).unwrap_or_else(|_e| {
        panic!("Problem when parsing file");
    });
    let ast_tree = build_ast(pairs);
    let _tree_type = ast_tree
        .check::<i32>()
        .unwrap_or_else(|e| panic!(format!("Typechecking for {} failed with {}", filename, e)));
    assert_eq!(ast_tree.eval(), expected);
}

#[test]
fn evaluate_examples() {
    run_file("examples/correct0.lambda", OutputValue::Nat(2));
    run_file("examples/correct1.lambda", OutputValue::Nat(1));
    run_file("examples/correct2.lambda", OutputValue::Nat(2));
    run_file("examples/correct3.lambda", OutputValue::Nat(1));
    run_file("examples/correct4.lambda", OutputValue::Nat(1));
    run_file("examples/correct5.lambda", OutputValue::Nat(0));
    run_file("examples/arrowtype.lambda", OutputValue::Nat(1));
    run_file("examples/high-order.lambda", OutputValue::Nat(2));

    let mut testmap = HashMap::new();
    testmap.insert("status".to_string(), OutputValue::Bool(true));
    testmap.insert("result".to_string(), OutputValue::Nat(1));

    run_file("examples/record.lambda", OutputValue::Record(testmap));
    run_file("examples/record_proj.lambda", OutputValue::Nat(1));
    run_file("examples/variant1.lambda", OutputValue::Nat(0));
    run_file("examples/variant2.lambda", OutputValue::Nat(3));
    run_file("examples/iseven1.lambda", OutputValue::Bool(true));
    run_file("examples/iseven2.lambda", OutputValue::Bool(false));
}
