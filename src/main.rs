use anyhow::{anyhow, Result};
use clap::Clap;
use colored::Colorize;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};

// Todo: Find config by default
// Todo: Fix line endings

#[derive(Clap, Debug, Clone, Hash, PartialEq, Eq)]
struct Options {
    #[clap(short, long)]
    config: String,
    #[clap(short, long)]
    skip_setup: bool,
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

fn main() -> Result<()> {
    let opts: Options = Options::parse();
    let file = File::open(opts.config)?;
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

    println!("{:?}", std::env::current_dir());

    for test in config.tests {
        println!("📝 {}", test.name);

        let succeeded = if !opts.skip_setup {
            if let Some(setup) = test.setup {
                match Command::new(setup).output() {
                    Ok(output) => {
                        if output.status.success() {
                            if let Ok(stdout) = String::from_utf8(output.stdout) {
                                println!("{}", stdout);
                            }
                            true
                        } else {
                            if let Ok(stderr) = String::from_utf8(output.stderr) {
                                eprintln!("{}", stderr);
                            }
                            eprintln!(
                                "❌ {} {}\n\n",
                                "Failed to set up test".red(),
                                test.name.red()
                            );
                            false
                        }
                    }
                    Err(error) => {
                        eprintln!(
                            "❌ {} {}\n\n",
                            "Failed to set up test".red(),
                            test.name.red()
                        );
                        eprintln!("{}", error.to_string().red());
                        false
                    }
                }
            } else {
                true
            }
        } else {
            true
        };

        let succeeded = if succeeded {
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
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    println!("{}", stdout);
                }
                println!("✅ {}\n\n", test.name);
                // Todo: Compare against output
                true
            } else {
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    eprintln!("{}", stderr);
                }
                eprintln!("❌ {}\n\n", test.name.red());
                false
            }
        } else {
            false
        };

        all_succeeded &= succeeded;
        if succeeded {
            if let Some(test_points) = test.points {
                points += test_points;
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
