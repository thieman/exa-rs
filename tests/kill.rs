mod common;

use common::*;

#[test]
fn kill_noop() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("kill\n copy 1 x\n");

    bench.run_cycle();
    bench.assert_no_error(&e1);
    bench.assert_alive(&e1);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 1);
}

#[test]
fn kill_killers() {
    let mut bench = TestBench::basic_vm();
    let e3 = bench.exa("noop\n noop\n");
    let e1 = bench.exa("kill\n noop\n");
    let e2 = bench.exa("kill\n noop\n");

    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_fatal_error(&e2);
    bench.assert_no_error(&e3);
    bench.run_cycle();
    bench.assert_dead(&e1);
    bench.assert_dead(&e2);
    bench.assert_alive(&e3);
}

#[test]
fn kill_prioritize_descendants() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("repl end\n kill\n mark end\n noop\n noop\n");
    let e2 = bench.exa("noop\n noop\n noop\n");

    bench.run_cycle();
    let e3 = bench.get_exa("x0:1");
    bench.run_cycle();
    bench.assert_fatal_error(&e3);
    bench.assert_no_error(&e2);
}

#[test]
fn kill_prioritize_ancestors() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("repl end\n noop\n noop\n mark end\n kill\n noop\n");
    let e2 = bench.exa("noop\n noop\n noop\n");

    bench.run_cycle();
    let e3 = bench.get_exa("x0:1");
    bench.run_cycle();
    bench.assert_fatal_error(&e1);
    bench.assert_no_error(&e2);
}
