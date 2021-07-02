// The golden image is as follows:
//  - the image name is "GOLDEN IMAGE"
//  - the first exa is named AB
//    - mode global, maximized window
//    - sprite is empty
//    - script is "COPY 1 X\nNOOP"
//  - the second exa is named CD
//    - mode local, maximized window
//    - custom sprite, activated pixels are the 4 corners
//    - script is "NOTE I AM A GOLDEN GOD\nHALT"

mod common;

use common::*;

#[test]
fn test_golden() {
    let mut bench = TestBench::redshift_vm_from_image("./tests/golden.png");

    let e1 = bench.get_exa("AB");
    let e2 = bench.get_exa("CD");

    bench.assert_exa_global_mode(&e1);
    bench.assert_exa_local_mode(&e2);

    bench.assert_exa_sprite(&e2, vec![0, 1, 8, 1, 80, 1, 8, 1]);
    bench.run_cycle();
    bench.assert_exa_register(&e1, "x", 1);
}
