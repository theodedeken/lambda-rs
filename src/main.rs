extern crate lambda_rs;
extern crate pest;

use lambda_rs::ast::*;
use lambda_rs::parser::*;
use pest::iterators::Pair;
use std::env;

use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("This interpreter takes exactly one argument: the filename of the lambda code");
        process::exit(1);
    }
    let filename = args[1].clone();
    let contents = lambda_rs::read_file(&filename).unwrap_or_else(|e| {
        println!("Problem when reading file: {}", e);
        process::exit(1);
    });
    let pairs = parse_file(&contents).unwrap_or_else(|e| {
        println!("Problem when parsing file: {}", e);
        process::exit(1);
    });

    for pair in pairs.clone() {
        recursive_print(pair, 0);
    }

    //build ast
    let ast_tree = build_ast(pairs);

    ast_tree.print();
    //check ast
    let _tree_type = ast_tree.check().unwrap_or_else(|e| {
        println!("Problem when type checking: {}", e);
        process::exit(1);
    });

    //evaluate ast
    println!("{}", ast_tree.eval());
}

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
