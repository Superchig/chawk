use std::fmt::Display;

use regex::Regex;

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
    ExpressionStatement(Expression),
    BlockStatement(Block),
    LocalVarStatement {
        id: Id,
        initial_expression: Option<Expression>,
    },
    IfStatement {
        condition: Expression,
        true_statement: Box<Statement>,
        false_statement: Option<Box<Statement>>,
    },
    WhileStatement {
        condition: Expression,
        body: Box<Statement>,
    },
    ForStatement {
        init_clause: Option<InitClause>,
        condition_expression: Option<Expression>,
        iteration_expression: Option<Expression>,
        body: Box<Statement>,
    }
}

#[derive(Debug)]
pub enum InitClause {
    Expression(Expression),
    // This should generally only be a LocalVarStatement, since no other statement makes sense in the initialization clause of a for loop
    Declaration(Box<Statement>),
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
    Regex(Regex),

    Plus(Box<Expression>, Box<Expression>),
    Minus(Box<Expression>, Box<Expression>),
    Times(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),

    Concatenate(Box<Expression>, Box<Expression>),

    LessThan(Box<Expression>, Box<Expression>),
    LessEqual(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    Equals(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    GreaterEqual(Box<Expression>, Box<Expression>),

    Assign(Id, Box<Expression>),
    PlusAssign(Id, Box<Expression>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Id(pub String);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
