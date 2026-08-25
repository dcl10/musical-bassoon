# Rust In-Memory Message Queue

A bounded, in-memory FIFO message queue supporting multiple concurrent
producers and consumers, built as a systems-programming learning project
in Rust. It runs as a small TCP-based message broker: producers and
consumers reach the queue over the network rather than only from
within the process.

## Overview

This project implements a thread-safe message queue with:

- **Configurable producer thread count** — the number of persistent
  worker threads accepting incoming produce connections and writing
  messages into the queue.
- **Configurable consumer thread count** — the number of persistent
  worker threads accepting incoming consume connections and reading
  messages off the queue.
- A shared, mutually-exclusive internal buffer (`Mutex<VecDeque<T>>`)
  so concurrent modification is safe.
- Shared ownership of the queue across all threads via `Arc<Queue<T>>`.

## Configurable Thread Counts

Both producer and consumer thread counts are exposed as parameters
(e.g. via CLI flags or constructor arguments) rather than hardcoded,
because the "correct" number of threads is not a constant — it depends
on the workload:

| Factor | Effect on thread count |
| --- | --- |
| Lock contention on the shared buffer | Past a small number of threads, additional threads spend more time waiting on the `Mutex` than doing useful work — throughput can plateau or regress. |
| Producer:consumer rate mismatch | If producers outpace consumers (or vice versa), skewing the ratio helps, up to the point where the added threads start contending with each other. |

## Network Protocol

The server listens on two separate TCP ports (defaults `7878` for
producing, `7879` for consuming), plain newline-delimited text, no
new dependencies beyond `clap`:

- **Produce**: connect to the produce port and send one line of text.
  The server enqueues it (blocking if the queue is full) and responds
  with `OK\n`, or `ERR ...\n` on failure.
- **Consume**: connect to the consume port. The server blocks until a
  message is available, then writes it back as one line (`<message>\n`).
  An empty queue is a valid state, reported as `EMPTY\n`.

One message per connection — the connection closes after the
response.

## Why `Mutex<VecDeque<T>>`

The queue's internal buffer needs **interior mutability with
synchronization**: multiple threads must be able to push and pop from
the same `VecDeque<T>` without two threads mutating it at the same
moment.

Rust's ownership rules alone won't allow this. The compiler's aliasing
rule is: at any given time, you may have *either* one mutable reference
*or* any number of shared references to a value — never both. Plain
shared access (`&VecDeque<T>`) is read-only and can't be used to push
or pop; genuine mutation requires `&mut VecDeque<T>`, and the borrow
checker cannot guarantee at compile time that only one thread will
ever hold that mutable access at a time, because thread scheduling is
a runtime property the compiler can't see.

`Mutex<T>` resolves this by moving the guarantee from compile time to
run time: it hands out mutable access via a lock guard
(`MutexGuard<T>`), and the OS/runtime ensures only one thread holds
that guard at once. Every other thread attempting to lock blocks until
the guard is dropped. This is what makes concurrent modification of
the same `VecDeque<T>` safe.

Only the buffer is wrapped in the `Mutex` — not the whole `Queue`
struct. This is because anything inside the lock boundary is what threads
will contend over. Fields that don't need synchronized mutation (e.g.
a fixed `capacity` set once at construction) stay outside it, since
locking them would create contention with no correctness benefit.

## Why `Arc<Queue<T>>`

`Mutex` solves *mutation* safety, but there's a separate problem: every
producer and consumer thread needs its own handle to the *same*
`Queue<T>` instance. Ownership in Rust is otherwise single-owner by
default — moving a value into one `thread::spawn` closure means no
other thread (or the main thread) can hold it afterward. Passing
`Queue<T>` by value to multiple threads simply doesn't compile: it's
an ownership violation, not a mutability one.

`Arc<T>` (atomically reference-counted pointer) solves this by
providing **shared ownership** of a single heap allocation. Cloning an
`Arc` doesn't clone the underlying data — it atomically increments a
reference count and hands back a new pointer to the same allocation.
The data is only actually dropped once the last `Arc` clone is gone.
This satisfies the borrow checker because each thread now legitimately
owns *something* (its own `Arc` clone), rather than trying to share a
single owned value across threads.

```rust
let queue = Arc::new(Queue::new(capacity));

let producer_queue = Arc::clone(&queue);
thread::spawn(move || {
    producer_queue.push(item); // locks Mutex internally, only for the push
});

let consumer_queue = Arc::clone(&queue);
thread::spawn(move || {
    consumer_queue.pop(); // locks Mutex internally, only for the pop
});
```

## How `Arc<Mutex<T>>` Together Satisfy Rust's Safety Rules

`Arc` and `Mutex` address two different halves of the same problem,
and both are required together for this design:

- **`Arc<Queue<T>>`** satisfies Rust's *ownership* rules: it lets
  multiple threads each legitimately own a reference to the same
  `Queue`, instead of violating the single-owner rule by trying to
  move or borrow one value across several threads.
- **`Mutex<VecDeque<T>>`** satisfies Rust's *aliasing/data-race*
  rules: it enforces — at runtime, via locking — the "one mutator at a
  time" guarantee that the compiler can't verify on its own for
  concurrently-running threads.

Neither substitutes for the other. `Arc` alone would only ever hand out
shared references (`&T`), which isn't enough to mutate the buffer.
`Mutex` alone, without `Arc`, can't be shared across threads in the
first place because of the ownership violation described above. Used
together, they let the compiler enforce Rust's core guarantee, no
data races, checked at compile time wherever possible and pushed to a
runtime lock only where genuinely necessary, without requiring
`unsafe` anywhere in the queue implementation.

## Project Structure (WIP)

- `Queue<T>` — core FIFO buffer. `enqueue` blocks via `Condvar` when
  full; `dequeue` never blocks — it returns `None` immediately if the
  queue is empty, since an empty queue is a valid state, not an error.
- Producer/consumer thread pools — spawned per the configured counts,
  each holding an `Arc::clone` of the queue.
- `server` — the TCP accept-loop/thread-pool layer that connects
  incoming produce/consume connections to the queue.
