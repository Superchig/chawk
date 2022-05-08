use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command, ffi::OsStr,
};

fn main() -> Result<()> {
    let awk_input_entries: Vec<_> = fs::read_dir("test/")?.map(|val| val.unwrap()).collect();

    let mut tested_files = 0;
    let mut successful_files = 0;

    for awk_input_entry in awk_input_entries {
        let awk_input_path = awk_input_entry.path();
        if let Some(ext) = awk_input_path.extension() {
            if ext != OsStr::new("awk") && ext != OsStr::new("chawk") {
                continue;
            }
        }

        let awk_input_path_str = awk_input_path.to_str().unwrap();

        println!("Testing {}", awk_input_path_str);

        if let Err(err) = test_awk_file(awk_input_path_str) {
            make_red();
            println!("Error testing {}: {}", awk_input_path_str, err);
            reset_color();
        } else {
            successful_files += 1;
        }

        tested_files += 1;
    }

    println!();
    println!("------------------------------");
    if successful_files == tested_files {
        make_green();
    } else {
        make_red();
    }
    println!("Good programs: {} out of {}", successful_files, tested_files);
    reset_color();

    Ok(())
}

fn test_awk_file(awk_input_file: &str) -> Result<()> {
    let awk_input_file_path = Path::new(&awk_input_file);

    // let unparsed_file = fs::read_to_string(&awk_input_file_path).expect("Cannot read file");
    // let awk_program_ast = parse(&unparsed_file).expect("Failed to parse awk program");

    let desired_output_file = format!("{}.output", awk_input_file);
    let desired_output_path = Path::new(&desired_output_file);
    let unparsed_desired_output_file =
        fs::read_to_string(&desired_output_path).with_context(|| "Failed to open output file")?;
    let desired_outputs = parse_output_file(&unparsed_desired_output_file);

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
            reset_color();

            bail!("Output of program did not match desired output");
        }
    }

    Ok(())
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
