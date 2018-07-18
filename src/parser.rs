use pest::iterators::Pairs;
use pest::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LambdaParser;

pub fn parse_file(contents: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    LambdaParser::parse(Rule::program, contents)
}
