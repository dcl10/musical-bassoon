use crate::queue::Queue;
use clap::Parser;
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

mod cli;
mod queue;

fn main() {
    let args = cli::Args::parse();

    println!("Using {:?} producers", args.n_producers);
    println!("Using {:?} consumers", args.n_consumers);
    println!("Using delay of {:?} milliseconds", args.delay);

    let queue = Arc::new(Queue::<&str>::new(100));

    loop {
        for _ in 0..args.n_producers {
            let producer = Arc::clone(&queue);
            thread::spawn(move || {
                let is_added = producer.enqueue("Hello, world!");
                match is_added {
                    Ok(_) => {
                        println!("Message added successfully!");
                    }
                    Err(err) => {
                        println!("Message failed with error: {:?}", err);
                    }
                }
            });
        }

        for _ in 0..args.n_consumers {
            let consumer = Arc::clone(&queue);
            thread::spawn(move || {
                let removed = consumer.dequeue();
                match removed {
                    Ok(item) => match item {
                        None => {}
                        Some(i) => {
                            println!("Dequeued message: {i}")
                        }
                    },
                    Err(err) => {
                        println!("Error determining message: {:?}", err);
                    }
                }
            });
        }
        sleep(Duration::from_millis(args.delay));
    }
}
