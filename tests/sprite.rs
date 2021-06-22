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

#[test]
fn repl_copies_sprite() {
    let mut bench = TestBench::redshift_vm();
    bench.exa("copy 1 gx\n copy 2 gy\n copy 3 gz\n copy 200 gp\n mark r\n repl r\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    bench.run_cycle();
    let e2 = bench.get_exa("x0:1");
    bench.assert_exa_register(&e2, "gx", 1);
    bench.assert_exa_register(&e2, "gy", 2);
    bench.assert_exa_register(&e2, "gz", 3);
    bench.assert_exa_sprite(&e2, vec![0, 1, 99]);
}

#[test]
fn gp_manipulation() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 100 gp\n copy 110 gp\n copy 120 gp\n copy 000 gp\n copy 210 gp\n");

    bench.run_cycle();
    bench.assert_exa_sprite(&e1, vec![0, 1, 99]);
    bench.run_cycle();
    bench.assert_exa_sprite(&e1, vec![0, 2, 98]);
    bench.run_cycle();
    bench.assert_exa_sprite(&e1, vec![0, 3, 97]);
    bench.run_cycle();
    bench.assert_exa_sprite(&e1, vec![1, 2, 97]);
    bench.run_cycle();
    bench.assert_exa_sprite(&e1, vec![2, 1, 97]);
}
