use pest::Span;
use pest_ast::FromPest;

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "chawk.pest"]
pub struct ChawkParser;

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::Program))]
pub struct Program {
    pattern_blocks: Vec<PatternBlock>,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::PatternBlock))]
pub struct PatternBlock {
    pattern: Option<Pattern>,
    block: Option<Block>,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::Pattern))]
pub struct Pattern {
    #[pest_ast(outer(with(span_into_str), with(extract_regex)))]
    regex: String,
}

// This removes the surrounding // from a /regex/
fn extract_regex(str_from_span: &str) -> String {
    str_from_span[1..str_from_span.len() - 1].to_string()
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::Block))]
pub struct Block {
    statements: Vec<Statement>,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::Statement))]
pub enum Statement {
    PrintStatement(PrintStatement),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::PrintStatement))]
pub struct PrintStatement {
    expression: Expression,
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::Expression))]
pub enum Expression {
    // TODO(Chris): Replace this with something that works. Perhaps an actual lexer and parser?
    String {
        #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
        value: String,
    },
    ColumnNumber(ColumnNumber),
}

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::ColumnNumber))]
pub struct ColumnNumber(Integer);

#[derive(Debug, FromPest)]
#[pest_ast(rule(Rule::Integer))]
pub struct Integer {
    #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
    value: i64,
}

fn span_into_str(span: Span) -> &str {
    span.as_str()
}
