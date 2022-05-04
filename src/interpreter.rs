use std::{
    collections::HashMap,
    fmt::Display,
    io::BufRead,
    ops::{Add, Div, Mul, Sub},
};

use crate::ast::{Block, Expression, Id, Pattern, PatternBlock, PrintStatement, Statement};
use crate::parser::parse;

pub struct Interpreter {
    // FIXME(Chris): Implement local variables (scope)
    // FIXME(Chris): Implement automatic test suite capable of comparing the output of chawk to the
    // output of awk (or gawk)
    // FIXME(Chris): Test out pest Separator with ";"
    pub curr_columns: Vec<String>,
    pub curr_line: String,
    pub global_vars: HashMap<Id, Value>,
}

impl Interpreter {
    // FIXME(Chris): Implement if-then-else statements with scope
    // FIXME(Chris): Implement while loop statements with scope
    // FIXME(Chris): Implement for loop statements with scope
    pub fn run(&mut self, program_str: &str, records_reader: &mut dyn BufRead) {
        let program_ast = parse(program_str).unwrap();

        // Execute BEGIN blocks
        for pattern_block in &program_ast.pattern_blocks {
            if let Some(crate::Pattern::Begin) = pattern_block.pattern {
                if let Some(block) = &pattern_block.block {
                    self.execute_block(block);
                } else {
                    // This is required by the POSIX standard. Though we don't need to support the
                    // standard, it could be useful in this case.
                    panic!("BEGIN block must have an associated action.");
                }
            }
        }

        let mut curr_line_num = 0.0;

        for line in records_reader.lines() {
            // TODO(Chris): Handle cases where UTF-8 doesn't parse correctly
            self.curr_line = line.unwrap();
            let chars: Vec<char> = self.curr_line.chars().collect();

            self.curr_columns.clear();

            let mut prev_ch = '\0';
            for ch in chars {
                if (prev_ch.is_ascii_whitespace() || self.curr_columns.is_empty())
                    && !ch.is_ascii_whitespace()
                {
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

            curr_line_num += 1.0;
            // TODO(Chris): Use the once_cell library to only create the NR string once
            let nr_variable = self.lookup(&Id("NR".to_string()));
            *nr_variable = Value::Num(curr_line_num);

            self.eval_pattern_blocks(&program_ast.pattern_blocks);
        }

        // Execute END blocks
        for pattern_block in &program_ast.pattern_blocks {
            if let Some(crate::Pattern::End) = pattern_block.pattern {
                if let Some(block) = &pattern_block.block {
                    self.execute_block(block);
                } else {
                    panic!("END block must have an associated action.");
                }
            }
        }
    }

    fn eval_pattern_blocks(&mut self, pattern_blocks: &[PatternBlock]) {
        for pattern_block in pattern_blocks {
            if let Some(pattern) = &pattern_block.pattern {
                match pattern {
                    Pattern::Expression(expression) => {
                        let value = self.eval_exp(expression);
                        if !value.to_bool() {
                            continue;
                        }
                    }
                    Pattern::Begin | Pattern::End => continue,
                }
            }

            if let Some(block) = &pattern_block.block {
                self.execute_block(block);
            } else {
                println!("{}", self.curr_line);
            }
        }
    }

    fn execute_block(&mut self, block: &Block) {
        for statement in &block.statements {
            match statement {
                Statement::PrintStatement(PrintStatement { expression }) => {
                    let expression_value = self.eval_exp(expression);

                    println!("{}", expression_value);
                }
                Statement::ExpressionStatement(expression) => {
                    self.eval_exp(expression);
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
            Expression::Concatenate(expr_left, expr_right) => {
                let value_left = self.eval_exp(expr_left);
                let value_right = self.eval_exp(expr_right);

                let string_result = match (&value_left, &value_right) {
                    (Value::String(string_left), Value::String(string_right)) => {
                        let mut string_result = String::new();
                        string_result.push_str(string_left);
                        string_result.push_str(string_right);
                        string_result
                    }
                    _ => {
                        format!("{}{}", &value_left, &value_right)
                    }
                };

                Value::String(string_result)
            }
            Expression::Equals(expr_left, expr_right) => {
                let value_left = self.eval_exp(expr_left);
                let value_right = self.eval_exp(expr_right);

                Value::from_bool(match (&value_left, &value_right) {
                    (Value::String(string_left), Value::String(string_right)) => {
                        string_left == string_right
                    }
                    (Value::Num(num_left), Value::Num(num_right)) => num_left == num_right,
                    _ => value_left.to_string() == value_right.to_string(),
                })
            }
            Expression::Regex(regex) => {
                // According to the POSIX standard, we treat the regex expression /ere/ as the
                // equivalent of $0 ~ /ere/, unless it's the right-hand of `~`, `!~`, or used as an
                // argument to the built-in gsub, match, and sub functions.
                Value::from_bool(regex.is_match(&self.curr_line))
            }
            Expression::Assign(id, rhs_expression) => {
                let expression_value = self.eval_exp(rhs_expression);

                let var_value = self.lookup(id);

                *var_value = expression_value.clone();

                expression_value
            }
            Expression::PlusAssign(id, rhs_expression) => {
                let var_value_num = self.lookup(id).to_num();
                let expression_value_num = self.eval_exp(rhs_expression).to_num();

                // NOTE(Chris): We call lookup() a second time to avoid mutably borrowing self
                // twice
                let var_value = self.lookup(id);

                *var_value = Value::Num(var_value_num + expression_value_num);

                var_value.clone()
            }
        }
    }

    fn apply_arith(
        &mut self,
        expr_left: &Expression,
        f: impl Fn(f64, f64) -> f64,
        expr_right: &Expression,
    ) -> Value {
        Value::Num(f(
            self.eval_exp(expr_left).to_num(),
            self.eval_exp(expr_right).to_num(),
        ))
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
pub enum Value {
    String(String),
    Num(f64),
}

// FIXME(Chris): Remove these dead_code attributes
#[allow(dead_code)]
const TRUE_VALUE: Value = Value::Num(1.0);
#[allow(dead_code)]
const FALSE_VALUE: Value = Value::Num(0.0);

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(string) => write!(f, "{}", string),
            // The default float format for awk is [-]D.DDDDDD, according to
            // https://en.wikibooks.org/wiki/An_Awk_Primer/Output_with_print_and_printf
            // Also, according to the POSIX standard, OFMT is "%.6g" by default
            Value::Num(num) => {
                // NOTE(Chris): This code fails to avoid trailing zeros
                // write!(f, "{:.6}", num)

                // Modified from
                // https://stackoverflow.com/questions/59506403/how-to-format-a-float-without-trailing-zeros-in-rust
                const DECIMAL_FORMAT: f64 = 10_000.0;

                let rounded = (num * DECIMAL_FORMAT).round() / DECIMAL_FORMAT;
                write!(f, "{}", rounded)
            }
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

    fn to_bool(&self) -> bool {
        match self {
            Value::String(string) => !string.is_empty(),
            Value::Num(num) => num != &0.0,
        }
    }

    fn from_bool(possible_val: bool) -> Self {
        if possible_val {
            TRUE_VALUE
        } else {
            FALSE_VALUE
        }
    }
}
