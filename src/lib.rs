use pest::{error::Error, iterators::Pair, Parser, Span};

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "chawk.pest"]
pub struct ChawkParser;

#[derive(Debug)]
pub struct Program {
    pattern_blocks: Vec<PatternBlock>,
}

#[derive(Debug)]
pub struct PatternBlock {
    pattern: Option<Pattern>,
    block: Option<Block>,
}

#[derive(Debug)]
pub struct Pattern {
    regex: String,
}

// This removes the surrounding // from a /regex/
fn extract_regex(str_from_span: &str) -> String {
    str_from_span[1..str_from_span.len() - 1].to_string()
}

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    PrintStatement(PrintStatement),
}

#[derive(Debug)]
pub struct PrintStatement {
    expression: Expression,
}

#[derive(Debug)]
pub enum Expression {
    // TODO(Chris): Replace this with something that works. Perhaps an actual lexer and parser?
    String { value: String },
    ColumnNumber(ColumnNumber),
}

#[derive(Debug)]
pub struct ColumnNumber(i64);

fn span_into_str(span: Span) -> &str {
    span.as_str()
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
                program
                    .pattern_blocks
                    .push(build_pattern_block(pair.into_inner()));
            }
            Rule::EOI => (),
            _ => panic!("Unsupported parsing rule: {:?}", pair),
        }
    }

    Ok(program)
}

fn build_pattern_block(pairs: pest::iterators::Pairs<Rule>) -> PatternBlock {
    let mut pattern_block = PatternBlock {
        pattern: None,
        block: None,
    };

    for pair in pairs {
        let span = pair.as_span().as_str();

        match pair.as_rule() {
            Rule::Pattern => {
                pattern_block.pattern = Some(Pattern {
                    regex: span[1..span.len() - 1].to_string(),
                });
            }
            Rule::Block => {
                // FIXME(Chris): Actually implement
                pattern_block.block = Some(build_block(pair.into_inner()));
            }
            _ => panic!("Unsupported parsing rule: {:?}", pair),
        }
    }

    pattern_block
}

fn build_block(pairs: pest::iterators::Pairs<Rule>) -> Block {
    let mut block = Block { statements: vec![] };

    for stm_pair in pairs {
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
            _ => panic!("Unsupported parsing rule: {:?}", pair),
        }
    }

    unreachable!()
}

fn build_expression(pair: Pair<Rule>) -> Expression {
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
                return Expression::ColumnNumber(ColumnNumber(column_num));
            }
            _ => panic!("Unsupported parsing rule: {:?}", pair),
        }
    }

    unreachable!()
}
