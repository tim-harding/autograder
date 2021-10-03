use anyhow::Context;
use clap::Clap;
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Clap, Debug, Clone, Hash, PartialEq, Eq)]
struct Options {
    #[clap(short, long, default_value = "./.github/classroom/autograding.json")]
    config: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ConfigRoot {
    tests: Vec<TestCase>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    name: String,
    setup: Option<String>,
    run: String,
    // Todo: Optional input and output
    input: String,
    output: String,
    comparison: Comparison,
    timeout: u16,
    points: Option<u16>,
}

#[derive(Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
enum Comparison {
    Included,
    Exact,
    Regex,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TestOutcome {
    success: bool,
    stdout: String,
}

#[derive(Debug, Error)]
enum TestFailure {
    #[error("{0}")]
    Stderr(String),
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("{0}")]
    Regex(#[from] regex::Error),
}

fn main() -> anyhow::Result<()> {
    let options: Options = Options::parse();
    let file =
        File::open(options.config).context("Could not find the autograding configuration")?;
    let reader = BufReader::new(file);
    let config: ConfigRoot = serde_json::from_reader(reader)
        .context("Could not read the autograding configuration file as JSON")?;
    let total_points = config
        .tests
        .iter()
        .filter_map(|test| test.points)
        .reduce(|a, b| a + b)
        .unwrap_or(0);

    let mut points = 0u16;
    let mut all_succeeded = true;

    for test in config.tests {
        let pass = set_up_and_run_test(&test);
        if pass {
            if let Some(test_points) = test.points {
                points += test_points;
            }
        } else {
            all_succeeded = false;
        }
        println!("\n");
    }

    if all_succeeded {
        println!(
            "{}\n✨🌟💖💎🦄💎💖🌟✨🌟💖💎🦄💎💖🌟✨",
            "All tests pass".green()
        );
    }
    println!("Points {}/{}", points, total_points);
    Ok(())
}

fn set_up_and_run_test(test: &TestCase) -> bool {
    println!("📝 {}", test.name);
    if let Some(setup) = &test.setup {
        match set_up_test(&setup) {
            Ok(stdout) => {
                print!("{}", stdout);
            }
            Err(error) => {
                if let TestFailure::Stderr(stderr) = error {
                    println!("{}❌ {}", stderr, test.name.red());
                } else {
                    println!("{}\n❌ {}", error.to_string().red(), test.name.red());
                }
                return false;
            }
        }
    }
    match run_test(&test) {
        Ok(outcome) => {
            if outcome.success {
                println!("{}✅ {}", outcome.stdout, test.name.green())
            } else {
                println!("{}❌ {}", outcome.stdout, test.name.red())
            }
            outcome.success
        }
        Err(error) => {
            println!("{}\n❌ {}", error.to_string().red(), test.name.red());
            false
        }
    }
}

fn set_up_test(setup_command: &str) -> anyhow::Result<String> {
    let output = Command::new(setup_command)
        .output()
        .context("Failed to run test setup")?;
    if output.status.success() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            Ok(stdout)
        } else {
            let message = format!("Could not read stdout as utf8");
            Err(TestFailure::Message(message).into())
        }
    } else {
        if let Ok(stderr) = String::from_utf8(output.stderr) {
            Err(TestFailure::Stderr(stderr).into())
        } else {
            let message = format!("Could not read stderr as utf8");
            Err(TestFailure::Message(message).into())
        }
    }
}

fn run_test(test: &TestCase) -> Result<TestOutcome, TestFailure> {
    let mut command = Command::new("bash")
        .args(&["-c", &test.run])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start Bash with the test run command")?;

    {
        let stdin = command.stdin.as_mut().ok_or(TestFailure::Message(
            "Could not get a handle to stdin".to_string(),
        ))?;
        stdin.write_all(test.input.as_bytes())?;
    } // Stdin drops and finishes input

    let output = command.wait_with_output()?;
    if output.status.success() {
        // Todo: Include this as part of error
        // std::io::stdout().write(&output.stdout)?;
        let stdout = String::from_utf8(output.stdout)?;
        let success = match test.comparison {
            Comparison::Included => stdout.contains(&test.output),
            Comparison::Exact => stdout.eq(&test.output),
            Comparison::Regex => {
                let re = Regex::new(&test.output)?;
                re.is_match(&stdout)
            }
        };
        Ok(TestOutcome { success, stdout })
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(TestFailure::Stderr(stderr))
    }
}
