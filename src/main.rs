use anyhow::{anyhow, Result};
use clap::Clap;
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};

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
struct TestResults {
    success: bool,
    output: String,
}

fn main() -> Result<()> {
    let options: Options = Options::parse();
    let file = File::open(options.config)?;
    let reader = BufReader::new(file);
    let config: ConfigRoot = serde_json::from_reader(reader)?;
    let mut points = 0u16;
    let total_points = config
        .tests
        .iter()
        .filter_map(|test| test.points)
        .reduce(|a, b| a + b)
        .unwrap_or(0);
    let mut all_succeeded = true;

    // Move Unicode error and success stuff up

    for test in config.tests {
        match run_test(&test) {
            Ok(results) => {
                if results.success {
                    if let Some(test_points) = test.points {
                        points += test_points;
                    }
                    println!("{}", results.output);
                } else {
                    all_succeeded = false;
                    eprintln!("{}", results.output);
                }
            }
            Err(error) => {
                all_succeeded = false;
                eprintln!("{}", error.to_string().red());
            }
        }
    }

    if all_succeeded {
        println!("{}", "All tests pass".green());
        println!("✨🌟💖💎🦄💎💖🌟✨🌟💖💎🦄💎💖🌟✨");
    }
    println!("Points {}/{}", points, total_points);
    Ok(())
}

fn run_test(test: &TestCase) -> Result<TestResults> {
    println!("📝 {}", test.name);
    set_up_test(&test)?;
    let results = get_test_results(&test)?;
    Ok(results)
}

fn set_up_test(test: &TestCase) -> Result<String> {
    if let Some(setup) = &test.setup {
        match Command::new(setup).output() {
            Ok(output) => {
                if output.status.success() {
                    if let Ok(stdout) = String::from_utf8(output.stdout) {
                        Ok(stdout)
                    } else {
                        Err(anyhow!("{}", "Could not read stdout as utf8"))
                    }
                } else {
                    let failure_message =
                        format!("❌ {} {}\n\n", "Failed to set up test", test.name);
                    if let Ok(stderr) = String::from_utf8(output.stderr) {
                        Err(anyhow!("{}\n{}", failure_message, stderr))
                    } else {
                        Err(anyhow!(
                            "{}\n{}",
                            failure_message,
                            "Coult not read stderr as utf8"
                        ))
                    }
                }
            }
            Err(error) => Err(anyhow!(
                "❌ {} {}\n{}",
                "Failed to set up test",
                test.name,
                error.to_string()
            )),
        }
    } else {
        Ok("".to_string())
    }
}

fn get_test_results(test: &TestCase) -> Result<TestResults> {
    let mut run_parts = test.run.split(" ");
    let executable = run_parts
        .next()
        .ok_or(anyhow!("Could not get run command executable"))?;
    let args: Vec<_> = run_parts.collect();
    let mut command = Command::new(&executable)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let stdin = command
            .stdin
            .as_mut()
            .ok_or(anyhow!("Could not get a handle to stdin"))?;
        stdin.write_all(test.input.as_bytes())?;
        // Stdin drops and finishes input
    }
    let output = command.wait_with_output()?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let success = match test.comparison {
            Comparison::Included => stdout.contains(&test.output),
            Comparison::Exact => stdout.eq(&test.output),
            Comparison::Regex => {
                let re = Regex::new(&test.output)?;
                re.is_match(&stdout)
            }
        };
        Ok(TestResults {
            success,
            output: stdout,
        })
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("{}", stderr))
    }
}
