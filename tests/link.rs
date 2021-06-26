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

#[test]
fn one_directional_blocking_bandwidth() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("link 800\n");
    let e2 = bench.exa("link 800\n");

    bench.run_cycle();
    bench.assert_position(&e1, "end");
    bench.assert_position(&e2, "start");
    bench.assert_blocking_error(&e2);
    bench.run_cycle();
    bench.assert_position(&e2, "end");
}

#[test]
fn two_directional_blocking_bandwidth() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("noop\nlink 800\n");
    let e2 = bench.exa("link 800\nlink -1\n");

    bench.run_cycle();
    bench.assert_position(&e1, "start");
    bench.assert_position(&e2, "end");
    bench.run_cycle();
    bench.assert_position(&e1, "end");
    bench.assert_position(&e2, "end");
    bench.assert_blocking_error(&e2);
    bench.run_cycle();
    bench.assert_position(&e2, "start");
}

#[test]
fn test_redshift_links() {
    let mut bench = TestBench::redshift_vm();
    let e1 =
        bench.exa("link 800\n link -1\n link 801\n link -1 \n link 802 \n link -1\n link 803 \n");

    for _ in 0..=5 {
        bench.run_cycle();
        bench.assert_no_error(&e1);
    }
}
