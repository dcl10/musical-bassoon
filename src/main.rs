use crate::queue::Queue;
use clap::Parser;
use std::sync::Arc;

mod cli;
mod queue;
mod server;

fn main() {
    let args = cli::Args::parse();

    println!("Using {:?} producer threads", args.n_producers);
    println!("Using {:?} consumer threads", args.n_consumers);
    println!("Listening for producers on 127.0.0.1:{}", args.produce_port);
    println!("Listening for consumers on 127.0.0.1:{}", args.consume_port);

    let queue = Arc::new(Queue::<String>::new(100));

    let mut handles = Vec::new();
    handles.extend(server::spawn_produce_pool(
        Arc::clone(&queue),
        args.produce_port,
        args.n_producers,
    ));
    handles.extend(server::spawn_consume_pool(
        Arc::clone(&queue),
        args.consume_port,
        args.n_consumers,
    ));

    for handle in handles {
        let _ = handle.join();
    }
}
