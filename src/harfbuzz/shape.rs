use harfbuzz_sys::{
    hb_buffer_get_glyph_infos, hb_buffer_get_glyph_positions, hb_glyph_info_t, hb_glyph_position_t,
    hb_shape,
};

use super::{buffer::Buffer, font::Font};

struct Shaper {
    glyph_count: u32,
    glyph_index: u32,
    glyph_info_ptr: *mut hb_glyph_info_t,
    glyph_position_ptr: *mut hb_glyph_position_t,
    scale: u32,
}

#[derive(Debug)]
pub struct Shape {
    pub glyph_id: u32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub x_advance: i32,
    pub y_advance: i32,
    pub scale: u32,
}

impl Iterator for Shaper {
    type Item = Shape;

    fn next(&mut self) -> Option<Self::Item> {
        if self.glyph_index >= self.glyph_count {
            return None;
        }

        let glyph_id = unsafe { (*self.glyph_info_ptr.add(self.glyph_index as usize)).codepoint };
        let (x_offset, y_offset, x_advance, y_advance) = unsafe {
            let position = *self.glyph_position_ptr.add(self.glyph_index as usize);

            (
                position.x_offset,
                position.y_offset,
                position.x_advance,
                position.y_advance,
            )
        };

        self.glyph_index += 1;
        Some(Shape {
            glyph_id: glyph_id,
            x_offset: x_offset,
            y_offset: y_offset,
            x_advance: x_advance,
            y_advance: y_advance,
            scale: self.scale,
        })
    }
}

pub fn shape(buffer: Buffer, font: &Font) -> Vec<Shape> {
    let _guard = font.lock.lock();
    let (count, info_ptr, pos_ptr) = unsafe {
        hb_shape(font.font_ptr, buffer.raw_ptr, std::ptr::null(), 0);

        let mut glyph_count: u32 = 0;
        let info_ptr = hb_buffer_get_glyph_infos(buffer.raw_ptr, &mut glyph_count);
        let pos_ptr = hb_buffer_get_glyph_positions(buffer.raw_ptr, &mut glyph_count);

        (glyph_count, info_ptr, pos_ptr)
    };

    let shape = Shaper {
        glyph_count: count,
        glyph_index: 0,
        glyph_info_ptr: info_ptr,
        glyph_position_ptr: pos_ptr,
        scale: font.upem,
    };

    shape.collect()
}
