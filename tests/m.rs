/// Tests of M register behavior
extern crate exa;

mod common;

use common::*;
use exa::vm::exa::Mode;

#[test]
fn test_simple_m_passing() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy 1 m\n");
    let e2 = bench.exa("copy m x\n");

    bench.run_cycle();
    bench.assert_freezing_error(&e1);
    bench.assert_blocking_error(&e2);
    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_fatal_error(&e2);
    bench.assert_exa_register(&e2, "x", 1);
    bench.run_cycle();
    bench.assert_dead(&e1);
    bench.assert_dead(&e2);
}

#[test]
fn test_multi_mode_m_passing() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy 1 m\n");
    let e2 = bench.exa("copy m x\n");
    let e3 = bench.exa_custom("copy 2 m\n", "start", Mode::Local);
    let e4 = bench.exa_custom("copy m t\n", "start", Mode::Local);

    bench.run_cycle();
    bench.assert_freezing_error(&e1);
    bench.assert_blocking_error(&e2);
    bench.assert_freezing_error(&e3);
    bench.assert_blocking_error(&e4);

    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_fatal_error(&e2);
    bench.assert_fatal_error(&e3);
    bench.assert_fatal_error(&e4);
    bench.assert_exa_register(&e2, "x", 1);
    bench.assert_exa_register(&e4, "t", 2);

    bench.run_cycle();
    bench.assert_dead(&e1);
    bench.assert_dead(&e2);
    bench.assert_dead(&e3);
    bench.assert_dead(&e4);
}

#[test]
fn test_exceeds_bandwidth_message_bus_global() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy 1 m\n");
    let e2 = bench.exa("copy m x\n");
    let e3 = bench.exa("copy 2 m\n");
    let e4 = bench.exa("copy m x\n");

    bench.run_cycle();
    bench.assert_freezing_error(&e1);
    bench.assert_blocking_error(&e2);
    bench.assert_freezing_error(&e3);
    bench.assert_blocking_error(&e4);
    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_fatal_error(&e2);
    bench.assert_freezing_error(&e3);
    bench.assert_blocking_error(&e4);
    bench.assert_exa_register(&e2, "x", 1);

    bench.run_cycle();
    bench.assert_fatal_error(&e3);
    bench.assert_fatal_error(&e4);
    bench.assert_exa_register(&e4, "x", 2);
}

#[test]
fn test_multiple_locals() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa_custom("copy 1 m\n", "end", Mode::Local);
    let e2 = bench.exa_custom("copy m x\n", "end", Mode::Local);
    let e3 = bench.exa_custom("copy 2 m\n", "start", Mode::Local);
    let e4 = bench.exa_custom("copy m x\n", "start", Mode::Local);

    bench.run_cycle();
    bench.assert_freezing_error(&e1);
    bench.assert_blocking_error(&e2);
    bench.assert_freezing_error(&e3);
    bench.assert_blocking_error(&e4);
    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_fatal_error(&e2);
    bench.assert_fatal_error(&e3);
    bench.assert_fatal_error(&e4);
    bench.assert_exa_register(&e2, "x", 1);
    bench.assert_exa_register(&e4, "x", 2);
}
