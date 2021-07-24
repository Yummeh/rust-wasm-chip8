pub struct Chip8Display {
    pixels: u32,
}

impl Chip8Display {
    pub fn new() -> Chip8Display {
        Chip8Display { pixels: 10 }
    }

    pub fn clear(&mut self) {
        self.pixels = 0;
    }
}
