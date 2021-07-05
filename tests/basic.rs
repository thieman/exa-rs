/// Tests of generic Exa behavior, e.g. dying on errors
mod common;

use common::*;

#[test]
fn test_halt_kills_exa() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("halt\n");

    bench.assert_alive(&e);
    bench.run_cycle();
    bench.assert_alive(&e);
    bench.assert_fatal_error(&e);
    bench.run_cycle();
    bench.assert_dead(&e);
}

#[test]
fn test_out_of_instructions() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("noop\n");

    bench.assert_alive(&e);
    bench.run_cycle();
    bench.assert_alive(&e);
    bench.assert_fatal_error(&e);
    bench.run_cycle();
    bench.assert_dead(&e);
}

#[test]
fn test_empty_exa() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("\n");

    bench.run_cycle();
    bench.assert_fatal_error(&e);
}
