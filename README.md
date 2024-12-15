# `default-vec2`

A simple `Vec`-like data structure with default elements and a bitset built using it.

It is `#![no-std]`, and `#![forbid(unsafe_code)]`, but does use `alloc`

## Comparisons

### vs [`default-vec`](https://crates.io/crates/default-vec)

* This crate does not use unsafe code or rely on the unstable `RawVec` type

### vs [`bit-set`](https://crates.io/crates/bit-set)

* This `BitSet` fits in 2 `usize`s while [`bit-set`](https://crates.io/crates/bit-set) takes 4.

### vs [`fixedbitset`](https://crates.io/crates/fixedbitset)

* This `BitSet` fits in 2 `usize`s while [`fixedbitset`](https://crates.io/crates/fixedbitset) takes 3.
* This `BitSet` will automatically increase its capacity as need
* This crate doesn't use any unsafe code

