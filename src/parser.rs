use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;
use regex::Regex;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "chawk.pest"]
pub struct ChawkParser;

macro_rules! panic_unexpected_rule {
    ($value:expr) => {
        panic!("Unexpected parsing rule: {:?}", $value)
    };
}

pub fn parse(source: &str) -> Result<Program, Error<Rule>> {
    let mut program = Program {
        pattern_blocks: vec![],
    };

    let mut pairs = ChawkParser::parse(Rule::Program, source)?;

    let start = pairs.next().unwrap();

    for pair in start.into_inner() {
        match pair.as_rule() {
            Rule::Program => (),
            Rule::PatternBlock => {
                program.pattern_blocks.push(build_pattern_block(pair));
            }
            Rule::EOI => (),
            _ => panic_unexpected_rule!(pair),
        }
    }

    Ok(program)
}

fn build_pattern_block(pair: Pair<Rule>) -> PatternBlock {
    let mut pattern_block = PatternBlock {
        pattern: None,
        block: None,
    };

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Pattern => {
                pattern_block.pattern = Some(build_pattern(pair));
            }
            Rule::Block => {
                pattern_block.block = Some(build_block(pair));
            }
            _ => panic_unexpected_rule!(pair),
        }
    }

    pattern_block
}

fn build_pattern(pair: Pair<Rule>) -> Pattern {
    assert_eq!(pair.as_rule(), Rule::Pattern);

    let span = pair.as_str();

    if let Some(inner_pair) = pair.into_inner().next() {
        match inner_pair.as_rule() {
            Rule::Expression => Pattern::Expression(build_expression(inner_pair)),
            _ => panic_unexpected_rule!(inner_pair),
        }
    } else {
        match span {
            "BEGIN" => Pattern::Begin,
            "END" => Pattern::End,
            _ => unreachable!(),
        }
    }
}

fn build_regex(pair: Pair<Rule>) -> Regex {
    assert_eq!(pair.as_rule(), Rule::Regex);

    let span = pair.as_str();
    let regex_str = &span[1..span.len() - 1];

    Regex::new(regex_str).expect("Unable to compile regex")
}

fn build_block(pair: Pair<Rule>) -> Block {
    assert_eq!(pair.as_rule(), Rule::Block);

    let mut block = Block { statements: vec![] };

    for stm_pair in pair.into_inner() {
        block.statements.push(build_statement(stm_pair));
    }

    block
}

fn build_statement(pair: Pair<Rule>) -> Statement {
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::PrintStatement => {
                let mut inner_iter = inner_pair.into_inner();
                let expression_pair = inner_iter.next().unwrap();
                let expression = build_expression(expression_pair);

                return Statement::PrintStatement(PrintStatement { expression });
            }
            Rule::LocalVarStatement => {
                return build_local_var_statement(inner_pair);
            }
            Rule::IfStatement => {
                return build_if_statement(inner_pair);
            }
            Rule::ExpressionStatement => {
                let inner_expression_pair = inner_pair.into_inner().next().expect("No inner pair");

                return Statement::ExpressionStatement(build_expression(inner_expression_pair));
            }
            Rule::Block => {
                return Statement::BlockStatement(build_block(inner_pair));
            }
            _ => panic_unexpected_rule!(inner_pair),
        }
    }

    unreachable!()
}

fn build_local_var_statement(pair: Pair<Rule>) -> Statement {
    assert_eq!(pair.as_rule(), Rule::LocalVarStatement);

    let mut inner_pairs = pair.into_inner();

    let id = build_id(inner_pairs.next().expect("No more pairs"));

    let possible_expression = inner_pairs.next().map(build_expression);

    Statement::LocalVarStatement {
        id,
        initial_expression: possible_expression,
    }
}

fn build_if_statement(pair: Pair<Rule>) -> Statement {
    assert_eq!(pair.as_rule(), Rule::IfStatement);

    let mut inner_pairs = pair.into_inner();

    let condition = build_expression(inner_pairs.next().expect("No more pairs"));

    let true_statement = Box::new(build_statement(inner_pairs.next().expect("No more pairs")));

    let false_statement = inner_pairs.next().map(|p| Box::new(build_statement(p)));

    Statement::IfStatement {
        condition,
        true_statement,
        false_statement,
    }
}

fn build_expression(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression);

    let inner_pair = pair.into_inner().next().expect("No pair inside rule");

    match inner_pair.as_rule() {
        Rule::Expression1 => build_expression1(inner_pair),
        _ => panic_unexpected_rule!(inner_pair),
    }
}

fn build_expression1(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression1);

    let mut inner_pairs: Vec<Pair<Rule>> = pair.into_inner().collect();

    assert!(!inner_pairs.is_empty());

    if inner_pairs.len() == 1 {
        let inner_pair = inner_pairs.pop().expect("inner_pairs is empty");

        build_expression2(inner_pair)
    } else {
        assert!(inner_pairs.len() == 3);

        let rhs_expression = build_expression2(inner_pairs.pop().expect("Ran out of pairs"));
        let rule_sign = {
            let inner_pair = inner_pairs.pop().expect("Ran out of pairs");
            match inner_pair.as_rule() {
                Rule::EqualSign => Expression::Assign,
                Rule::PlusEqualsSign => Expression::PlusAssign,
                _ => panic_unexpected_rule!(inner_pair),
            }
        };
        let id = build_id(inner_pairs.pop().expect("Ran out of pairs"));

        rule_sign(id, Box::new(rhs_expression))
    }
}

