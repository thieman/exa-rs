mod parse {
    pub mod parse;
    mod parts;
    mod preprocess;
}

mod vm {
    pub mod instruction;
}

use parse::parse::parse_text;

fn main() {
    let v = parse_text("LINK -1\n copy 1 x\naddi 1 1 t\n");
    println!("{:?}", v);
}
