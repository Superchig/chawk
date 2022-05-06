use chawk::Interpreter;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    process::exit,
};

use clap::{arg, command};

fn main() {
    let mut command_cli = command!()
        .arg(arg!([argument]).multiple_occurrences(true))
        .arg(arg!(-f <progfile>).required(false));

    // Store help text before obtaining matches, which consumes command_cli
    let mut help_text = Vec::new();
    command_cli.write_help(&mut help_text).unwrap();

    let matches = command_cli.get_matches();

    let mut positional_arguments: Vec<&str> = if let Some(arguments) = matches.values_of("argument")
    {
        arguments.collect()
    } else {
        vec![]
    };

    if positional_arguments.is_empty() && atty::is(atty::Stream::Stdin) {
        let mut stdout = io::stdout();
        stdout.write_all(&help_text).unwrap();
        exit(1);
    }

    // Obtain the text of the awk program
    let unparsed_file = if let Some(progfile) = matches.value_of("progfile") {
        fs::read_to_string(progfile).expect("Cannot read progfile")
    } else if positional_arguments.is_empty() {
        eprintln!("No awk program text was specified.");

        exit(1);
    } else {
        let program_text = positional_arguments.remove(0);
        program_text.to_string()
    };

    // Obtain the input for the records (file vs stdin)
    let mut records_reader: Box<dyn BufRead> = if positional_arguments.is_empty() {
        let stdin = io::stdin();
        Box::new(BufReader::new(stdin))
    } else {
        let file = File::open(&positional_arguments[0]).expect("Cannot read records file");
        Box::new(BufReader::new(file))
    };

    let mut interpreter = Interpreter {
        curr_columns: vec![],
        curr_line: String::new(),
        global_vars: HashMap::new(),
        local_vars: vec![],
    };

    interpreter.run(&unparsed_file, &mut records_reader);
}
