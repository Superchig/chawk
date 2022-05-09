use std::{fmt::Display, collections::HashMap};

use regex::Regex;

#[derive(Debug)]
pub struct Program {
    pub pattern_blocks: Vec<PatternBlock>,
    // This redundantly stores the name of a function as a key and contains the name of the
    // function in the definition
    pub function_defs: HashMap<Id, FunctionDef>,
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

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: Id,
    pub parameters: Vec<Id>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
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
    },
    ReturnStatement(Expression),
}

#[derive(Debug, Clone)]
pub enum InitClause {
    Expression(Expression),
    // This should generally only be a LocalVarStatement, since no other statement makes sense in the initialization clause of a for loop
    Declaration(Box<Statement>),
}

#[derive(Debug, Clone)]
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

    RegexMatch(Box<Expression>, Box<Expression>),
    RegexNotMatch(Box<Expression>, Box<Expression>),

    FunctionCall {
        name: Id,
        arguments: Vec<Box<Expression>>,
    },

    LogicalOr(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Id(pub String);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
