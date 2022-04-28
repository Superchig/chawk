use pest::{error::Error, iterators::Pair, Parser};

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
pub struct Pattern {
    pub regex: String,
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

    Plus(Box<Expression>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
    Times(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
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
        let span = pair.as_span().as_str();

        match pair.as_rule() {
            Rule::Pattern => {
                pattern_block.pattern = Some(Pattern {
                    regex: span[1..span.len() - 1].to_string(),
                });
            }
            Rule::Block => {
                pattern_block.block = Some(build_block(pair));
            }
            _ => panic!("Unsupported parsing rule: {:?}", pair),
        }
    }

    pattern_block
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
        Rule::Expression2 => build_expression2(inner_pair),
        _ => panic!("Unsupported parsing rule: {:#?}", inner_pair),
    }
}

fn build_expression2(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression2);

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
            Rule::Expression1 => operands.push(build_expression1(inner_pair)),
            _ => panic!("Unsupported parsing rule: {:#?}", inner_pair)
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

// The Expression1 rule is used to build an Expression
// This may result in a Expression::Plus or an Expression::Minus
fn build_expression1(pair: Pair<Rule>) -> Expression {
    assert_eq!(pair.as_rule(), Rule::Expression1);

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
            _ => panic!("Unsupported parsing rule: {:#?}", inner_pair)
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
            Rule::Expression => {
                return build_expression(pair);
            }
            _ => panic!("Unsupported parsing rule: {:#?}", pair),
        }
    }

    unreachable!()
}

fn build_id(pair: Pair<Rule>) -> Id {
    Id(pair.as_str().to_string())
}
