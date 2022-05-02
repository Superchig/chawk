use std::{fs, path::{Path, PathBuf}, process::Command};

// FIXME(Chris): Use anyhow for this function
fn main() {
    let awk_input_file = std::env::args().nth(1).unwrap();
    let awk_input_file_path = Path::new(&awk_input_file);

    // let unparsed_file = fs::read_to_string(&awk_input_file_path).expect("Cannot read file");
    // let awk_program_ast = parse(&unparsed_file).expect("Failed to parse awk program");

    let desired_output_file = format!("{}.output", awk_input_file);
    let desired_output_path = Path::new(&desired_output_file);
    let unparsed_desired_output_file =
        fs::read_to_string(&desired_output_path).expect("Cannot read file");
    let desired_outputs = parse_output_file(&unparsed_desired_output_file);

    for desired_output in &desired_outputs {
        // dbg!(&desired_output.data_file_name);

        let data_file_path = if let Some(parent_path) = awk_input_file_path.parent() {
            let mut result = parent_path.to_path_buf();
            result.push(&desired_output.data_file_name);
            result
        } else {
            let mut result = PathBuf::new();
            result.push(&desired_output.data_file_name);
            result
        };

        // TODO(Chris): Avoid shelling out to target/debug/chawk directly, as it's not guaranteed
        // to have the most recently-built version of the interpreter.
        let output = Command::new("target/debug/chawk")
            .arg("-f")
            .arg(&awk_input_file)
            .arg(&data_file_path)
            .output()
            .unwrap();

        let stdout = std::str::from_utf8(&output.stdout).unwrap();

        // println!("{}", stdout);

        // println!("{}", &desired_output.output);

        // FIXME(Chris): Print differently-colored output for correct and incorrect runs, with
        // child stdout and stderr shown on incorrect runs
        if stdout == desired_output.output {
            println!("Program ran correctly!");
        } else {
            println!("Program did NOT run correctly.");
        }
    }
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
