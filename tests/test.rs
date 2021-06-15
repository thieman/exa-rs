mod common;

use common::*;

#[test]
fn test_equal() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("test 5 = 5\n");
    let e2 = bench.exa("test 4 = 5\n");
    let e3 = bench.exa("copy 1 x\n test x = 1\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
    bench.assert_exa_register(&e2, "t", 0);
    bench.run_cycle();
    bench.assert_exa_register(&e3, "t", 1);
}

#[test]
fn test_greater() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("test 5 > 5\n");
    let e2 = bench.exa("test 5 > 4\n");
    let e3 = bench.exa("copy 1 x\n test x > 0\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 0);
    bench.assert_exa_register(&e2, "t", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e3, "t", 1);
}

#[test]
fn test_lesser() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("test 5 < 5\n");
    let e2 = bench.exa("test 4 < 5\n");
    let e3 = bench.exa("copy 1 x\n test x < 0\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 0);
    bench.assert_exa_register(&e2, "t", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e3, "t", 0);
}
