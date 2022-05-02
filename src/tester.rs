use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() -> Result<()> {
    let awk_input_file = std::env::args().nth(1).unwrap();

    test_awk_file(&awk_input_file)?;

    Ok(())
}

fn test_awk_file(awk_input_file: &str) -> Result<bool> {
    let awk_input_file_path = Path::new(&awk_input_file);

    // let unparsed_file = fs::read_to_string(&awk_input_file_path).expect("Cannot read file");
    // let awk_program_ast = parse(&unparsed_file).expect("Failed to parse awk program");

    let desired_output_file = format!("{}.output", awk_input_file);
    let desired_output_path = Path::new(&desired_output_file);
    let unparsed_desired_output_file = fs::read_to_string(&desired_output_path)?;
    let desired_outputs = parse_output_file(&unparsed_desired_output_file);

    let mut are_all_outputs_correct = true;

    for desired_output in &desired_outputs {
        let mut data_file_path = if let Some(parent_path) = awk_input_file_path.parent() {
            parent_path.to_path_buf()
        } else {
            PathBuf::new()
        };
        data_file_path.push(&desired_output.data_file_name);

        // TODO(Chris): Avoid shelling out to target/debug/chawk directly, as it's not guaranteed
        // to have the most recently-built version of the interpreter.
        let mut chawk_command = Command::new("target/debug/chawk");
        chawk_command
            .arg("-f")
            .arg(&awk_input_file)
            .arg(&data_file_path);

        // println!("Running {:?}", chawk_command);
        let output = chawk_command.output()?;

        let stdout = std::str::from_utf8(&output.stdout)?;

        if stdout == desired_output.output {
            make_green();
            println!(
                "Successfully run: {} {}",
                &awk_input_file_path
                    .to_str()
                    .expect("Failed to convert to str"),
                &data_file_path.to_str().expect("Failed to convert to str"),
            );
            reset_color();
        } else {
            are_all_outputs_correct = false;

            make_red();
            println!(
                "Incorrect output: {} {}",
                &awk_input_file_path
                    .to_str()
                    .expect("Failed to convert to str"),
                &data_file_path.to_str().expect("Failed to convert to str")
            );
            reset_color();

            println!("Execution output:");
            make_red();
            println!("{}", &stdout);
            reset_color();

            println!("Expected output:");
            make_cyan();
            println!("{}", &desired_output.output);
            reset_color();

            println!("Stderr:");
            make_magenta();
            println!("{}", std::str::from_utf8(&output.stderr)?);
        }
    }

    Ok(are_all_outputs_correct)
}

fn make_green() {
    print!("\x1b[32m");
}

fn make_red() {
    print!("\x1b[31m");
}

fn make_cyan() {
    print!("\x1b[36m");
}

fn make_magenta() {
    print!("\x1b[35m");
}

fn reset_color() {
    print!("\x1b[0m");
}

struct DesiredOutput {
    data_file_name: String,
    output: String,
}

fn parse_output_file(source: &str) -> Vec<DesiredOutput> {
    let mut result = vec![];

    for line in source.lines() {
        if line.starts_with("{{ ") && line.ends_with(" }}") {
            let data_file_name = line[3..line.len() - 3].to_string();
            result.push(DesiredOutput {
                data_file_name,
                output: String::new(),
            });
        } else {
            let last_output = result
                .last_mut()
                .expect("No data file name has been detected");
            last_output.output.push_str(line);
            last_output.output.push('\n');
        }
    }

    result
}
