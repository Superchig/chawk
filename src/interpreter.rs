use std::{
    collections::HashMap,
    fmt::Display,
    io::BufRead,
    mem::swap,
    ops::{Add, Div, Mul, Rem, Sub},
};

use crate::{
    ast::{Block, Expression, Id, Pattern, PatternBlock, PrintStatement, Statement},
    InitClause,
};
use crate::{parser::parse, FunctionDef};

pub struct Interpreter {
    pub curr_columns: Vec<String>,
    pub curr_line: String,
    pub global_vars: HashMap<Id, Value>,
    pub local_vars: Vec<HashMap<Id, Value>>,
    pub function_defs: HashMap<Id, FunctionDef>,
}

impl Interpreter {
    // FIXME(Chris): Implement function definitions and function calls
    pub fn run(&mut self, program_str: &str, records_reader: &mut dyn BufRead) {
        let program_ast = parse(program_str).unwrap();

        // Copy function definitions over to pseudo-global interpreter state
        self.function_defs.clone_from(&program_ast.function_defs);

        // Execute BEGIN blocks
        for pattern_block in &program_ast.pattern_blocks {
            if let Some(crate::Pattern::Begin) = pattern_block.pattern {
                if let Some(block) = &pattern_block.block {
                    let return_value = self.execute_block(block);

                    // TODO(Chris): Implement better error handling for return statements outside
                    // of functions
                    if return_value.is_some() {
                        panic!("Used a return statement outside of a function");
                    }
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
                    let return_value = self.execute_block(block);

                    if return_value.is_some() {
                        panic!("Used a return statement outside of a function");
                    }
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
                let return_value = self.execute_block(block);

                if return_value.is_some() {
                    panic!("Used a return statement outside of a function");
                }
            } else {
                println!("{}", self.curr_line);
            }
        }
    }

    /// Returns an optional "return" value from within a function
    fn execute_block(&mut self, block: &Block) -> Option<Value> {
        self.local_vars.push(HashMap::new());

        let mut return_value = None;

        for statement in &block.statements {
            if let Some(stm_return_value) = self.execute_statement(statement) {
                return_value = Some(stm_return_value);
                break;
            }
        }

        self.local_vars.pop();

        return_value
    }

    /// Returns an optional "return" value from within a function
    fn execute_statement(&mut self, statement: &Statement) -> Option<Value> {
        match statement {
            Statement::PrintStatement(PrintStatement { expression }) => {
                let expression_value = self.eval_exp(expression);

                println!("{}", expression_value);
            }
            Statement::LocalVarStatement {
                id,
                initial_expression,
            } => {
                let initial_value = if let Some(expr) = initial_expression {
                    self.eval_exp(expr)
                } else {
                    Value::String("".to_string())
                };

                let context = self
                    .local_vars
                    .last_mut()
                    .expect("No local context available");

                if context.contains_key(id) {
                    panic!(
                        "Tried to declare a local variable that already existed: {}",
                        id
                    );
                }

                context.insert(id.clone(), initial_value);
            }
            Statement::ExpressionStatement(expression) => {
                self.eval_exp(expression);
            }
            Statement::BlockStatement(other_block) => {
                let return_value = self.execute_block(other_block);

                if return_value.is_some() {
                    return return_value;
                }
            }
            Statement::IfStatement {
                condition,
                true_statement,
                false_statement,
            } => {
                let cond_value = self.eval_exp(condition);

                let mut return_value = None;

                self.local_vars.push(HashMap::new());

                if cond_value.to_bool() {
                    return_value = self.execute_statement(true_statement);
                } else if let Some(false_statement) = false_statement {
                    return_value = self.execute_statement(false_statement);
                }

                self.local_vars.pop();

                return return_value;
            }
            Statement::WhileStatement { condition, body } => {
                let mut return_value;
                let mut cond_bool;
                loop {
                    cond_bool = self.eval_exp(condition).to_bool();

                    if !cond_bool {
                        break;
                    }

                    self.local_vars.push(HashMap::new());

                    return_value = self.execute_statement(body);

                    self.local_vars.pop();

                    if return_value.is_some() {
                        return return_value;
                    }
                }
            }
            Statement::ForStatement {
                init_clause,
                condition_expression,
                iteration_expression,
                body,
            } => {
                // NOTE(Chris): This is mostly based on the specification for a `for` loop provided
                // at https://en.cppreference.com/w/c/language/for

                let mut return_value;

                match init_clause {
                    Some(InitClause::Expression(expr)) => {
                        self.eval_exp(expr);
                    }
                    Some(InitClause::Declaration(decl_statement)) => {
                        self.local_vars.push(HashMap::new());

                        // NOTE(Chris): This should only be a local variable declaration statement,
                        // so we should be able to ignore the return value (which represents an
                        // awk-function's possible return value)
                        _ = self.execute_statement(decl_statement);
                    }
                    None => (),
                }

                let mut cond_bool;

                loop {
                    if let Some(condition_expression) = condition_expression {
                        cond_bool = self.eval_exp(condition_expression).to_bool();
                    } else {
                        cond_bool = true;
                    }

                    if !cond_bool {
                        break;
                    }

                    self.local_vars.push(HashMap::new());

                    return_value = self.execute_statement(body);

                    self.local_vars.pop();

                    if return_value.is_some() {
                        return return_value;
                    }

                    if let Some(iteration_expression) = iteration_expression {
                        self.eval_exp(iteration_expression);
                    }
                }

                if let Some(InitClause::Declaration(_decl_statement)) = init_clause {
                    self.local_vars.pop();
                }
            }
            Statement::ReturnStatement(expression) => {
                let value = self.eval_exp(expression);

                return Some(value);
            }
        }

        None
    }

    fn eval_exp(&mut self, expression: &Expression) -> Value {
        match expression {
            Expression::String { value } => Value::String(value.clone()),
            Expression::ColumnNumber(num) => {
                Value::String(if self.curr_columns.len() >= *num as usize {
                    if *num == 0 {
                        self.curr_line.clone()
                    } else {
                        let col_index = num - 1;

                        self.curr_columns[col_index as usize].to_string()
                    }
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
            Expression::Modulo(expr_left, expr_right) => {
                self.apply_arith(expr_left, Rem::rem, expr_right)
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
            Expression::LessThan(expr_left, expr_right) => {
                self.apply_cmp(expr_left, expr_right, f64::lt, String::lt)
            }
            Expression::LessEqual(expr_left, expr_right) => {
                self.apply_cmp(expr_left, expr_right, f64::le, String::le)
            }
            Expression::NotEqual(expr_left, expr_right) => {
                self.apply_cmp(expr_left, expr_right, f64::ne, String::ne)
            }
            Expression::Equals(expr_left, expr_right) => {
                self.apply_cmp(expr_left, expr_right, f64::eq, String::eq)
            }
            Expression::GreaterThan(expr_left, expr_right) => {
                self.apply_cmp(expr_left, expr_right, f64::gt, String::gt)
            }
            Expression::GreaterEqual(expr_left, expr_right) => {
                self.apply_cmp(expr_left, expr_right, f64::ge, String::ge)
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
            Expression::RegexMatch(expr_left, expr_right) => {
                Value::from_bool(self.apply_regex_from_right(expr_left, expr_right))
            }
            Expression::RegexNotMatch(expr_left, expr_right) => {
                Value::from_bool(!self.apply_regex_from_right(expr_left, expr_right))
            }
            Expression::FunctionCall { name, arguments } => {
                let function_def = if let Some(function_def) = self.function_defs.get(name) {
                    // TODO(Chris): Initialize function definitions with once_cell to avoid cloning
                    // here
                    function_def.clone()
                } else {
                    // TODO(Chris): Implement better error msg for undefined function
                    panic!("Tried to call undefined function: {}", name);
                };

                let mut new_context = HashMap::new();

                for (i, arg) in arguments.iter().enumerate() {
                    let param_name = if let Some(param_name) = function_def.parameters.get(i) {
                        param_name
                    } else {
                        // TODO(Chris): Implement better error msg for too many function arguments
                        panic!(
                            "Too many arguments to function: {} has {} parameters, but {} arguments were used.",
                             name,
                             function_def.parameters.len(),
                             arguments.len()
                        );
                    };

                    let value = self.eval_exp(arg);

                    new_context.insert(param_name.clone(), value);
                }

                let mut function_vars = vec![new_context];

                swap(&mut function_vars, &mut self.local_vars);

                let return_value = self.execute_block(&function_def.body);

                swap(&mut function_vars, &mut self.local_vars);

                // TODO(Chris): Return the empty string rather than FALSE_VALUE once you have a
                // universal empty string value
                return_value.unwrap_or(FALSE_VALUE)
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

    fn apply_cmp(
        &mut self,
        expr_left: &Expression,
        expr_right: &Expression,
        cmp_float: impl Fn(&f64, &f64) -> bool,
        cmp_string: impl Fn(&String, &String) -> bool,
    ) -> Value {
        let value_left = self.eval_exp(expr_left);
        let value_right = self.eval_exp(expr_right);

        Value::from_bool(match (&value_left, &value_right) {
            (Value::String(string_left), Value::String(string_right)) => {
                cmp_string(string_left, string_right)
            }
            (Value::Num(num_left), Value::Num(num_right)) => cmp_float(num_left, num_right),
            _ => cmp_string(&value_left.to_string(), &value_right.to_string()),
        })
    }

    fn apply_regex_from_right(&mut self, expr_left: &Expression, expr_right: &Expression) -> bool {
        if let Expression::Regex(_) = expr_left {
            eprintln!("WARNING: regular expression on the left of `~` or `!~` operator");
            return false;
        }

        let value_left = self.eval_exp(expr_left);

        if let Expression::Regex(regex) = expr_right {
            regex.is_match(&value_left.to_string())
        } else {
            // TODO(Chris): Treat this as a full regex by converting the corresponding
            // string value into a regex in the parser

            let value_right = self.eval_exp(expr_right);
            value_left.to_string().contains(&value_right.to_string())
        }
    }

    // NOTE(Chris): Uninitialized variables have a default value of the empty string, allowing for
    // uses like `sum += 1` without prior references to a `sum` variable.
    fn lookup(&mut self, id: &Id) -> &mut Value {
        let containing_context = self
            .local_vars
            .iter_mut()
            .rev()
            .find(|context| context.contains_key(id));

        if let Some(containing_context) = containing_context {
            containing_context.get_mut(id).unwrap()
        } else if self.global_vars.contains_key(id) {
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

const TRUE_VALUE: Value = Value::Num(1.0);
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
