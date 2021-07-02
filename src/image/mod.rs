use std::boxed::Box;
use std::convert::TryInto;
use std::error::Error;

use fletcher;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::Pixel;
use miniz_oxide::inflate::decompress_to_vec_zlib;

use super::vm::exa::sprite::Sprite;
use super::vm::exa::{Exa, Mode};
use super::vm::VM;

struct ImageData {
    pos: usize,
    data: Vec<u8>,
}

impl ImageData {
    pub fn new(data: Vec<u8>) -> ImageData {
        ImageData { pos: 0, data }
    }

    pub fn read_bool(&mut self) -> bool {
        let value = self.data[self.pos] == 1;
        self.pos += 1;
        value
    }

    pub fn read_byte(&mut self) -> u8 {
        let value = self.data[self.pos];
        self.pos += 1;
        value
    }

    pub fn read_int(&mut self) -> i32 {
        let value = unsafe {
            std::mem::transmute::<[u8; 4], i32>(
                self.data[self.pos..self.pos + 4]
                    .try_into()
                    .expect("failed to transmute to i32"),
            )
        };
        self.pos += 4;
        value
    }

    pub fn read_string(&mut self) -> String {
        let length = self.read_int() as usize;
        let value = String::from_utf8_lossy(&self.data[self.pos..self.pos + length]);
        self.pos += length;
        value.to_string()
    }
}

fn png_to_image_data(path: &str) -> Result<ImageData, Box<dyn Error>> {
    let img = ImageReader::open(path)?.decode()?;

    let mut stream: Vec<u8> = vec![];

    let (mut pos, mut this) = (0, 0);
    for (_, _, pixel) in img.pixels() {
        for subpixel in pixel.to_rgb().channels() {
            if pos == 8 {
                stream.push(this);
                this = 0;
                pos = 0;
            }
            if subpixel & 0b00000001 == 0b00000001 {
                this |= 1 << pos;
            }
            pos += 1;
        }
    }

    let length = unsafe { std::mem::transmute::<[u8; 4], u32>(stream[0..4].try_into()?) };

    let checksum_expected =
        unsafe { std::mem::transmute::<[u8; 4], u32>(stream[4..8].try_into()?) };

    let compressed = &stream[8..8 + length as usize];

    let mut checksum_got = fletcher::Fletcher16::new();
    checksum_got.update(compressed);
    assert_eq!(checksum_expected, checksum_got.value() as u32);

    let data = decompress_to_vec_zlib(compressed).expect("failed to decompress image data");

    Ok(ImageData::new(data))
}

/// Load a Redshift image from the specified file and return
/// an initialized Redshift VM implementing the program.
pub fn load_image<'a>(path: &str) -> Result<VM, Box<dyn Error>> {
    let mut image_data = png_to_image_data(path)?;

    let mut vm = VM::new_redshift();

    let _unknown_1 = image_data.read_int();
    let _level_id = image_data.read_string();

    let game_name = image_data.read_string();
    vm.redshift.as_mut().unwrap().game_name = game_name;

    let _unknown_2 = image_data.read_int();

    let _solution_length = image_data.read_int();
    let _unknown_3 = image_data.read_int();

    let exa_count = image_data.read_int();

    let start_host = vm.hosts.get("core").unwrap().clone();
    for _ in 0..exa_count {
        let _unknown_4 = image_data.read_byte();
        let exa_name = image_data.read_string();
        let mut exa_script = image_data.read_string();
        // hack to help out our parser
        if !exa_script.ends_with("\n") {
            exa_script.push_str("\n");
        }

        let _view_mode = image_data.read_byte();
        let message_bus_mode = image_data.read_byte();

        let mut raw_sprite = [false; 100];
        for idx in 0..100 {
            raw_sprite[idx] = image_data.read_bool();
        }
        let sprite = Sprite::from_pixels(raw_sprite);

        let exa = Exa::spawn(
            &mut vm,
            start_host.clone(),
            exa_name,
            true,
            exa_script.as_str(),
        )
        .expect("failed to initialize exa");
        exa.borrow_mut().sprite = sprite;
        if message_bus_mode == 1 {
            exa.borrow_mut().mode = Mode::Local;
        }
    }

    Ok(vm)
}
