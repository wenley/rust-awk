#[cfg(test)]
mod integration_tests {
    use crate::*;
    // use std::fs::read_to_string;
    use std::process::{Command, Stdio};
    use std::io::Write;
    use std::str::from_utf8;

    fn check_program_with_input(program_string: &str, input: &str) {
        let program = parse_program(program_string);
        let mut run = start_run(&program);

        let rust_output = input.lines().flat_map(|line| run.output_for_line(line)).map(|s| s + "\n").collect::<Vec<String>>().join("");
        println!("{}", rust_output);

        let mut reference_command = Command::new("awk")
            .args(&[program_string])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn");
        {
            let stdin = reference_command.stdin.as_mut().expect("Failed to open StdIn for reference");
            stdin.write(input.as_bytes()).expect("Failed to write");
        }

        let reference_output = reference_command.wait_with_output().expect("Failed to read StdOut from reference").stdout;
        assert_eq!(rust_output, from_utf8(&reference_output).expect("Failed to deserialize UTF-8 to String"));
    }

    #[test]
    fn first_test() {
        let program_string = r#"{ print($0); }"#;
        let input = r#"foo
        bar
        baz"#;
        check_program_with_input(program_string, input);
    }
}
