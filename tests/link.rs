mod common;

use common::*;

#[test]
fn basic_link() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("link 800\nlink -1\n");

    bench.assert_position(&e, "start");
    bench.run_cycle();
    bench.assert_position(&e, "end");
    bench.run_cycle();
    bench.assert_position(&e, "start");
}

#[test]
fn invalid_link_errors() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("link -999\n");

    bench.assert_position(&e, "start");
    bench.run_cycle();
    bench.assert_position(&e, "start");
    bench.assert_fatal_error(&e);
}

#[test]
fn link_from_exa_register() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("copy 800 x\ncopy -1 t\nlink x\nlink t\n");

    bench.assert_position(&e, "start");
    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    bench.assert_position(&e, "end");
    bench.run_cycle();
    bench.assert_position(&e, "start");
}

#[test]
fn link_from_hardware_register() {
    let mut bench = TestBench::basic_vm();
    let e = bench.exa("copy 800 #REG\nlink #REG\n");

    bench.assert_position(&e, "start");
    bench.run_cycle();
    bench.run_cycle();
    bench.assert_position(&e, "end");
}
