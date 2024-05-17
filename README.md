# counter

Rust version of the [counter](https://github.com/ChizhovVadim/CounterGo) chess engine. Supports NNUE evaluation on Apple Silicon. Currently early alpha version, single thread search only.

You need to have [nnue file](https://github.com/ChizhovVadim/CounterGo/blob/master/pkg/eval/nnue/n-30-5268.nn) in working dir, binary dir or ~/chess dir.

## Manual compilation
Install nightly rust version to use std::intrinsics::fadd_fast:
```
$ rustup toolchain install nightly
```
Use nightly for project:
```
$ cd ~/projects/counter
$ rustup override set nightly
```
Build binary:
```
$ cargo build --release
```
Or optimize for you processor with possible better performance: 
```
$ RUSTFLAGS='-C target-cpu=native' cargo build --release
```
