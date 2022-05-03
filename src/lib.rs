use pest::{error::Error, iterators::Pair, Parser};
use regex::Regex;

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "chawk.pest"]
pub struct ChawkParser;

#[derive(Debug)]
pub struct Program {
    pub pattern_blocks: Vec<PatternBlock>,
}

#[derive(Debug)]
pub struct PatternBlock {
    pub pattern: Option<Pattern>,
    pub block: Option<Block>,
}

#[derive(Debug)]
pub enum Pattern {
    Regex(Regex),
    Expression(Expression),
    Begin,
    End,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    PrintStatement(PrintStatement),
    AssignStatement { id: Id, expression: Expression },
}

#[derive(Debug)]
pub struct PrintStatement {
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    String { value: String },
    ColumnNumber(u32),
    VarLookup(Id),
    Num(f64), // In awk, all numbers are floats

    Plus(Box<Expression>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
    Times(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),

    Equals(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Id(String);

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
            // FIXME(Chris): Replace this message with "Unexpected parsing rule: ..."
            _ => panic!("Unsupported parsing rule: {:?}", pair),
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
            _ => panic!("Unsupported parsing rule: {:?}", pair),
        }
    }

    pattern_block
}

fn build_pattern(pair: Pair<Rule>) -> Pattern {
    assert_eq!(pair.as_rule(), Rule::Pattern);

    let span = pair.as_str();

    if let Some(inner_pair) = pair.into_inner().next() {
        match inner_pair.as_rule() {
            Rule::Regex => Pattern::Regex(build_regex(inner_pair)),
            Rule::Expression => Pattern::Expression(build_expression(inner_pair)),
            _ => panic!("Unsupported parsing rule: {:?}", inner_pair),
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
    let mut block = Block { statements: vec![] };

    for stm_pair in pair.into_inner() {
        block.statements.push(build_statement(stm_pair));
    }

    block
}

fn build_statement(pair: Pair<Rule>) -> Statement {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::PrintStatement => {
                let mut inner_iter = pair.into_inner();
                let expression_pair = inner_iter.next().unwrap();
                let expression = build_expression(expression_pair);

                return Statement::PrintStatement(PrintStatement { expression });
            }
            Rule::AssignStatement => {
                let mut inner_iter = pair.into_inner();
                let id_pair = inner_iter.next().unwrap();
                let expression_pair = inner_iter.next().unwrap();

                let id = build_id(id_pair);
                let expression = build_expression(expression_pair);

                return Statement::AssignStatement { id, expression };
            }
            _ => panic!("Unsupported parsing rule: {:#?}", pair),
        }
    }

    unreachable!()
}

fn build_expression(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression);

    let inner_pair = pair.into_inner().next().expect("No pair inside rule");

    match inner_pair.as_rule() {
        Rule::Expression1 => build_expression1(inner_pair),
        _ => panic!("Unsupported parsing rule: {:#?}", inner_pair),
    }
}

fn build_expression1(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression1);

    let mut inner_pair = pair.into_inner().next().expect("No pair inside rule");

    loop {
        match inner_pair.as_rule() {
            Rule::Expression2
            | Rule::Expression3
            | Rule::Expression4
            | Rule::Expression5
            | Rule::Expression6 => {
                inner_pair = inner_pair.into_inner().next().expect("No pair inside rule")
            }
            Rule::Expression7 => {
                return build_expression7(inner_pair);
            }
            _ => panic!("Unsupported parsing rule: {:#?}", inner_pair),
        }
    }
}

fn build_expression7(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression7);

    let mut operands: Vec<Expression> = pair.into_inner().map(build_expression8).collect();

    if operands.len() == 1 {
        operands.pop().unwrap()
    } else {
        let expr_right = operands.pop().unwrap();
        let expr_left = operands.pop().unwrap();

        Expression::Equals(Box::new(expr_left), Box::new(expr_right))
    }
}

fn build_expression8(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression8);

    let inner_pair = pair.into_inner().next().expect("No pair inside rule");

    match inner_pair.as_rule() {
        Rule::Expression9 => build_expression9(inner_pair),
        _ => panic!("Unsupported parsing rule: {:#?}", inner_pair),
    }
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
            _ => panic!("Unsupported parsing rule: {:#?}", inner_pair),
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
            _ => panic!("Unsupported parsing rule: {:#?}", inner_pair),
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
                return Expression::VarLookup(build_id(pair));
            }
            Rule::Num => {
                return build_num(pair);
            }
            Rule::Expression => {
                return build_expression(pair);
            }
            _ => panic!("Unsupported parsing rule: {:#?}", pair),
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
    Id(pair.as_str().to_string())
}
