use std::sync::mpsc::{Receiver, Sender};

use super::VM;

pub enum VMMessage {
    SendFrame,
    Quit,
}

pub struct VMFrame {
    pub framebuffer: [bool; 120 * 100],
}

impl<'a> VM<'a> {
    pub fn run_forever(&mut self, message_rx: Receiver<VMMessage>, frame_tx: Sender<VMFrame>) {
        let mut frames = 0;
        loop {
            match message_rx.try_recv() {
                Ok(VMMessage::SendFrame) => {
                    let frame = VMFrame {
                        framebuffer: *self.render(),
                    };
                    frame_tx.send(frame).unwrap();
                    println!("{}", frames);
                    frames = 0;
                }
                Ok(VMMessage::Quit) => return,
                Err(_) => (),
            }

            self.run_cycles(50);
            frames += 50;
        }
    }
}
