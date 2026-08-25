use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short = 'p', long = "producers", default_value = "1")]
    pub(crate) n_producers: usize,
    #[arg(short = 'c', long = "consumers", default_value = "1")]
    pub(crate) n_consumers: usize,
    #[arg(short = 'P', long = "produce-port", default_value = "7878")]
    pub(crate) produce_port: u16,
    #[arg(short = 'C', long = "consume-port", default_value = "7879")]
    pub(crate) consume_port: u16,
}
