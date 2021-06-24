extern crate libretro_backend;

pub mod parse;
pub mod vm;

use libretro_backend::*;

use vm::exa::Exa;
use vm::VM;

struct Emulator<'a> {
    rom_path: Option<String>,
    game_data: Option<GameData>,

    vm: Option<VM<'a>>,
    video_frame: [u8; 120 * 100 * 2],
}

impl Default for Emulator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Emulator<'_> {
    fn new() -> Emulator<'a> {
        Emulator {
            rom_path: None,
            game_data: None,
            vm: None,
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

impl Core for Emulator<'_> {
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

        self.vm = Some(VM::new_redshift());
        let host = self.vm.as_ref().unwrap().hosts.get("core").unwrap().clone();

        Exa::spawn(
            &mut self.vm.as_mut().unwrap(),
            host,
            "x0".to_string(),
            true,
            "copy 301 gp\n wait\n ",
        )
        .expect("cannot spawn");

        // TODO load and verify rom

        let av_info = AudioVideoInfo::new()
            .video(120, 100, 30.0, PixelFormat::RGB565)
            .audio(44100.0)
            .region(Region::NTSC);
        LoadGameResult::Success(av_info)
    }

    fn on_unload_game(&mut self) -> GameData {
        self.rom_path = None;
        self.vm = None;
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        let vm = self.vm.as_mut().unwrap();

        vm.run_cycle();

        Emulator::update_video_frame(&mut self.video_frame, vm.render());
        handle.upload_video_frame(&self.video_frame);

        let audio = [0; (44100 / 30) * 2];
        handle.upload_audio_frame(&audio);
    }

    fn on_reset(&mut self) {}
}

libretro_core!(Emulator);
