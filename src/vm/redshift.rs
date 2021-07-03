use std::cell::RefCell;

use super::audio::{Noise, SquareWave, TriangleWave};
use super::register::Register;
use super::{Host, Permissions, Shared, VM};

#[derive(Debug)]
pub struct RedshiftEnvironment {
    pub game_name: String,
    pub pady: Shared<Register>,
    pub padx: Shared<Register>,
    pub padb: Shared<Register>,
    pub en3d: Shared<Register>,
    pub sqr0: Shared<Register>,
    pub sqr1: Shared<Register>,
    pub tri0: Shared<Register>,
    pub nse0: Shared<Register>,

    pub sqr0_wave: RefCell<SquareWave>,
    pub sqr1_wave: RefCell<SquareWave>,
    pub tri0_wave: RefCell<TriangleWave>,
    pub nse0_wave: RefCell<Noise>,
}

#[derive(Debug)]
pub enum RedshiftButton {
    Up,
    Down,
    Left,
    Right,
    Start,
    X,
    Y,
    Z,
}

impl<'a> VM<'a> {
    // Instantiate a VM matching the Redshift spec
    pub fn new_redshift() -> VM<'a> {
        let mut vm = VM::new();

        let core = Host::new_shared("core".to_string(), 18);
        let input = Host::new_shared("input".to_string(), 24);
        let sound = Host::new_shared("sound".to_string(), 24);
        let aux1 = Host::new_shared("aux1".to_string(), 3);
        let aux2 = Host::new_shared("aux2".to_string(), 3);

        let padx = Register::new_shared(Permissions::ReadOnly, 0);
        let pady = Register::new_shared(Permissions::ReadOnly, 0);
        let padb = Register::new_shared(Permissions::ReadOnly, 0);
        let en3d = Register::new_shared(Permissions::ReadOnly, 0);
        let sqr0 = Register::new_shared(Permissions::ReadWrite, 0);
        let sqr1 = Register::new_shared(Permissions::ReadWrite, 0);
        let tri0 = Register::new_shared(Permissions::ReadWrite, 0);
        let nse0 = Register::new_shared(Permissions::ReadWrite, 0);

        input
            .borrow_mut()
            .add_register(String::from("#PADX"), padx.clone());
        input
            .borrow_mut()
            .add_register(String::from("#PADY"), pady.clone());
        input
            .borrow_mut()
            .add_register(String::from("#PADB"), padb.clone());
        input
            .borrow_mut()
            .add_register(String::from("#EN3D"), en3d.clone());

        sound
            .borrow_mut()
            .add_register(String::from("#SQR0"), sqr0.clone());
        sound
            .borrow_mut()
            .add_register(String::from("#SQR1"), sqr1.clone());
        sound
            .borrow_mut()
            .add_register(String::from("#TRI0"), tri0.clone());
        sound
            .borrow_mut()
            .add_register(String::from("#NSE0"), nse0.clone());

        vm.add_host(core.clone());
        vm.add_host(input.clone());
        vm.add_host(sound.clone());
        vm.add_host(aux1.clone());
        vm.add_host(aux2.clone());

        vm.add_link(800, core.clone(), input.clone());
        vm.add_link(-1, input.clone(), core.clone());
        vm.add_link(801, core.clone(), sound.clone());
        vm.add_link(-1, sound.clone(), core.clone());
        vm.add_link(802, core.clone(), aux1.clone());
        vm.add_link(-1, aux1.clone(), core.clone());
        vm.add_link(803, core.clone(), aux2.clone());
        vm.add_link(-1, aux2.clone(), core.clone());

        vm.redshift = Some(RedshiftEnvironment {
            game_name: "".to_string(),
            padx: padx.clone(),
            pady: pady.clone(),
            padb: padb.clone(),
            en3d: en3d.clone(),
            sqr0: sqr0.clone(),
            sqr1: sqr1.clone(),
            tri0: tri0.clone(),
            nse0: nse0.clone(),

            sqr0_wave: RefCell::new(SquareWave::default()),
            sqr1_wave: RefCell::new(SquareWave::default()),
            tri0_wave: RefCell::new(TriangleWave::default()),
            nse0_wave: RefCell::new(Noise::default()),
        });

        vm
    }

    pub fn reset_inputs(&mut self) {
        let r = &mut self.redshift.as_mut().unwrap();
        r.padx.borrow_mut().value = 0;
        r.pady.borrow_mut().value = 0;
        r.padb.borrow_mut().value = 0;
    }

    pub fn input_pressed(&mut self, for_input: RedshiftButton) {
        let r = &self.redshift.as_ref().unwrap();
        match for_input {
            RedshiftButton::Up => r.pady.borrow_mut().value = -1,
            RedshiftButton::Down => r.pady.borrow_mut().value = 1,
            RedshiftButton::Left => r.padx.borrow_mut().value = -1,
            RedshiftButton::Right => r.padx.borrow_mut().value = 1,
            RedshiftButton::Start => r.padb.borrow_mut().value += 1000,
            RedshiftButton::Z => r.padb.borrow_mut().value += 100,
            RedshiftButton::Y => r.padb.borrow_mut().value += 10,
            RedshiftButton::X => r.padb.borrow_mut().value += 1,
        }
    }
}
