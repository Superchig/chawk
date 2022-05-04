use chawk::{ChawkParser, Rule};
use pest::Parser;
use std::fs;

fn main() {
    let input_file = std::env::args().nth(1).unwrap();
    let unparsed_file = fs::read_to_string(&input_file).expect("Cannot read file");

    let program = ChawkParser::parse(Rule::Program, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap();

    println!("{:#?}", program);
}
