extern crate libretro_backend;

pub mod parse;
pub mod vm;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use libretro_backend::*;

use vm::exa::Exa;
use vm::runtime::{VMFrame, VMMessage};
use vm::VM;

struct Emulator {
    rom_path: Option<String>,
    game_data: Option<GameData>,

    message_tx: Option<Sender<VMMessage>>,
    frame_rx: Option<Receiver<VMFrame>>,
    video_frame: [u8; 120 * 100 * 2],
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            rom_path: None,
            game_data: None,
            message_tx: None,
            frame_rx: None,
            video_frame: [0; 120 * 100 * 2],
        }
    }

    fn update_video_frame(video_frame: &mut [u8; 120 * 100 * 2], framebuffer: &[bool; 120 * 100]) {
        for (idx, pixel) in framebuffer.iter().enumerate() {
            video_frame[idx * 2] = if *pixel { 255 } else { 0 };
            video_frame[(idx * 2) + 1] = if *pixel { 255 } else { 0 };
        }
    }
}

impl Core for Emulator {
    fn info() -> CoreInfo {
        CoreInfo::new("exa-rs", env!("CARGO_PKG_VERSION"))
            .supports_roms_with_extension("png")
            .requires_path_when_loading_roms()
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if game_data.is_empty() || game_data.path().is_none() {
            return LoadGameResult::Failed(game_data);
        }

        self.rom_path = Some(game_data.path().unwrap().to_string());
        self.game_data = Some(game_data);

        let (message_tx, message_rx): (Sender<VMMessage>, Receiver<VMMessage>) = mpsc::channel();
        self.message_tx = Some(message_tx);

        let (frame_tx, frame_rx): (Sender<VMFrame>, Receiver<VMFrame>) = mpsc::channel();
        self.frame_rx = Some(frame_rx);

        thread::spawn(move || {
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
                "x1".to_string(),
                true,
                "copy 302 gp\n mark a\n rand 0 100 gx\n jump a\n",
            )
            .expect("cannot spawn");

            // TODO load and verify rom

            vm.run_forever(message_rx, frame_tx)
        });

        let av_info = AudioVideoInfo::new()
            .video(120, 100, 30.0, PixelFormat::RGB565)
            .audio(44100.0)
            .region(Region::NTSC);
        LoadGameResult::Success(av_info)
    }

    fn on_unload_game(&mut self) -> GameData {
        self.rom_path = None;
        self.message_tx = None;
        self.frame_rx = None;
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        self.message_tx
            .as_ref()
            .unwrap()
            .send(VMMessage::SendFrame)
            .expect("vm thread died");

        let frame = self
            .frame_rx
            .as_ref()
            .unwrap()
            .recv()
            .expect("vm thread died");

        Emulator::update_video_frame(&mut self.video_frame, &frame.framebuffer);
        handle.upload_video_frame(&self.video_frame);

        let audio = [0; (44100 / 30) * 2];
        handle.upload_audio_frame(&audio);
    }

    fn on_reset(&mut self) {}
}

libretro_core!(Emulator);
