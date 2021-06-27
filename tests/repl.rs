mod common;

use common::*;

#[test]
fn test_repl() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("mark start\n copy 1 x \n copy 2 t\n repl start\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    let e2 = bench.get_exa("x0:1");
    bench.assert_exa_register(&e2, "x", 1);
    bench.assert_exa_register(&e2, "t", 2);
    bench.assert_fatal_error(&e1);
    bench.assert_no_error(&e2);
}

#[test]
fn test_chain_repl() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("mark start\n noop \n repl start\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    bench.assert_dead(&e1);
    let e2 = bench.get_exa("x0:2");
    bench.assert_no_error(&e2);
}

#[test]
fn test_repl_blocks_when_full() {
    let mut bench = TestBench::basic_vm();
    let _ = bench.exa("noop \n noop\n");
    let _ = bench.exa("noop \n noop\n");
    let _ = bench.exa("noop \n noop\n");
    let _ = bench.exa("noop \n noop\n");
    let e5 = bench.exa("mark start\n repl start\n");

    bench.run_cycle();
    bench.assert_blocking_error(&e5);
}

/// Regression test for EXAs passing references, rather than fresh clones,
/// of their registers to their descendants upon REPL
#[test]
fn test_repl_independent_registers() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("mark start\n copy 1 x \n repl start \n copy 2 x\n noop\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    let e2 = bench.get_exa("x0:1");
    bench.assert_exa_register(&e1, "x", 2);
    bench.assert_exa_register(&e2, "x", 1);
}
