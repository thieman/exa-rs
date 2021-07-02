mod common;

use common::*;

#[test]
fn test_no_collision() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 100 gp\n noop\n");
    let e2 = bench.exa("copy 111 gp\n noop\n");

    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
}

#[test]
fn test_collision_and_reset() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy 1 co\n copy 100 gp\n copy 0 gp\n noop\n");
    let e2 = bench.exa("copy 2 co\n copy 100 gp\n noop\n noop\n");

    bench.run_cycle();
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
    bench.run_cycle();
    // collision detection happens before exa cycles
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
    bench.run_cycle();
    // now they collided
    bench.assert_exa_register(&e1, "ci", 2);
    bench.assert_exa_register(&e2, "ci", 1);
    bench.run_cycle();
    // reset after first sprite is zeroed out
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
}

#[test]
fn test_out_of_bounds_collision() {
    let mut bench = TestBench::redshift_vm();
    let e1 = bench.exa("copy -5 gx\n copy 1 co\n copy 100 gp\n copy 0 gp\n noop\n");
    let e2 = bench.exa("copy -5 gx\n copy 2 co\n copy 100 gp\n noop\n noop\n");

    bench.run_cycle();
    bench.run_cycle();
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
    bench.run_cycle();
    // collision detection happens before exa cycles
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
    bench.run_cycle();
    // now they collided
    bench.assert_exa_register(&e1, "ci", 2);
    bench.assert_exa_register(&e2, "ci", 1);
    bench.run_cycle();
    // reset after first sprite is zeroed out
    bench.assert_exa_register(&e1, "ci", -9999);
    bench.assert_exa_register(&e2, "ci", -9999);
}
