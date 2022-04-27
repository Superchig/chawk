use ariadne::{Source, Label, Report, ReportKind};
use chawk::{Expression, Id, Statement, PrintStatement};
use chumsky::prelude::*;
use std::fs;

fn main() {
    let input_file = std::env::args().nth(1).unwrap();
    let src = fs::read_to_string(&input_file).unwrap();

    match parse_statement().parse(src.clone()) {
        Ok(program_ast) => {
            println!("{:#?}", &program_ast);
        },
        Err(parse_errs) => {
            let first_err = parse_errs.first().unwrap();

            // NOTE(Chris): The use of `&input_file` in the build() call, the Label::new() call,
            // and the print() call must all be of the same type
            Report::build(ReportKind::Error, &input_file, 0)
                .with_message("Unable to parse")
                .with_label(Label::new((&input_file, first_err.span())).with_message(first_err))
                .finish()
                .print((&input_file, Source::from(src)))
                .unwrap();

            // parse_errs
            //     .into_iter()
            //     .for_each(|e| println!("Parse error: {}", e));
        }
    }
}

// FIXME(Chris): Support comments, which may require the use of a chumsky-driven tokenizer/lexer

fn parse_statement() -> impl Parser<char, Statement, Error = Simple<char>> {
    let print_statement = just("print")
        .ignore_then(parse_expression())
        .map(|expr: Expression| Statement::PrintStatement(PrintStatement { expression: expr }))
        .padded();

    print_statement.then_ignore(end())
}

// char is the possible input token, Expression is the output value, Simple<char> is the Error
fn parse_expression() -> impl Parser<char, Expression, Error = Simple<char>> {
    let string = just("\"")
        .ignore_then(none_of("\"").repeated())
        .then_ignore(just("\""))
        .collect::<String>()
        .map(|s: String| Expression::String { value: s })
        .padded();

    let column_number = just("$")
        .ignore_then(text::int(10))
        .map(|s: String| Expression::ColumnNumber(s.parse().unwrap()))
        .padded();

    let var_lookup = text::ident()
        .map(|s: String| Expression::VarLookup(Id(s)))
        .padded();

    string.or(column_number).or(var_lookup)
}
