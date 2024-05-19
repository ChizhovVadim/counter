# counter

Rust version of the [counter](https://github.com/ChizhovVadim/CounterGo) chess engine. Supports NNUE evaluation on Apple Silicon. Currently early alpha version, single thread search only.

You need to have [nnue file](https://github.com/ChizhovVadim/CounterGo/blob/master/pkg/eval/nnue/n-30-5268.nn) in working dir, binary dir or ~/chess dir.

## Motivation for migrate from go to rust
Go does not help you to use SIMD instructions. In the era of deep learning SIMD calculations play a big role. Rust supports auto-vectorization, so your code can be effective on different CPUs. For example, calculate dotProduct(RELU(hidden_outputs), wieghts):
```
	hidden_outputs
            .iter()
            .zip(weights.iter())
            .fold(0_f32, |acc, (&x, &w)| unsafe {
                std::intrinsics::fadd_fast(acc, x.max(0_f32) * w)
            });
```

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
