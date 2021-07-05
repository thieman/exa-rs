/// Tests of M register behavior
extern crate exa;

mod common;

use common::*;
use exa::vm::exa::Mode;

#[test]
fn simple_m_passing() {
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

/// Regression test for message bus unfreezing causing crash
/// based on order that relevant EXAs were created in
#[test]
fn simple_m_passing_order() {
    let mut bench = TestBench::basic_vm();
    let e2 = bench.exa("copy m x\n");
    let e1 = bench.exa("copy 1 m\n");

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
fn multi_mode_m_passing() {
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
fn exceeds_bandwidth_message_bus_global() {
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
fn multiple_locals() {
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

#[test]
fn mode() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("noop\n copy 1 m\n");
    let e2 = bench.exa_custom("mode\n copy m x\n", "start", Mode::Local);

    bench.run_cycle();
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
fn void_m() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy 1 m\n");
    let e2 = bench.exa("void m\n");

    bench.run_cycle();
    bench.assert_freezing_error(&e1);
    bench.assert_blocking_error(&e2);
    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_fatal_error(&e2);
    bench.assert_exa_register(&e2, "x", 0);
    bench.run_cycle();
    bench.assert_dead(&e1);
    bench.assert_dead(&e2);
}

#[test]
fn test_mrd_immediately_available() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy 1 m\n noop\n");
    let e2 = bench.exa("test mrd\n noop\n");

    bench.run_cycle();
    bench.assert_freezing_error(&e1);
    bench.assert_no_error(&e2);
    bench.assert_exa_register(&e2, "t", 1);
}

#[test]
fn test_mrd_not_ready() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("copy -1 t\n test mrd\n noop\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.assert_no_error(&e1);
    bench.assert_exa_register(&e1, "t", 0);
}

#[test]
fn test_mrd_after_read() {
    let mut bench = TestBench::basic_vm();
    let _e1 = bench.exa("copy 1 m\n noop\n");
    let _e2 = bench.exa("copy 1 m\n noop\n");
    let e3 = bench.exa("noop\n copy m x\n noop\n");
    let e4 = bench.exa("test mrd\n test mrd\n test mrd\n noop\n");

    bench.run_cycle();
    bench.assert_exa_register(&e4, "t", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e3, "x", 1);
    bench.assert_exa_register(&e4, "t", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e4, "t", 1);
}

// Regression test for a hilarious bug where, if you ever ran
// TEST MRD in your program, we'd helpfully set your T value
// to the TEST MRD result every cycle for the rest of eternity.
#[test]
fn test_mrd_stops_affecting_t_register() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("test mrd\n copy 5 t\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 5);
}