fn build_expression2(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression2);

    let mut inner_pair = pair.into_inner().next().expect("inner_pairs is empty");

    loop {
        match inner_pair.as_rule() {
            Rule::Expression3 | Rule::Expression4 | Rule::Expression5 | Rule::Expression6 => {
                inner_pair = inner_pair.into_inner().next().expect("Ran out of pairs")
            }
            Rule::Expression7 => {
                return build_expression7(inner_pair);
            }
            _ => panic_unexpected_rule!(inner_pair),
        }
    }
}

fn build_expression7(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression7);

    let mut operands: Vec<Pair<Rule>> = pair.into_inner().collect();

    if operands.len() == 1 {
        build_expression8(operands.pop().unwrap())
    } else {
        let expr_right = build_expression8(operands.pop().unwrap());
        let middle_pair = operands.pop().unwrap();
        let expr_left = build_expression8(operands.pop().unwrap());

        let rule_sign = match middle_pair.as_rule() {
            Rule::LessThanSign => Expression::LessThan,
            Rule::LessEqualSign => Expression::LessEqual,
            Rule::NotEqualSign => Expression::NotEqual,
            Rule::EqualEqualSign => Expression::Equals,
            Rule::GreaterThanSign => Expression::GreaterThan,
            Rule::GreaterEqualSign => Expression::GreaterEqual,
            _ => panic_unexpected_rule!(middle_pair),
        };

        rule_sign(Box::new(expr_left), Box::new(expr_right))
    }
}

fn build_expression8(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression8);

    let operands: Vec<_> = pair.into_inner().map(build_expression9).collect();

    operands[1..]
        .iter()
        .fold(operands[0].clone(), |acc, operand| {
            Expression::Concatenate(Box::new(acc), Box::new(operand.clone()))
        })
}

fn build_expression9(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression9);

    let mut operands = vec![];

    let mut rule_sign: Option<fn(_, _) -> _> = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::PlusSign => {
                rule_sign = Some(Expression::Plus);
            }
            Rule::MinusSign => {
                rule_sign = Some(Expression::Minus);
            }
            Rule::Expression10 => operands.push(build_expression10(inner_pair)),
            _ => panic_unexpected_rule!(inner_pair),
        }
    }

    if operands.len() == 1 {
        operands[0].clone()
    } else {
        let rule_sign = rule_sign.expect("Rule sign not set for addition/subtraction");
        operands[1..].iter().fold(operands[0].clone(), |acc, item| {
            rule_sign(Box::new(acc), Box::new(item.clone()))
        })
    }
}

fn build_expression10(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression10);

    let mut operands = vec![];

    // NOTE(Chris): Using this type allows for inference of a type that supports both
    // Expression::Plus and Expression::Minus
    let mut rule_sign: Option<fn(_, _) -> _> = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::TimesSign => {
                rule_sign = Some(Expression::Times);
            }
            Rule::DivSign => {
                rule_sign = Some(Expression::Div);
            }
            Rule::Atom => operands.push(build_atom(inner_pair)),
            _ => panic_unexpected_rule!(inner_pair),
        }
    }

    if operands.len() == 1 {
        operands[0].clone()
    } else {
        let rule_sign = rule_sign.expect("Rule sign not set for addition/subtraction");
        operands[1..].iter().fold(operands[0].clone(), |acc, item| {
            rule_sign(Box::new(acc), Box::new(item.clone()))
        })
    }
}

// The Atom rule is used to build an Expression
fn build_atom(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Atom);

    for pair in pair.into_inner() {
        let s = pair.as_str();
        match pair.as_rule() {
            Rule::String => {
                return Expression::String {
                    value: s[1..s.len() - 1].to_string(),
                };
            }
            Rule::ColumnNumber => {
                let column_num = s[1..].parse().unwrap();
                return Expression::ColumnNumber(column_num);
            }
            Rule::VarLookup => {
                let inner_id_pair = pair.into_inner().next().expect("No inner pair");
                return Expression::VarLookup(build_id(inner_id_pair));
            }
            Rule::Num => {
                return build_num(pair);
            }
            Rule::Regex => {
                return Expression::Regex(build_regex(pair));
            }
            Rule::Expression => {
                return build_expression(pair);
            }
            _ => panic_unexpected_rule!(pair),
        }
    }

    unreachable!()
}

// The Num rule is used to build an Expression
fn build_num(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Num);

    Expression::Num(pair.as_str().parse().expect("Failed to parse number"))
}

fn build_id(pair: Pair<Rule>) -> Id {
    assert_eq!(pair.as_rule(), Rule::Id);

    Id(pair.as_str().to_string())
}
