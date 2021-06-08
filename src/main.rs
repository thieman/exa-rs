mod parse {
    pub mod parse;
    mod parts;
    mod preprocess;
}

mod vm {
    pub mod instruction;
    pub mod vm;
}

use parse::parse::parse_text;
use vm::vm::*;

fn main() {
    let v = parse_text("LINK -1\n copy 1 x\naddi 1 1 t\n");
    println!("{:?}", v);

    let h1 = Host::new_shared(String::from("one"), 2);

    let h2 = Host::new_shared(String::from("two"), 2);

    let mut vm = VM::new();

    vm.add_host(h1.clone());
    vm.add_host(h2.clone());

    vm.add_link(800, h1, h2);

    println!("{:?}", vm);
}
