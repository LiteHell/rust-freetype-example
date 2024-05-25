/// Measured size of string bitmap
#[derive(Clone, Copy)]
pub struct StringBitmapSize {
    pub width: u64,
    pub height: u64,
    pub(crate) y_min: u64,
    pub(crate) y_max: u64,
}

// Rendered string bitmap
pub struct StringBitmap {
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>,
    pub a: Vec<u8>,
    pub size: StringBitmapSize,
}

impl StringBitmap {
    pub(crate) fn new(size: StringBitmapSize) -> StringBitmap {
        StringBitmap {
            r: vec![0; (size.width * size.height).try_into().expect("Too big")],
            g: vec![0; (size.width * size.height).try_into().expect("Too big")],
            b: vec![0; (size.width * size.height).try_into().expect("Too big")],
            a: vec![0; (size.width * size.height).try_into().expect("Too big")],
            size: size.clone(),
        }
    }

    pub(crate) fn set_rgba(&mut self, x: i64, y: i64, rgba: (u8, u8, u8, u8)) {
        let pos = self.get_pos(x, y);

        self.r[pos] = rgba.0;
        self.g[pos] = rgba.1;
        self.b[pos] = rgba.2;
        self.a[pos] = rgba.3;
    }

    pub(crate) fn get_pos(&self, x: i64, y: i64) -> usize {
        ((y) * self.size.width as i64 + (x)) as usize
    }

    pub fn get_rgba(&self, x: i64, y: i64) -> (u8, u8, u8, u8) {
        let pos = self.get_pos(x, y);
        return (self.r[pos], self.g[pos], self.b[pos], self.a[pos]);
    }
}
