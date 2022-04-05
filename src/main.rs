#[macro_use]
extern crate pest_derive;

use std::fs;

use pest::Parser;

#[derive(Parser)]
#[grammar = "chawk.pest"]
pub struct CSVParser;

fn main() {
    let unparsed_file = fs::read_to_string("examples/print_line.awk").expect("Cannot read file");

    let program = CSVParser::parse(Rule::Program, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `file` rule; never fails
    
    dbg!(program);
}
