use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

mod utils;

include!(concat!(env!("OUT_DIR"), "/runtests_tests.rs"));

fn run_test(path: &str) -> Result<(), io::Error> {
    let path = Path::new(path);

    let path = path.to_owned();
    println!("testing {}", path.display());
    let program = std::fs::read_to_string(&path).unwrap();
    let mut reader = io::BufReader::new(std::fs::File::open(&path)?);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let test_func = match first_line.as_str().trim() {
        // make sure the test compiles, but don't run it
        "// compile" => utils::assert_compiles,
        // the test compiles, and don't require a `main` function
        "// no-main" => utils::assert_compiles_no_main,
        // the test shouldn't compile
        "// fail" | "// compile-fail" | "// compile-error" => utils::assert_compile_error,
        // it should compile, run, and exit successfully
        "// succeeds" => utils::assert_succeeds,
        // it should compile successfully then crash at runtime
        "// crash" => utils::assert_crash,
        // crashes and exits with a stack overflow status
        "// stack-overflow" => utils::assert_stack_overflow,
        // tests can only be ignored if they have an issue open on github
        "// ignore" => {
            // let p: String = path;
            println!("WARNING: Ignored test {}", path.to_str().unwrap());
            // panic!("ignored tests should have an associated issue")
            return Ok(());
        }
        // `code: x` - it should compile, run, and exit with code x
        // NOTE: x should not be negative
        // NOTE: x should be less than 256 since Linux only has 8-bit exit codes
        line if line.starts_with("// code: ") => {
            let code = line
                .trim_start_matches("// code: ")
                .trim()
                .parse()
                .expect("tests should have an integer after code:");
            utils::assert_code(&program, path, code);
            return Ok(());
        }
        // `errors: x` - it should not compile and rcc should output `x` errors
        line if line.starts_with("// errors: ") => {
            let errors = line
                .trim_start_matches("// errors: ")
                .trim()
                .parse()
                .expect("tests should have an integer after errors:");
            utils::assert_num_errs(&program, path, errors);
            return Ok(());
        }
        line if line.starts_with("// ignore: ") => {
            let url = line.trim_start_matches("// ignore: ").trim();
            assert!(
                url.starts_with("https://") || url.starts_with("http://"),
                "ignored tests should have an associated issue"
            );
            return Ok(());
        }
        // `output: x` - it should run and output `x`
        // this has a convoluted syntax for multiline strings, see `output_test`
        line if line.starts_with("// output: ") => {
            // let output = line
            //     .trim_start_matches("// output: ")
            //     .trim();
            return output_test(&line["// output: ".len()..], &mut reader, &program, path);
        }
        line => panic!("Unrecognized test: {}", line),
    };

    test_func(&program, path);
    Ok(())
}

/// small state machine to handle 'output' syntax
/// syntax: '// output: ' expected_output
/// expected_output: '[^\n]*' | 'BEGIN: ' (comment_line* '\n' | [^\n]+) 'END'
/// comment_line: '\n// ' [^\n+]
///
/// Examples:
/// No output: `// output: `
/// Single-line output: `// output: hello, wojkjrld!`
/// Single-line with no trailing newline: `// output: BEGIN: hi END`
/// Multi-line output:
/// ```c
/// // output: BEGIN:
/// // multi-line
/// // string
/// // END
/// ```
fn output_test<B: BufRead>(
    line: &str,
    reader: &mut B,
    program: &str,
    path: PathBuf,
) -> Result<(), io::Error> {
    const BEGIN: &str = "BEGIN: ";
    const END: &str = "END";
    let tmp_str;
    let expected = match line {
        "" => "", // special case this so empty output doesn't need to use 'BEGIN: END'
        // everything between BEGIN: (...) END
        _ if line.starts_with(BEGIN) && line.ends_with(END) => {
            &line[BEGIN.len()..line.len() - END.len() - 1]
        }
        // special case initial lines that are empty
        "BEGIN:" => {
            tmp_str = state_machine(reader)?;
            &tmp_str
        }
        _ if line.starts_with(BEGIN) => {
            tmp_str = format!("{}{}", &line[BEGIN.len()..], state_machine(reader)?);
            &tmp_str
        }
        _ => {
            tmp_str = format!("{}\n", line);
            &tmp_str
        }
    };
    utils::assert_output(program, path, expected);
    Ok(())
}

fn state_machine<B: BufRead>(reader: &mut B) -> Result<String, io::Error> {
    const COMMENT: &str = "// ";
    let mut expected_out = String::new();
    for line in reader.lines() {
        let line = dbg!(line?);
        if line == "// END" {
            break;
        } else if !line.starts_with(COMMENT) {
            println!("warning: test runner: invalid syntax in program comment, expected `// <output>` or `// END`");
            break;
        }
        expected_out.push_str(&line[COMMENT.len()..]);
        expected_out.push('\n');
    }
    Ok(expected_out)
}
