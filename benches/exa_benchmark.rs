use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::cell::RefCell;
use std::rc::Rc;

use exa::vm::exa::Exa;
use exa::vm::register::Register;
use exa::vm::{Host, Permissions, VM};

pub fn copy_register_loop(c: &mut Criterion) {
    let h1 = Host::new_shared(String::from("start"), 4);
    let r = Register::new(Permissions::ReadWrite, 100);
    h1.borrow_mut()
        .add_register(String::from("#REG"), Rc::new(RefCell::new(r)));

    let h2 = Host::new_shared(String::from("end"), 4);

    let mut vm = VM::new();

    vm.add_host(h1.clone());
    vm.add_host(h2.clone());

    vm.add_link(800, h1.clone(), h2.clone());
    vm.add_link(-1, h2.clone(), h1.clone());

    let name = "start";
    let host = vm.hosts.get(name).unwrap().clone();
    let name = String::from("x0");

    Exa::spawn(&mut vm, host, name, false, "mark a\n copy 1 x\n jump a\n").unwrap();

    c.bench_function("copy register loop", |b| {
        b.iter(|| black_box(vm.run_cycle()))
    });
}

pub fn rand_loop(c: &mut Criterion) {
    let h1 = Host::new_shared(String::from("start"), 4);
    let r = Register::new(Permissions::ReadWrite, 100);
    h1.borrow_mut()
        .add_register(String::from("#REG"), Rc::new(RefCell::new(r)));

    let h2 = Host::new_shared(String::from("end"), 4);

    let mut vm = VM::new();

    vm.add_host(h1.clone());
    vm.add_host(h2.clone());

    vm.add_link(800, h1.clone(), h2.clone());
    vm.add_link(-1, h2.clone(), h1.clone());

    let name = "start";
    let host = vm.hosts.get(name).unwrap().clone();
    let name = String::from("x0");

    Exa::spawn(
        &mut vm,
        host,
        name,
        false,
        "mark a\n rand 1 100 x\n jump a\n",
    )
    .unwrap();

    c.bench_function("rand loop", |b| b.iter(|| black_box(vm.run_cycle())));
}

pub fn rand_gx_with_sprite_defined(c: &mut Criterion) {
    let h1 = Host::new_shared(String::from("start"), 4);
    let r = Register::new(Permissions::ReadWrite, 100);
    h1.borrow_mut()
        .add_register(String::from("#REG"), Rc::new(RefCell::new(r)));

    let h2 = Host::new_shared(String::from("end"), 4);

    let mut vm = VM::new();

    vm.add_host(h1.clone());
    vm.add_host(h2.clone());

    vm.add_link(800, h1.clone(), h2.clone());
    vm.add_link(-1, h2.clone(), h1.clone());

    let name = "start";
    let host = vm.hosts.get(name).unwrap().clone();
    let name = String::from("x0");

    Exa::spawn(
        &mut vm,
        host,
        name,
        false,
        "copy 301 gp\n mark a\n rand 1 100 gx\n jump a\n",
    )
    .unwrap();

    vm.run_cycle();

    c.bench_function("rand_gx_with_sprite_defined", |b| {
        b.iter(|| black_box(vm.run_cycle()))
    });
}

criterion_group!(
    benches,
    copy_register_loop,
    rand_loop,
    rand_gx_with_sprite_defined
);
criterion_main!(benches);
