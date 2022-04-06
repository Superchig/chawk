use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    process::exit,
};

use chawk::{Expression, PatternBlock, PrintStatement, Statement};
use clap::{arg, command};

fn main() {
    let matches = command!()
        .arg(arg!([argument]).multiple_occurrences(true))
        .arg(arg!(-f <progfile>).required(false))
        .get_matches();

    let mut positional_arguments: Vec<&str> = if let Some(arguments) = matches.values_of("argument")
    {
        arguments.collect()
    } else {
        vec![]
    };

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
    };

    interpreter.run(&unparsed_file, &mut records_reader);
}

struct Interpreter {
    curr_columns: Vec<String>,
}

impl Interpreter {
    fn run(&mut self, program_str: &str, records_reader: &mut dyn BufRead) {
        let program_ast = chawk::parse(program_str).unwrap();

        for line in records_reader.lines() {
            // TODO(Chris): Handle cases where UTF-8 doesn't parse correctly
            let line = line.unwrap();
            let chars: Vec<char> = line.chars().collect();

            self.curr_columns.clear();

            let mut prev_ch = '\0';
            for ch in chars {
                if (prev_ch.is_ascii_whitespace() || self.curr_columns.is_empty()) && ch != ' ' {
                    self.curr_columns.push(String::new());
                    // TODO(Chris): Refactor this (and memory allocation) into its own type, with its
                    // own method
                    let columns_len = self.curr_columns.len();
                    self.curr_columns[columns_len - 1].push(ch);
                } else if ch != ' ' {
                    let columns_len = self.curr_columns.len();
                    self.curr_columns[columns_len - 1].push(ch);
                }

                prev_ch = ch;
            }

            self.eval_pattern_blocks(&program_ast.pattern_blocks);
        }
    }

    fn eval_pattern_blocks(&mut self, pattern_blocks: &[PatternBlock]) {
        for pattern_block in pattern_blocks {
            // FIXME(Chris): Implement regex matching in pattern

            if let Some(block) = &pattern_block.block {
                for statement in &block.statements {
                    match statement {
                        Statement::PrintStatement(PrintStatement { expression }) => {
                            let expression_value = self.eval_exp(expression);

                            println!("{}", expression_value);
                        }
                    }
                }
            }
        }
    }

    fn eval_exp(&mut self, expression: &Expression) -> Value {
        match expression {
            Expression::String { value } => Value::String(value.clone()),
            Expression::ColumnNumber(num) => {
                Value::String(if self.curr_columns.len() >= *num as usize {
                    let col_index = num - 1;

                    // TODO(Chris): Use usize for num in the firstplace
                    self.curr_columns[col_index as usize].to_string()
                } else {
                    "".to_string()
                })
            }
        }
    }
}

enum Value {
    String(String),
    #[allow(dead_code)]
    Integer(i64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            Value::Integer(num) => write!(f, "{}", num),
        }
    }
}
