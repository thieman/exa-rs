mod common;

use common::*;

#[test]
fn rand_equal() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("rand 3 3 x\n noop\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 3);
}

#[test]
fn rand_range() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("rand 1 3 x\n test x > 0\n test x < 4\n noop");

    bench.run_cycle();
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
}

#[test]
fn rand_invalid() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("rand 0 -1 x\n noop\n");

    bench.run_cycle();
    bench.assert_fatal_error(&e1);
}
