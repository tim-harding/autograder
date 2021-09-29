use anyhow::Result;
use clap::Clap;
use colored::Colorize;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;

#[derive(Clap, Debug, Clone, Hash, PartialEq, Eq)]
struct Options {
    #[clap(short, long)]
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

    for test in config.tests {
        println!("📝 {}", test.name);

        let succeeded = if let Some(setup) = test.setup {
            match Command::new(setup).output() {
                Ok(output) => {
                    if output.status.success() {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            println!("{}", stdout);
                        }
                        true
                    } else {
                        if let Ok(stderr) = String::from_utf8(output.stderr) {
                            println!("{}", stderr);
                        }
                        println!("❌ {} {}\n", "Failed to set up test".red(), test.name.red());
                        false
                    }
                }
                Err(error) => {
                    println!("❌ {} {}\n", "Failed to set up test".red(), test.name.red());
                    println!("{}", error.to_string().red());
                    false
                }
            }
        } else {
            true
        };

        let succeeded = if succeeded {
            match Command::new(test.run).output() {
                Ok(output) => {
                    if output.status.success() {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            println!("{}", stdout);
                        }
                        println!("✅ {}\n", test.name.green());
                        true
                    } else {
                        if let Ok(stderr) = String::from_utf8(output.stderr) {
                            println!("{}", stderr);
                        }
                        println!("❌ {}\n", test.name.red());
                        false
                    }
                }
                Err(error) => {
                    println!("❌ {}\n{}", test.name.red(), error.to_string().red());
                    false
                }
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
