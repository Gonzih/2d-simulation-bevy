use clap::Parser;

/// Simulation core
#[derive(Parser, Debug, Default)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    ///Initial population size
    #[clap(short, long, default_value_t = 80)]
    pub population: usize,
}

pub fn parse() -> Args {
    Args::parse()
}
