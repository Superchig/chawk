use std::fs;

fn main() {
    let input_file = std::env::args().nth(1).unwrap();
    let unparsed_file = fs::read_to_string(&input_file).expect("Cannot read file");

    let program_ast = chawk::parse(&unparsed_file).unwrap();
    println!("{:#?}", &program_ast);
}
