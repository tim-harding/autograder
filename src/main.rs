use clap::{Clap};
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;

#[derive(Clap, Debug, Clone, Hash, PartialEq, Eq)]
struct Options {
    #[clap(short, long)]
    config: String,
    #[clap(short, long)]
    program: String,
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
    Ok(())
}
