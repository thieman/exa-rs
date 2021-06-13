mod common;

use common::*;

#[test]
fn addi() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("addi 4 -10 x\n");
    let e2 = bench.exa("addi 5000 5000 x\n");
    let e3 = bench.exa("addi -9999 -9999 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", -6);
    bench.assert_exa_register(&e2, "x", 9999);
    bench.assert_exa_register(&e3, "x", -9999);
}

#[test]
fn subi() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("subi 4 -10 x\n");
    let e2 = bench.exa("subi 5000 -5000 x\n");
    let e3 = bench.exa("subi -9999 -9999 x\n");
    let e4 = bench.exa("subi -5000 5000 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 14);
    bench.assert_exa_register(&e2, "x", 9999);
    bench.assert_exa_register(&e3, "x", 0);
    bench.assert_exa_register(&e4, "x", -9999);
}

#[test]
fn muli() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("muli 4 -10 x\n");
    let e2 = bench.exa("muli 5000 -5000 x\n");
    let e3 = bench.exa("muli -9999 0 x\n");
    let e4 = bench.exa("muli -5000 5000 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", -40);
    bench.assert_exa_register(&e2, "x", -9999);
    bench.assert_exa_register(&e3, "x", 0);
    bench.assert_exa_register(&e4, "x", -9999);
}

#[test]
fn divi() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("divi 40 4 x\n");
    let e2 = bench.exa("divi 40 -4 x\n");
    let e3 = bench.exa("divi -9999 0 x\n");
    let e4 = bench.exa("divi -9999 -3 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 10);
    bench.assert_exa_register(&e2, "x", -10);
    bench.assert_fatal_error(&e3);
    bench.assert_exa_register(&e4, "x", 3333);
}

#[test]
fn divi_rounds_towards_zero() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("divi 41 4 x\n");
    let e2 = bench.exa("divi 39 4 x\n");
    let e3 = bench.exa("divi -41 4 x\n");
    let e4 = bench.exa("divi -39 4 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 10);
    bench.assert_exa_register(&e2, "x", 9);
    bench.assert_exa_register(&e3, "x", -10);
    bench.assert_exa_register(&e4, "x", -9);
}
