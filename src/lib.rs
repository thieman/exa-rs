mod libretro;

pub mod image;
pub mod parse;
pub mod vm;

use libretro::*;

use crate::image::load_image;
use vm::exa::Exa;
use vm::redshift::RedshiftButton;
use vm::VM;

#[allow(dead_code)]
struct Emulator<'a> {
    #[allow(dead_code)]
    rom_path: Option<String>,
    game_data: Option<GameData>,

    pub frame_counter: u64,
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
            frame_counter: 0,
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

        let path = game_data.path().unwrap().to_string();
        self.rom_path = Some(path.clone());
        self.game_data = Some(game_data);

        match load_image(path) {
            Ok(vm) => self.vm = Some(vm),
            Err(_) => {
                self.rom_path = None;
                return LoadGameResult::Failed(self.game_data.take().unwrap());
            }
        }

        let av_info = AudioVideoInfo::new()
            .video(120, 100, 60.0, PixelFormat::RGB565)
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
        self.frame_counter += 1;

        let vm = self.vm.as_mut().unwrap();

        if self.frame_counter % 2 == 0 {
            vm.reset_inputs();
            vm.unfreeze_waiters();
        }

        if handle.is_joypad_button_pressed(0, JoypadButton::Y) {
            vm.input_pressed(RedshiftButton::X);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::B) {
            vm.input_pressed(RedshiftButton::Y);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::A) {
            vm.input_pressed(RedshiftButton::Z);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::Start) {
            vm.input_pressed(RedshiftButton::Start);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::Up) {
            vm.input_pressed(RedshiftButton::Up);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::Down) {
            vm.input_pressed(RedshiftButton::Down);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::Left) {
            vm.input_pressed(RedshiftButton::Left);
        }
        if handle.is_joypad_button_pressed(0, JoypadButton::Right) {
            vm.input_pressed(RedshiftButton::Right);
        }

        if handle.is_joypad_button_pressed(0, JoypadButton::Select) {
            println!("{}", &vm);
        }

        if handle.is_joypad_button_pressed(0, JoypadButton::X) {
            vm.run_cycle();
            println!("{}", &vm);
        }

        vm.run_for_frame();

        Emulator::update_video_frame(&mut self.video_frame, vm.render());
        handle.upload_video_frame(&self.video_frame);

        handle.upload_audio_frame(vm.audio_frame());
    }

    fn on_reset(&mut self) {
        self.vm.as_mut().unwrap().run_cycle();
    }
}

libretro_core!(Emulator);
