use chawk::{ChawkParser, Program, Rule};
use from_pest::FromPest;
use pest::Parser;
use std::{
    fs,
    io::{self, BufRead, BufReader},
};

fn main() {
    let stdin = io::stdin();
    let stdin_reader = BufReader::new(stdin);

    let mut curr_columns = vec![];

    for line in stdin_reader.lines() {
        // TODO(Chris): Handle cases where UTF-8 doesn't parse correctly
        let line = line.unwrap();
        let chars: Vec<char> = line.chars().collect();

        curr_columns.clear();

        let mut prev_ch = '\0';
        for ch in chars {
            if (prev_ch.is_ascii_whitespace() || curr_columns.is_empty()) && ch != ' ' {
                curr_columns.push(String::new());
                // TODO(Chris): Refactor this (and memory allocation) into its own type, with its
                // own method
                let columns_len = curr_columns.len();
                curr_columns[columns_len - 1].push(ch);
            } else if ch != ' ' {
                let columns_len = curr_columns.len();
                curr_columns[columns_len - 1].push(ch);
            }

            prev_ch = ch;
        }

        if !curr_columns.is_empty() {
            println!("curr_columns: {:?}", &curr_columns);
        }
    }

    let unparsed_file = fs::read_to_string("examples/print_line.awk").expect("Cannot read file");

    // let program = ChawkParser::parse(Rule::Program, &unparsed_file)
    //     .expect("unsuccessful parse") // unwrap the parse result
    //     .next()
    //     .unwrap();

    // println!("{:#?}", program);

    let program_ast = chawk::parse(&unparsed_file).unwrap();
    println!("{:#?}", &program_ast);
}
