mod common;

use common::*;

#[test]
fn pos_x() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 1 gx\n copy gx t\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "gx", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
}
