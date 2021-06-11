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
