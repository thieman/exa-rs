mod parse {
    pub mod parse;
    mod parts;
    mod preprocess;
}

mod vm {
    pub mod instruction;
    pub mod level;
}

use parse::parse::parse_text;
use vm::level::*;

fn main() {
    let v = parse_text("LINK -1\n copy 1 x\naddi 1 1 t\n");
    println!("{:?}", v);

    let h1 = Host::new(String::from("one"), 2);

    let h2 = Host::new(String::from("two"), 2);

    let mut l = Level::new();

    let h1 = l.add_host(h1);
    let h2 = l.add_host(h2);

    l.add_link(800, h1, h2);
}
