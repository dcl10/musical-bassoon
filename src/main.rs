use clap::Parser;

mod cli;

fn main() {
    let args = cli::Args::parse();

    println!("Using {:?} producers", args.n_producers);
    println!("Using {:?} consumers", args.n_consumers);
}
