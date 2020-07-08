use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::PathBuf;
use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::{Builder, NamedTempFile};

/// Runs the assembler command and compares the output 'hack' file generated
/// against a known correct comparison file.
///
/// The paths to the input file and comparison file must be supplied, relative
/// to the directory containing the manifest of the package.
///
fn check_output_against_file(infile_relative_path: &str, compfile_relative_path: &str) {
    let mut cmd = Command::cargo_bin("assembler").unwrap();

    let mut infile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    infile_path.push(infile_relative_path);

    let temp_outfile = Builder::new()
        .suffix(".hack")
        .tempfile()
        .unwrap();

    cmd.arg(infile_path)
        .arg(temp_outfile.path())
        .assert()
        .success();

    let mut compfile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    compfile_path.push(compfile_relative_path);
    let compfile = File::open(compfile_path).unwrap();

    let mut outfile_buff = BufReader::new(temp_outfile);
    let mut compfile_buff = BufReader::new(compfile);

    let mut outfile_line = String::new();
    let mut compfile_line = String::new();

    let mut line_num = 0;

    loop {
        outfile_line.clear();
        compfile_line.clear();

        let outfile_line_bytes = outfile_buff.read_line(&mut outfile_line).unwrap();
        let compfile_line_bytes = compfile_buff.read_line(&mut compfile_line).unwrap();

        // check for end of files
        if outfile_line_bytes == 0 && compfile_line_bytes == 0 {
            break;
        } else if (outfile_line_bytes == 0) ^ (compfile_line_bytes == 0) {
            panic!("Error: The generated output file and comparison file are \
            different lengths!");
        }

        let result = if outfile_line == compfile_line {
            "Equivalent"
        } else {
            "***DIFFERENT!***"
        };

        println!(
            "Line {}\nOutput: {:>20}\nComparison: {}\n{:>22}\n",
            line_num,
            outfile_line.trim_end(),
            compfile_line.trim_end(),
            result
        );

        assert_eq!(outfile_line, compfile_line);

        line_num += 1;
    }
}

/// The below integration tests supply various input files and output comparison files
/// used to verify correct output is generated by the assembler command.
///
/// File paths must be specified relative to the directory containing the package
/// manifest.
///
#[test]
fn add() {
    check_output_against_file("testfiles/add/Add.asm", "testfiles/add/Add_comp.hack");
}

#[test]
fn max_l() {
    check_output_against_file("testfiles/max/MaxL.asm", "testfiles/max/MaxL_comp.hack");
}

#[test]
fn max() {
    check_output_against_file("testfiles/max/Max.asm", "testfiles/max/Max_comp.hack");
}

#[test]
#[ignore]  // Runtime ~1-2 mins.
fn pong_l() {
    check_output_against_file("testfiles/pong/PongL.asm", "testfiles/pong/PongL_comp.hack");
}

#[test]
#[ignore]  // Runtime ~1-2 mins.
fn pong() {
    check_output_against_file("testfiles/pong/Pong.asm", "testfiles/pong/Pong_comp.hack");
}

#[test]
fn rect_l() {
    check_output_against_file("testfiles/rect/RectL.asm", "testfiles/rect/RectL_comp.hack");
}

#[test]
fn rect() {
    check_output_against_file("testfiles/rect/Rect.asm", "testfiles/rect/Rect_comp.hack");
}

/// The below integration tests verify that the correct error responses are
/// generated when invalid, or too few, command line arguments are specified.
///
fn test_invalid_args(infile_relative_path: &str, temp_outfile: NamedTempFile, error: &str) {
    let mut cmd = Command::cargo_bin("assembler").unwrap();

    let mut infile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    infile_path.push(infile_relative_path);

    cmd.arg(infile_path)
        .arg(temp_outfile.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(error));
}

#[test]
fn invalid_infile_extension() {
    let infile_relative_path = "testfiles/add/Add_comp.hack"; // Should be '.asm'

    let temp_outfile = Builder::new()
        .suffix(".hack")
        .tempfile()
        .unwrap();

    test_invalid_args(
        infile_relative_path,
        temp_outfile,
        "invalid input file extension, only '.asm' accepted",
    );
}

#[test]
fn invalid_outfile_extension() {
    let infile_relative_path = "testfiles/add/Add.asm"; // Valid.

    let temp_outfile = Builder::new()
        .suffix(".asm") // Should be '.hack'
        .tempfile()
        .unwrap();

    test_invalid_args(
        infile_relative_path,
        temp_outfile,
        "invalid output file extension, only '.hack' accepted",
    );
}

#[test]
fn missing_arguments() {
    let mut cmd_0 = Command::cargo_bin("assembler").unwrap();
    let mut cmd_1 = Command::cargo_bin("assembler").unwrap();

    let mut infile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    infile_path.push("testfiles/add/Add.asm");

    // No arguments.
    cmd_0.assert()
        .failure()
        .stderr(predicate::str::contains("input and output filenames were not provided"));

    // 1 of 2 arguments provided.
    cmd_1.arg(infile_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("output filename not provided"));
}

#[test]
fn infile_does_not_exist() {
    let mut cmd = Command::cargo_bin("assembler").unwrap();

    let mut infile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    infile_path.push("testfiles/add/Add01.asm");

    let temp_outfile = Builder::new()
        .suffix(".hack")
        .tempfile()
        .unwrap();

    cmd.arg(infile_path)
        .arg(temp_outfile.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}
