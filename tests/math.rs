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

#[test]
fn modi() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("modi 4 4 x\n");
    let e2 = bench.exa("modi 5 4 x\n");
    let e3 = bench.exa("modi -4 4 x\n");
    let e4 = bench.exa("modi -5 4 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 0);
    bench.assert_exa_register(&e2, "x", 1);
    bench.assert_exa_register(&e3, "x", 0);
    bench.assert_exa_register(&e4, "x", 3);
}

#[test]
fn swiz() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("swiz -1579 0032 x\n");
    let e2 = bench.exa("swiz 1234 1234 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", -57);
    bench.assert_exa_register(&e2, "x", 4321);
}

#[test]
fn swiz_sign() {
    let mut bench = TestBench::basic_vm();
    let e3 = bench.exa("swiz 12 1234 x\n");
    let e4 = bench.exa("swiz -12 1234 x\n");
    let e5 = bench.exa("swiz 12 -1234 x\n");
    let e6 = bench.exa("swiz -12 -1234 x\n");

    bench.run_cycle();
    bench.assert_exa_register(&e3, "x", 2100);
    bench.assert_exa_register(&e4, "x", -2100);
    bench.assert_exa_register(&e5, "x", -2100);
    bench.assert_exa_register(&e6, "x", 2100);
}

#[test]
fn modi_neg() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy 80 t\n modi -1 t t\n modi -1 t t\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 80);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 79);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 78);
}
