#![allow(dead_code)]
#![feature(float_algebraic)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::needless_return)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::collapsible_if)]

mod chess;
mod domain;
mod engine;
mod eval;
mod tests;
mod uci;

fn main() {
    unsafe { chess::init() };
    if tests::test_handler() {
        return;
    }
    let mut eng = engine::Engine::new();
    uci::run(&mut eng);
}
