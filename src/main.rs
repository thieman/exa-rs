extern crate exa;

use exa::vm::exa::Exa;
use exa::vm::VM;

fn main() {
    let mut vm = VM::new_redshift();
    let host1 = vm.hosts.get("core").unwrap().clone();
    let host2 = vm.hosts.get("core").unwrap().clone();

    Exa::spawn(
        &mut vm,
        host1,
        "x0".to_string(),
        true,
        "copy 301 gp\n wait\n ",
    )
    .expect("cannot spawn");

    Exa::spawn(
        &mut vm,
        host2,
        "x0".to_string(),
        true,
        "copy 302 gp\n mark a\n rand 0 100 gx\n jump a\n",
    )
    .expect("cannot spawn");

    loop {
        vm.run_cycle();
    }
}
