mod common;

use common::*;

#[test]
fn test_single_data() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("data 1 2 3\n file x\n noop\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 400);
    bench.assert_exa_file_contents(&e1, vec![1, 2, 3]);
}

#[test]
fn test_multiple_data() {
    let mut bench = TestBench::basic_vm();
    let e1 = bench.exa("data 1 2 3\n file x\n data 4 5 6\n noop\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 400);
    bench.assert_exa_file_contents(&e1, vec![1, 2, 3, 4, 5, 6]);
}
