use crate::queue::Queue;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub(crate) fn spawn_produce_pool(
    queue: Arc<Queue<String>>,
    port: u16,
    n_threads: usize,
) -> Vec<JoinHandle<()>> {
    spawn_pool(queue, port, n_threads, handle_produce_connection)
}

pub(crate) fn spawn_consume_pool(
    queue: Arc<Queue<String>>,
    port: u16,
    n_threads: usize,
) -> Vec<JoinHandle<()>> {
    spawn_pool(queue, port, n_threads, handle_consume_connection)
}

fn spawn_pool(
    queue: Arc<Queue<String>>,
    port: u16,
    n_threads: usize,
    handler: fn(TcpStream, &Queue<String>),
) -> Vec<JoinHandle<()>> {
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|err| {
        eprintln!("failed to bind port {port}: {err}");
        std::process::exit(1);
    });
    let listener = Arc::new(listener);

    (0..n_threads)
        .map(|_| {
            let listener = Arc::clone(&listener);
            let queue = Arc::clone(&queue);
            thread::spawn(move || accept_loop(listener, queue, handler))
        })
        .collect()
}

fn accept_loop(listener: Arc<TcpListener>, queue: Arc<Queue<String>>, handler: fn(TcpStream, &Queue<String>)) -> ! {
    loop {
        match listener.accept() {
            Ok((stream, _addr)) => handler(stream, &queue),
            Err(err) => eprintln!("accept error: {err}"),
        }
    }
}

fn handle_produce_connection(stream: TcpStream, queue: &Queue<String>) {
    let mut line = String::new();
    let bytes_read = match BufReader::new(&stream).read_line(&mut line) {
        Ok(n) => n,
        Err(err) => {
            eprintln!("read error: {err}");
            return;
        }
    };

    if bytes_read == 0 {
        return;
    }

    let message = line.trim_end_matches(['\n', '\r']);
    if message.is_empty() {
        let _ = (&stream).write_all(b"ERR empty message\n");
        return;
    }

    match queue.enqueue(message.to_string()) {
        Ok(()) => {
            let _ = (&stream).write_all(b"OK\n");
        }
        Err(err) => {
            eprintln!("enqueue error: {err}");
            let _ = (&stream).write_all(b"ERR internal\n");
        }
    }
}

fn handle_consume_connection(stream: TcpStream, queue: &Queue<String>) {
    match queue.dequeue() {
        Ok(Some(message)) => {
            let _ = (&stream).write_all(format!("{message}\n").as_bytes());
        }
        Ok(None) => {
            let _ = (&stream).write_all(b"EMPTY\n");
        }
        Err(err) => {
            eprintln!("dequeue error: {err}");
            let _ = (&stream).write_all(b"ERR internal\n");
        }
    }
}
