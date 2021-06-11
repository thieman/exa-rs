use std::cell::RefCell;
use std::rc::Rc;

mod parse;
mod vm;

use parse::parse_text;
use vm::exa::Exa;
use vm::register::Register;
use vm::*;

fn main() {
    let v = parse_text("LINK -1\n copy 1 x\naddi 1 1 t\n");
    println!("{:?}", v);

    let h1 = Host::new_shared(String::from("one"), 2);
    let r = Register::new(Permissions::ReadWrite, 0);
    h1.borrow_mut()
        .add_register(String::from("#REG"), Rc::new(RefCell::new(r)));
    let h2 = Host::new_shared(String::from("two"), 2);

    let mut vm = VM::new();

    vm.add_host(h1.clone());
    vm.add_host(h2.clone());

    vm.add_link(800, h1.clone(), h2.clone());

    let exa1 = Exa::spawn(
        &mut vm,
        h1.clone(),
        String::from("X0"),
        "link 800\ncopy 1 x\n",
    );

    println!("{}", vm);
    vm.run_cycle();
    vm.run_cycle();
}
