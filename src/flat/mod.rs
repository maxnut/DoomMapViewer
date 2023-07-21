use tinywad::lumps::palette::{Palette, Palettes};

pub struct Flat {
    buffer: [u8; 4096],
}

impl Flat {
    pub fn from_lump(data: &[u8]) -> Flat {
        Flat {
            buffer: data.clone().try_into().unwrap(),
        }
    }

    pub fn get_image(&self, palette: Palette) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        let (mut r, mut g, mut b, mut a): (u8, u8, u8, u8) = (0, 0, 0, 0);

        for byte in self.buffer.iter() {
            (r, g, b, a) = palette[*byte as usize].into();

            buffer.push(r);
            buffer.push(g);
            buffer.push(b);
            buffer.push(a);
        }

        buffer
    }
}
