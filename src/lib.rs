mod libretro;

pub mod parse;
pub mod vm;

use libretro::*;

use vm::exa::Exa;
use vm::redshift::RedshiftButton;
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
        let host1 = self.vm.as_ref().unwrap().hosts.get("core").unwrap().clone();
        let _host2 = self.vm.as_ref().unwrap().hosts.get("core").unwrap().clone();

        if true {
            Exa::spawn(
            &mut self.vm.as_mut().unwrap(),
            host1.clone(),
            "x0".to_string(),
            true,
            "MODE\n ; INIT STATE\n DATA 0 0 0 0 0 0 0 0\n DATA 0 0 0 0 0 0 0 0\n \n ; INIT 2 RANDOMS\n RAND 0 15 X\n \n SEEK X\n COPY 1 F\n \n MARK INITLOOP\n SEEK -9999\n RAND 0 15 X\n SEEK X\n TEST F = 0\n FJMP INITLOOP\n SEEK -1\n COPY 1 F\n \n ; RENDER BOARD STATE\n COPY 0 X\n SEEK -9999\n \n MARK RENDER\n COPY F T\n REPL SPRITE\n ADDI X 1 X\n TEST EOF\n FJMP RENDER\n COPY 0 X \n JUMP WAIT\n \n MARK SPRITE\n LINK 801\n COPY T CO\n TEST T = 0\n TJMP BLANKSPRITE\n ADDI 327 CO GP\n MARK BLANKSPRITE\n COPY CO T\n MODI X 4 CO\n MULI 25 CO T\n ADDI 5 T GX\n \n DIVI X 4 CO\n MULI 25 CO T\n ADDI 5 T GY\n \n MARK FOREVER\n WAIT\n JUMP FOREVER\n \n MARK WAIT\n DROP\n VOID M\n REPL KILLER\n @REP 17\n COPY T T\n @END\n GRAB 400\n JUMP RENDER\n \n MARK KILLER\n LINK 801\n @REP 16\n KILL\n @END\n HALT\n",
        )
        .expect("cannot spawn");

            Exa::spawn(
            &mut self.vm.as_mut().unwrap(),
            host1.clone(),
            "x1".to_string(),
            true,
            "MODE\n MARK START\n LINK 800\n \n ; WAIT FOR CLEAR DPAD\n ; BEFORE CONTINUING TO\n ; AVOID REPEAT INPUT\n MARK WAITFORCLEAR\n WAIT\n TEST #PADX = #PADY\n FJMP WAITFORCLEAR\n \n MARK INPUT\n COPY #PADX X\n TEST X = 1\n TJMP RIGHTSTART\n \n TEST X = -1\n TJMP LEFTSTART\n \n COPY #PADY X\n TEST X = 1\n TJMP DOWNSTART\n \n TEST X = -1\n TJMP UPSTART\n \n WAIT\n JUMP INPUT\n \n ; GX CURRENT POS\n ; GY NEXT POS TO CHECK\n ; X CURRENT VAL\n ; CO LOOP VAR\n \n MARK RIGHTSTART\n LINK -1\n GRAB 400\n \n COPY -1 CO\n \n MARK RIGHTLOOP\n ADDI 1 CO CO\n \n ; MAP CO TO GX GY\n MODI CO 4 X\n MULI 4 X X\n ADDI 2 X X\n DIVI CO 4 GX\n SUBI X GX GX\n \n SEEK -9999\n SEEK GX\n COPY F X\n TEST X = 0\n TJMP RIGHTNEXTREP\n COPY GX GY\n \n ; LOOP FOR EACH GY GOING\n ; RIGHT UNTIL WE HIT ONE\n MARK RIGHTGYLOOP\n ADDI GY 1 GY\n \n SEEK -9999\n SEEK GY\n TEST X = F\n FJMP RIGHTNOMATCH\n \n ; MERGE RIGHT\n SEEK -9999\n SEEK GY\n ADDI X 1 F\n SEEK -9999\n SEEK GX\n COPY 0 F\n JUMP RIGHTNEXTREP\n \n MARK RIGHTNOMATCH\n SEEK -9999\n SEEK GY\n TEST F = 0\n FJMP RIGHTNEXTREP\n \n ; MOVE RIGHT\n SEEK -9999\n SEEK GY\n COPY X F\n SEEK -9999\n SEEK GX\n COPY 0 F\n ; INCR GX, KEEP GOING\n ADDI GX 1 GX\n \n MODI GX 4 T\n TEST T = 3\n FJMP RIGHTGYLOOP\n \n MARK RIGHTNEXTREP\n TEST CO < 11\n TJMP RIGHTLOOP\n \n JUMP END\n \n MARK LEFTSTART\n LINK -1\n GRAB 400\n \n COPY -1 CO\n \n MARK LEFTLOOP\n ADDI 1 CO CO\n \n ; MAP CO TO GX GY\n MODI CO 4 X\n MULI 4 X X\n ADDI 1 X X\n DIVI CO 4 GX\n ADDI X GX GX\n \n SEEK -9999\n SEEK GX\n COPY F X\n TEST X = 0\n TJMP LEFTNEXTREP\n COPY GX GY\n \n ; LOOP FOR EACH GY GOING\n ; LEFT UNTIL WE HIT ONE\n MARK LEFTGYLOOP\n ADDI GY -1 GY\n \n SEEK -9999\n SEEK GY\n TEST X = F\n FJMP LEFTNOMATCH\n \n ; MERGE LEFT\n SEEK -9999\n SEEK GY\n ADDI X 1 F\n SEEK -9999\n SEEK GX\n COPY 0 F\n JUMP LEFTNEXTREP\n \n MARK LEFTNOMATCH\n SEEK -9999\n SEEK GY\n TEST F = 0\n FJMP LEFTNEXTREP\n \n ; MOVE LEFT\n SEEK -9999\n SEEK GY\n COPY X F\n SEEK -9999\n SEEK GX\n COPY 0 F\n ; INCR GX, KEEP GOING\n ADDI GX -1 GX\n \n MODI GX 4 T\n TEST T = 0\n FJMP LEFTGYLOOP\n \n MARK LEFTNEXTREP\n TEST CO < 11\n TJMP LEFTLOOP\n \n JUMP END\n \n MARK DOWNSTART\n LINK -1\n GRAB 400\n \n COPY 12 CO\n \n MARK DOWNLOOP\n SUBI CO 1 CO\n COPY CO GX\n SEEK -9999\n SEEK GX\n COPY F X\n TEST X = 0\n TJMP DOWNNEXTREP\n COPY GX GY\n \n ; LOOP FOR EACH GY GOING\n ; DOWN UNTIL WE HIT ONE\n MARK DOWNGYLOOP\n ADDI GY 4 GY\n \n SEEK -9999\n SEEK GY\n TEST X = F\n FJMP DOWNNOMATCH\n \n ; MERGE DOWN\n SEEK -9999\n SEEK GY\n ADDI X 1 F\n SEEK -9999\n SEEK GX\n COPY 0 F\n JUMP DOWNNEXTREP\n \n MARK DOWNNOMATCH\n SEEK -9999\n SEEK GY\n TEST F = 0\n FJMP DOWNNEXTREP\n \n ; MOVE DOWN\n SEEK -9999\n SEEK GY\n COPY X F\n SEEK -9999\n SEEK GX\n COPY 0 F\n ; INCR GX, KEEP GOING\n ADDI GX 4 GX\n \n TEST GY < 12\n TJMP DOWNGYLOOP\n \n MARK DOWNNEXTREP\n TEST CO > 0\n TJMP DOWNLOOP\n JUMP END\n \n MARK UPSTART\n LINK -1\n GRAB 400\n \n COPY 3 CO\n \n MARK UPLOOP\n ADDI 1 CO CO\n COPY CO GX\n SEEK -9999\n SEEK GX\n COPY F X\n TEST X = 0\n TJMP UPNEXTREP\n COPY GX GY\n \n ; LOOP FOR EACH GY GOING\n ; UP UNTIL WE HIT ONE\n MARK UPGYLOOP\n SUBI GY 4 GY\n \n SEEK -9999\n SEEK GY\n TEST X = F\n FJMP UPNOMATCH\n \n ; MERGE UP\n SEEK -9999\n SEEK GY\n ADDI X 1 F\n SEEK -9999\n SEEK GX\n COPY 0 F\n JUMP UPNEXTREP\n \n MARK UPNOMATCH\n SEEK -9999\n SEEK GY\n TEST F = 0\n FJMP UPNEXTREP\n \n ; MOVE UP\n SEEK -9999\n SEEK GY\n COPY X F\n SEEK -9999\n SEEK GX\n COPY 0 F\n ; DECR GX, KEEP GOING\n SUBI GX 4 GX\n \n TEST GY > 3\n TJMP UPGYLOOP\n \n MARK UPNEXTREP\n TEST CO < 15\n TJMP UPLOOP\n \n MARK END\n \n ; CHECK FOR GAME OVER\n SEEK -9999\n MARK GAMEOVERLOOP\n TEST F = 0\n TJMP TRYRANDOM\n TEST EOF\n FJMP GAMEOVERLOOP\n HALT\n \n ; ADD A RANDOM NEW 1\n MARK TRYRANDOM\n RAND 0 15 X\n SEEK -9999\n SEEK X\n TEST F = 0\n FJMP TRYRANDOM\n SEEK -1\n COPY 1 F\n \n DROP\n COPY 0 M\n WAIT\n JUMP START\n",
        )
        .expect("cannot spawn");
        }

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

        vm.reset_inputs();
        vm.unfreeze_waiters();
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
            vm.run_cycle();
            println!("{}", vm);
        }

        if handle.is_joypad_button_pressed(0, JoypadButton::Select) {
            println!("\x1B[2J\x1B[1;1H");
        }

        if handle.is_joypad_button_pressed(0, JoypadButton::X) {
            vm.run_cycle();
            println!("{}", vm);
        }

        // vm.run_for_frame();

        Emulator::update_video_frame(&mut self.video_frame, vm.render());
        handle.upload_video_frame(&self.video_frame);

        let audio = [0; (44100 / 30) * 2];
        handle.upload_audio_frame(&audio);
    }

    fn on_reset(&mut self) {
        self.vm.as_mut().unwrap().run_cycle();
    }
}

libretro_core!(Emulator);
