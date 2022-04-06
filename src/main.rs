use std::{
    fmt::Display,
    fs,
    io::{self, BufRead, BufReader},
};

use chawk::{Expression, PatternBlock, PrintStatement, Statement};

fn main() {
    let mut interpreter = Interpreter {
        curr_columns: vec![],
    };

    interpreter.run();
}

struct Interpreter {
    curr_columns: Vec<String>,
}

impl Interpreter {
    fn run(&mut self) {
        let unparsed_file =
            fs::read_to_string("examples/print_line.awk").expect("Cannot read file");

        let program_ast = chawk::parse(&unparsed_file).unwrap();

        let stdin = io::stdin();
        let stdin_reader = BufReader::new(stdin);

        for line in stdin_reader.lines() {
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
            // FIXME(Chris): Actually return the value in the correct column, or nothing, if
            // nothing exists
            Expression::ColumnNumber(num) => Value::Integer(*num),
        }
    }
}

enum Value {
    String(String),
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
