use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short = 'p', long = "Number of producers", default_value = "1")]
    pub(crate) n_producers: usize,
    #[arg(short = 'c', long = "Number of consumers", default_value = "1")]
    pub(crate) n_consumers: usize,
}