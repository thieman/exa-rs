#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub pixels: [bool; 100],
}

impl Sprite {
    pub fn empty() -> Sprite {
        Sprite {
            pixels: [false; 100],
        }
    }

    pub fn from_pixels(pixels: [bool; 100]) -> Sprite {
        Sprite { pixels }
    }

    // shorthand is [number of false pixels, number of true pixels, number of false pixels...]
    // until you've covered all 100 of the pixels
    pub fn from_shorthand(shorthand: Vec<u32>) -> Sprite {
        let total: u32 = shorthand.iter().sum();
        if total != 100 {
            panic!("invalid sprite shorthand, must sum to 100");
        }

        let mut pixels = [false; 100];
        let (mut idx, mut value) = (0, false);
        for elem in shorthand.iter() {
            for _ in 0..*elem {
                pixels[idx] = value;
                idx += 1;
            }
            value = !value;
        }
        Sprite { pixels }
    }

    pub fn enable(&mut self, x: u32, y: u32) {
        let idx = (x + (y * 10)) as usize;
        self.pixels[idx] = true;
    }

    pub fn disable(&mut self, x: u32, y: u32) {
        let idx = (x + (y * 10)) as usize;
        self.pixels[idx] = false;
    }

    pub fn toggle(&mut self, x: u32, y: u32) {
        let idx = (x + (y * 10)) as usize;
        self.pixels[idx] = !self.pixels[idx];
    }
}
