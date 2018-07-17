extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::Parser;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::process;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LambdaParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("This interpreter takes exactly one argument: the filename of the lambda code");
        process::exit(1);
    }
    let filename = args[1].clone();
    let contents = read_file(&filename).unwrap_or_else(|e| {
        println!("Problem when reading file: {}", e);
        process::exit(1);
    });
    let pairs = LambdaParser::parse(Rule::program, &contents).unwrap_or_else(|e| {
        println!("Problem when parsing file: {}", e);
        process::exit(1);
    });

    for pair in pairs {
        recursive_print(pair, 0);
    }
}

fn read_file(path: &str) -> Result<String, Box<Error>> {
    let mut f = File::open(path)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

fn recursive_print(pair: Pair<'_, Rule>, level: usize) {
    let span = pair.clone().into_span();
    println!("{}Rule:    {:?}", "\t".repeat(level), pair.as_rule());
    println!("{}Text:    {}", "\t".repeat(level), span.as_str());

    for inner_pair in pair.into_inner() {
        recursive_print(inner_pair, level + 1);
    }
}
