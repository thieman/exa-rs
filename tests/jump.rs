mod common;

use common::*;

#[test]
fn test_jump() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("jump l\n halt\n mark l\n copy 1 x\n");

    bench.run_cycle();
    bench.assert_no_error(&e);
    bench.run_cycle();
    bench.assert_exa_register(&e, "x", 1);
    bench.run_cycle();
    bench.assert_dead(&e);
}

#[test]
fn test_tjmp() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("tjmp l\n halt\n mark l\n copy 1 x\n");

    bench.run_cycle();
    bench.assert_no_error(&e);
    bench.run_cycle();
    bench.assert_fatal_error(&e);
    bench.run_cycle();
    bench.assert_dead(&e);
}

#[test]
fn test_tjmp_succeeds() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("copy 1 t\n tjmp l\n halt\n mark l\n copy 1 x\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.assert_no_error(&e);
    bench.run_cycle();
    bench.assert_exa_register(&e, "x", 1);
    bench.run_cycle();
    bench.assert_dead(&e);
}

#[test]
fn test_fjmp() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("fjmp l\n halt\n mark l\n copy 1 x\n");

    bench.run_cycle();
    bench.assert_no_error(&e);
    bench.run_cycle();
    bench.assert_exa_register(&e, "x", 1);
    bench.run_cycle();
    bench.assert_dead(&e);
}
