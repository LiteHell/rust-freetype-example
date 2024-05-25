use crate::{
    bitmap::{StringBitmap, StringBitmapSize},
    freetype,
    harfbuzz::{self, buffer, shape},
};

#[derive(Clone)]
pub struct Font {
    harfbuzz_font: harfbuzz::font::Font,
    freetype_font: freetype::face::FontFace,
}

impl Font {
    pub fn from_file(filename: &str, index: u32) -> Font {
        Font {
            harfbuzz_font: harfbuzz::font::Font::new(filename, index),
            freetype_font: freetype::face::FontFace::from_file(filename, index as i64)
                .expect("Failed to load font with FreeType"),
        }
    }

    pub fn render(&mut self, text: &str) -> Result<StringBitmap, i32> {
        let buffer = buffer::Buffer::new(text);
        let shapes = shape::shape(buffer, &self.harfbuzz_font);
        println!("{:#?}", shapes);

        self.freetype_font.render_string(shapes.as_slice())
    }

    pub fn measure_size(&mut self, text: &str) -> Result<StringBitmapSize, i32> {
        let buffer = buffer::Buffer::new(text);
        let shapes = shape::shape(buffer, &self.harfbuzz_font);

        self.freetype_font.measure_size(shapes.as_slice())
    }
    pub fn set_dpi(&mut self, hdpi: u32, vdpi: u32) {
        self.freetype_font.set_dpi(hdpi, vdpi);
    }
    pub fn set_font_size(&mut self, pt: f32) {
        self.freetype_font.set_font_size(pt);
        let (x_ppem, y_ppem) = self.freetype_font.get_ppem().expect("Failed to get ppem");
        self.harfbuzz_font.set_ppem(x_ppem, y_ppem);
    }
}
