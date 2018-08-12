extern crate lambda_rs;
extern crate pest;

use lambda_rs::ast::*;
use lambda_rs::parser::*;
use pest::iterators::Pair;
use std::env;

use std::process;

fn main() {
    // Setup environment
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("This interpreter takes exactly one argument: the filename of the lambda code");
        process::exit(1);
    }

    // Read file contents
    let filename = args[1].clone();
    let contents = lambda_rs::read_file(&filename).unwrap_or_else(|e| {
        println!("Encountered an error when reading file: {}", e);
        process::exit(1);
    });

    // Parse file
    let pairs = parse_file(&contents).unwrap_or_else(|e| {
        println!("Encountered an error when parsing file:\n{}", e);
        process::exit(1);
    });

    /* DEBUG
    for pair in pairs.clone() {
        recursive_print(pair, 0);
    }*/

    // Build the Abstract Syntax Tree
    let ast_tree = build_ast(pairs);

    // DEBUG
    // println!("{}", ast_tree);

    // Perform typechecking on the syntax tree
    let _tree_type = ast_tree.check::<i32>().unwrap_or_else(|e| {
        println!("Encountered an error when typechecking:\n{}", e);
        process::exit(1);
    });

    // Evaluate the Abstract Syntax tree
    println!("{}", ast_tree.eval());
}

/// Debug method to print result of parsing
///
/// # Arguments
/// * `pair` - Structure used by parser
/// * `level` - Variable to control indentation, so inner blocks get printed nicely indented
fn recursive_print(pair: Pair<'_, Rule>, level: usize) {
    let span = pair.clone().into_span();
    println!("{}Rule:    {:?}", "\t".repeat(level), pair.as_rule());
    println!(
        "{}Text:    {}",
        "\t".repeat(level),
        span.as_str().replace("\n", " ")
    );

    for inner_pair in pair.into_inner() {
        recursive_print(inner_pair, level + 1);
    }
}
