extern crate lambda_rs;

use lambda_rs::{ast::build_ast, eval::OutputValue, parser::parse_file, read_file};
use std::error::Error;

fn run_file(filename: &str, expected: OutputValue) -> Result<(), Box<Error>> {
    let contents = read_file(&filename)?;
    let pairs = parse_file(&contents).unwrap_or_else(|_e| {
        panic!("Problem when parsing file");
    });
    let ast_tree = build_ast(pairs)?;
    let _tree_type = ast_tree.check()?;
    assert_eq!(ast_tree.eval(), expected);
    Ok(())
}

#[test]
fn evaluate_examples() {
    run_file("examples/correct0.lambda", OutputValue::Nat(2))
        .expect("Integration test for correct0 failed");
    run_file("examples/correct1.lambda", OutputValue::Nat(1))
        .expect("Integration test for correct1 failed");
    run_file("examples/correct2.lambda", OutputValue::Nat(2))
        .expect("Integration test for correct2 failed");
    run_file("examples/correct3.lambda", OutputValue::Nat(1))
        .expect("Integration test for correct3 failed");
    run_file("examples/correct4.lambda", OutputValue::Nat(1))
        .expect("Integration test for correct4 failed");
    run_file("examples/correct5.lambda", OutputValue::Nat(0))
        .expect("Integration test for correct5 failed");

    run_file("examples/arrowtype.lambda", OutputValue::Nat(1))
        .expect("Integration test for arrowtype failed");
    run_file("examples/high-order.lambda", OutputValue::Nat(2))
        .expect("Integration test for high-order failed");
}
