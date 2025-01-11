# static-atomic-queue
Forked from [Augmented Audio Libraries](https://github.com/CodeSteak/static-atomic-queue/tree/master/crates/augmented/data/atomic-queue)

Multi-producer multi-consumer bounded lock-free queue for use in ~~Audio~~ embedded applications, ported from
https://github.com/max0x7ba/atomic_queue.

Changes: 
- All buffers for the queues are now statically allocated.
- `no_std` support.
- `atomic_queue::Queue::new()` is now const.

## Usage
```rust
static QUEUE: atomic_queue::Queue<usize, 23> = atomic_queue::Queue::new();

fn do_stuff() {
    QUEUE.push(42);
    if let Some(v) = QUEUE.pop() {
        assert_eq!(v, 42);
    }
}
```

## License
MIT
