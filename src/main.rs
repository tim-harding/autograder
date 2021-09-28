use clap::{Clap};

#[derive(Clap, Debug, Clone, Hash, PartialEq, Eq)]
struct Options {
    #[clap(short, long)]
    config: String,
    #[clap(short, long)]
    program: String,
}

fn main() {
    let opts: Options = Options::parse();
}
