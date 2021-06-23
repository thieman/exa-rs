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

    pub fn from_builtin(code: u32) -> Sprite {
        match code {
            0 => Sprite::empty(),
            1 => Sprite::from_shorthand(vec![
                24, 1, 8, 1, 1, 1, 6, 1, 3, 1, 5, 1, 3, 1, 4, 7, 3, 1, 5, 1, 3, 1, 5, 1, 12,
            ]),
            2 => Sprite::from_shorthand(vec![
                21, 6, 4, 1, 5, 1, 3, 1, 5, 1, 3, 6, 4, 1, 5, 1, 3, 1, 5, 1, 3, 6, 13,
            ]),
            3 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 9, 1, 9, 1, 9, 1, 5, 1, 4, 5, 13,
            ]),
            4 => Sprite::from_shorthand(vec![
                21, 6, 4, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 6, 13,
            ]),
            5 => Sprite::from_shorthand(vec![21, 7, 3, 1, 9, 1, 9, 6, 4, 1, 9, 1, 9, 7, 12]),
            6 => Sprite::from_shorthand(vec![21, 7, 3, 1, 9, 1, 9, 6, 4, 1, 9, 1, 9, 1, 18]),
            7 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 9, 1, 3, 3, 3, 1, 5, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            8 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 7, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 12,
            ]),
            9 => Sprite::from_shorthand(vec![23, 3, 8, 1, 9, 1, 9, 1, 9, 1, 9, 1, 8, 3, 14]),
            10 => Sprite::from_shorthand(vec![27, 1, 9, 1, 9, 1, 9, 1, 9, 1, 3, 1, 5, 1, 4, 5, 13]),
            11 => Sprite::from_shorthand(vec![
                21, 1, 4, 2, 3, 1, 2, 2, 5, 1, 1, 2, 6, 3, 7, 1, 1, 2, 6, 1, 2, 2, 5, 1, 4, 2, 12,
            ]),
            12 => Sprite::from_shorthand(vec![21, 1, 9, 1, 9, 1, 9, 1, 9, 1, 9, 1, 9, 7, 12]),
            13 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 3, 2, 3, 2, 3, 1, 1, 1, 1, 1, 1, 1, 3, 1, 2, 1, 2, 1, 3, 1, 5, 1, 3,
                1, 5, 1, 3, 1, 5, 1, 12,
            ]),
            14 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 3, 2, 4, 1, 3, 1, 1, 1, 3, 1, 3, 1, 2, 1, 2, 1, 3, 1, 3, 1, 1, 1, 3,
                1, 4, 2, 3, 1, 5, 1, 12,
            ]),
            15 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            16 => Sprite::from_shorthand(vec![
                21, 6, 4, 1, 5, 1, 3, 1, 5, 1, 3, 6, 4, 1, 9, 1, 9, 1, 18,
            ]),
            17 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 3, 1, 1, 1, 3, 1, 4, 2, 4, 6, 12,
            ]),
            18 => Sprite::from_shorthand(vec![
                21, 6, 4, 1, 5, 1, 3, 1, 5, 1, 3, 6, 4, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 12,
            ]),
            19 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 10, 5, 10, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            20 => Sprite::from_shorthand(vec![21, 7, 6, 1, 9, 1, 9, 1, 9, 1, 9, 1, 9, 1, 15]),
            21 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            22 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 3, 1, 5, 1, 4, 1, 3, 1, 5, 1, 3, 1, 6, 1, 1, 1, 7, 1, 1, 1, 8, 1, 15,
            ]),
            23 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 3, 1, 5, 1, 3, 1, 5, 1, 4, 1, 1, 1, 1, 1, 5, 1, 1, 1, 1, 1, 6, 1, 1,
                1, 7, 1, 1, 1, 14,
            ]),
            24 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 4, 1, 3, 1, 6, 1, 1, 1, 8, 1, 8, 1, 1, 1, 6, 1, 3, 1, 4, 1, 5, 1, 12,
            ]),
            25 => Sprite::from_shorthand(vec![
                21, 1, 5, 1, 4, 1, 3, 1, 6, 1, 1, 1, 8, 1, 9, 1, 9, 1, 9, 1, 15,
            ]),
            26 => Sprite::from_shorthand(vec![21, 7, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 7, 12]),
            27 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 4, 2, 3, 1, 3, 1, 1, 1, 3, 1, 2, 1, 2, 1, 3, 1, 1, 1, 3, 1, 3, 2, 4,
                1, 4, 5, 13,
            ]),
            28 => Sprite::from_shorthand(vec![23, 2, 7, 1, 1, 1, 9, 1, 9, 1, 9, 1, 9, 1, 9, 1, 15]),
            29 => Sprite::from_shorthand(vec![22, 5, 4, 1, 5, 1, 9, 1, 4, 5, 4, 1, 9, 1, 9, 7, 12]),
            30 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 9, 1, 6, 3, 10, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            31 => Sprite::from_shorthand(vec![
                26, 1, 8, 2, 7, 1, 1, 1, 6, 1, 2, 1, 5, 1, 3, 1, 4, 7, 8, 1, 13,
            ]),
            32 => {
                Sprite::from_shorthand(vec![21, 7, 3, 1, 9, 1, 9, 6, 10, 1, 3, 1, 5, 1, 4, 5, 13])
            }
            33 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 9, 6, 4, 1, 5, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            34 => Sprite::from_shorthand(vec![21, 7, 9, 1, 8, 1, 8, 1, 8, 1, 8, 1, 8, 1, 17]),
            35 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 5, 1, 4, 5, 4, 1, 5, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            36 => Sprite::from_shorthand(vec![
                22, 5, 4, 1, 5, 1, 3, 1, 5, 1, 4, 6, 9, 1, 3, 1, 5, 1, 4, 5, 13,
            ]),
            37 => Sprite::from_shorthand(vec![83, 1, 16]),
            38 => Sprite::from_shorthand(vec![22, 5, 4, 1, 5, 1, 9, 1, 6, 3, 6, 1, 19, 1, 16]),
            39 => Sprite::from_shorthand(vec![23, 1, 9, 1, 9, 1, 9, 1, 9, 1, 19, 1, 16]),
            _ => Sprite::empty(),
        }
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
