use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("examples/print_line.awk").expect("Cannot read file");

    let program_ast = chawk::parse(&unparsed_file).unwrap();
    println!("{:#?}", &program_ast);
}
