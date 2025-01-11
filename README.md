# static-atomic-queue
[![crates.io](https://img.shields.io/crates/v/atomic-queue.svg)](https://crates.io/crates/atomic-queue)
[![docs.rs](https://docs.rs/atomic-queue/badge.svg)](https://docs.rs/atomic-queue/)
- - -

Forked from [Augmented Audio Libraries](https://github.com/CodeSteak/static-atomic-queue/tree/master/crates/augmented/data/atomic-queue)

Multi-producer multi-consumer bounded lock-free queue for use in ~~Audio~~ embedded applications, ported from
https://github.com/max0x7ba/atomic_queue.

Changes: 
- Are buffers for the queues are now staticly allocated.

## License
MIT
