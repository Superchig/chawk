use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    process::exit, ops::{Mul, Div, Sub, Add},
};

use chawk::{Expression, Id, PatternBlock, PrintStatement, Statement};
use clap::{arg, command};

// FIXME(Chris): Automate testing of input awk files and data files

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
        global_vars: HashMap::new(),
    };

    interpreter.run(&unparsed_file, &mut records_reader);
}

struct Interpreter {
    // FIXME(Chris): Implement local variables (scope)
    // FIXME(Chris): Implement automatic test suite capable of comparing the output of chawk to the
    // output of awk (or gawk)
    // FIXME(Chris): Test out pest Separator with ";"
    curr_columns: Vec<String>,
    global_vars: HashMap<Id, Value>,
}

impl Interpreter {
    // FIXME(Chris): Implement if-then-else statements with scope
    // FIXME(Chris): Implement while loop statements with scope
    // FIXME(Chris): Implement for loop statements with scope
    fn run(&mut self, program_str: &str, records_reader: &mut dyn BufRead) {
        let program_ast = chawk::parse(program_str).unwrap();

        for line in records_reader.lines() {
            // TODO(Chris): Handle cases where UTF-8 doesn't parse correctly
            let line = line.unwrap();
            let chars: Vec<char> = line.chars().collect();

            self.curr_columns.clear();

            let mut prev_ch = '\0';
            for ch in chars {
                if (prev_ch.is_ascii_whitespace() || self.curr_columns.is_empty()) && !ch.is_ascii_whitespace() {
                    self.curr_columns.push(String::new());
                    // TODO(Chris): Refactor this (and memory allocation) into its own type, with its
                    // own method
                    let columns_len = self.curr_columns.len();
                    self.curr_columns[columns_len - 1].push(ch);
                } else if !ch.is_ascii_whitespace() {
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
                        Statement::AssignStatement { id, expression } => {
                            let expression_value = self.eval_exp(expression);

                            let var_value = self.lookup(id);

                            *var_value = expression_value;
                        } // FIXME(Chris): Implement addition and addition assignment statement
                    }
                }
            }
        }
    }

    // FIXME(Chris): Implement string concatenation
    // FIXME(Chris): Implement built-in NR variable
    fn eval_exp(&mut self, expression: &Expression) -> Value {
        match expression {
            Expression::String { value } => Value::String(value.clone()),
            Expression::ColumnNumber(num) => {
                Value::String(if self.curr_columns.len() >= *num as usize {
                    let col_index = num - 1;

                    self.curr_columns[col_index as usize].to_string()
                } else {
                    "".to_string()
                })
            }
            Expression::VarLookup(var_id) => self.lookup(var_id).clone(),
            Expression::Plus(expr_left, expr_right) => {
                self.apply_arith(expr_left, Add::add, expr_right)
            }
            Expression::Minus(expr_left, expr_right) => {
                self.apply_arith(expr_left, Sub::sub, expr_right)
            }
            Expression::Times(expr_left, expr_right) => {
                self.apply_arith(expr_left, Mul::mul, expr_right)
            }
            Expression::Div(expr_left, expr_right) => {
                self.apply_arith(expr_left, Div::div, expr_right)
            }
            Expression::Num(num) => Value::Num(*num),
        }
    }

    fn apply_arith(
        &mut self,
        expr_left: &Expression,
        f: impl Fn(f64, f64) -> f64,
        expr_right: &Expression,
    ) -> Value {
        Value::Num(f(self.eval_exp(expr_left).to_num(), self.eval_exp(expr_right).to_num()))
    }

    // NOTE(Chris): Uninitialized variables have a default value of the empty string, allowing for
    // uses like `sum += 1` without prior references to a `sum` variable.
    fn lookup(&mut self, id: &Id) -> &mut Value {
        if self.global_vars.contains_key(id) {
            self.global_vars.get_mut(id).unwrap()
        } else {
            self.global_vars
                .insert(id.clone(), Value::String(String::new()));
            self.global_vars.get_mut(id).unwrap()
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Num(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            Value::Num(num) => write!(f, "{}", num),
        }
    }
}

impl Value {
    fn to_num(&self) -> f64 {
        match self {
            Value::String(string) => string.parse::<f64>().unwrap_or(0.0),
            Value::Num(num) => *num,
        }
    }
}
