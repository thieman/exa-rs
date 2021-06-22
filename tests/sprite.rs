mod common;

use common::*;

#[test]
fn pos_x() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 1 gx\n copy gx t\n");
    // test clamps
    let e2 = bench.exa("copy 9999 gx\n copy gx t\n");
    let e3 = bench.exa("copy -9999 gx\n copy gx t\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "gx", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
    bench.assert_exa_register(&e2, "t", 120);
    bench.assert_exa_register(&e3, "t", -10);
}

#[test]
fn pos_y() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 1 gy\n copy gy t\n");
    // test clamps
    let e2 = bench.exa("copy 9999 gy\n copy gy t\n");
    let e3 = bench.exa("copy -9999 gy\n copy gy t\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "gy", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
    bench.assert_exa_register(&e2, "t", 100);
    bench.assert_exa_register(&e3, "t", -10);
}

#[test]
fn pos_z() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 1 gz\n copy gz t\n");
    // test clamps
    let e2 = bench.exa("copy 9999 gz\n copy gz t\n");
    let e3 = bench.exa("copy -9999 gz\n copy gz t\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "gz", 1);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "t", 1);
    bench.assert_exa_register(&e2, "t", 9);
    bench.assert_exa_register(&e3, "t", -9);
}
