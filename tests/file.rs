mod common;

use common::*;

#[test]
fn file_handling() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("make\n drop\n grab 400\n wipe\n noop");

    bench.assert_exa_no_file(&e1);
    bench.run_cycle();
    bench.assert_exa_file(&e1, 400);
    bench.run_cycle();
    bench.assert_exa_no_file(&e1);
    bench.assert_host_file("start", 400);
    bench.run_cycle();
    bench.assert_exa_file(&e1, 400);
    bench.assert_host_no_file("start", 400);
    bench.run_cycle();
    bench.assert_exa_no_file(&e1);
    bench.assert_host_no_file("start", 400);
}

#[test]
fn make_error() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("make\n make\n noop\n");

    bench.run_cycle();
    bench.assert_no_error(&e1);
    bench.run_cycle();
    bench.assert_fatal_error(&e1);
}

#[test]
fn wipe_error() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("wipe\n noop\n");

    bench.run_cycle();
    bench.assert_fatal_error(&e1);
}

#[test]
fn drop_error() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("drop\n noop\n");

    bench.run_cycle();
    bench.assert_fatal_error(&e1);
}

#[test]
fn grab_error() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("grab 10\n noop\n");

    bench.run_cycle();
    bench.assert_fatal_error(&e1);
}