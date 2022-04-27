use ariadne::{Label, Report, ReportKind, Source};
use chawk::{Expression, Id, PrintStatement, Statement};
use chumsky::prelude::*;
use std::fs;

fn main() {
    let input_file = std::env::args().nth(1).unwrap();
    let raw_src = fs::read_to_string(&input_file).unwrap();

    let uncommented_src = uncomment(&raw_src);

    match parse_statement().parse(uncommented_src) {
        Ok(program_ast) => {
            println!("{:#?}", &program_ast);
        }
        Err(parse_errs) => {
            let first_err = parse_errs.first().unwrap();

            // NOTE(Chris): The use of `&input_file` in the build() call, the Label::new() call,
            // and the print() call must all be of the same type
            Report::build(ReportKind::Error, &input_file, 0)
                .with_message("Unable to parse")
                .with_label(Label::new((&input_file, first_err.span())).with_message(first_err))
                .finish()
                .print((&input_file, Source::from(raw_src)))
                .unwrap();
        }
    }
}

/// This function provides a quick-and-dirty way to replace comments with whitespace, which will
/// allow for spans to accurately describe locations in the input.
fn uncomment(string: &str) -> String {
    let mut result = String::new();

    let mut is_in_comment = false;

    for ch in string.chars() {
        if is_in_comment {
            if ch == '\n' {
                is_in_comment = false;
                result.push(ch);
            } else {
                result.push(' ');
            }
        } else if ch == '#' {
            is_in_comment = true;
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    result
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncomment() {
        let text = r#"
        # This will print the same output for each line
        { print "This line will be repeated!" }
        { print $1 } # This comment is at the end
        "#;

        // The "first" line of this string has a bunch of whitespace rather than a comment
        // The "third" line of this string has whitespace at the end rather than a comment
        let text_with_spaces = r#"
                                                       
        { print "This line will be repeated!" }
        { print $1 }                             
        "#;

        assert_eq!(uncomment(text), text_with_spaces);
    }
}
