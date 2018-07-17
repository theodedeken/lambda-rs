extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LambdaParser;

fn main() {
    let pairs = LambdaParser::parse(Rule::program, "(λ a: Nat. succ succ 0) iszero true")
        .unwrap_or_else(|e| panic!("{}", e));

    // Because ident_list is silent, the iterator will contain idents
    for pair in pairs {
        recursive_print(pair, 0);
    }
}

fn recursive_print(pair: Pair<'_, Rule>, level: usize) {
    let span = pair.clone().into_span();
    // A pair is a combination of the rule which matched and a span of input
    print!("{}", "\t".repeat(level));
    println!("Rule:    {:?}", pair.as_rule());
    print!("{}", "\t".repeat(level));
    //println!("Span:    {:?}", span);
    println!("Text:    {}", span.as_str());

    // A pair can be converted to an iterator of the tokens which make it up:
    for inner_pair in pair.into_inner() {
        recursive_print(inner_pair, level + 1);
        /*
        let inner_span = inner_pair.clone().into_span();
        match inner_pair.as_rule() {
            Rule::alpha => println!("Letter:  {}", inner_span.as_str()),
            Rule::digit => println!("Digit:   {}", inner_span.as_str()),
            _ => unreachable!(),
        };*/
    }
}
