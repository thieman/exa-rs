mod parse;
mod vm;

use parse::parse_text;
use vm::exa::Exa;
use vm::*;

fn main() {
    let v = parse_text("LINK -1\n copy 1 x\naddi 1 1 t\n");
    println!("{:?}", v);

    let h1 = Host::new_shared(String::from("one"), 2);

    let h2 = Host::new_shared(String::from("two"), 2);

    let mut vm = VM::new();

    vm.add_host(h1.clone());
    vm.add_host(h2.clone());

    vm.add_link(800, h1.clone(), h2.clone());

    println!("{:?}", vm);

    let exa1 = Exa::spawn(h1.clone(), String::from("X0"), "link 800\n");

    vm.run_cycle();
    vm.run_cycle();
}
