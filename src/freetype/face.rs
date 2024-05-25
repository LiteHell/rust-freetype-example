use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

use freetype::freetype::{
    FT_Done_Face, FT_Face, FT_Load_Glyph, FT_New_Face, FT_Render_Glyph, FT_Set_Char_Size,
    FT_LOAD_NO_BITMAP,
};
use freetype::freetype::{FT_Pixel_Mode_, FT_Render_Mode};

use crate::{
    bitmap::{StringBitmap, StringBitmapSize},
    harfbuzz::shape::Shape,
};

use super::init::init_freetype;

/// Handy macro for producing `Err` while handling integer-type error value
///
/// ## Usage
///  - `error_if_not_zero(err)` : Returns `Err(err)` if `err` is not zero, otherwise returns `Ok()`
///  - `error_if_not_zero(err, ok_value)` : Returns `Err(err)` if `err` is not zero, otherwise returns `Ok(ok_value)`
macro_rules! error_if_not_zero {
    ($error_code: expr) => {{
        if $error_code != 0 {
            Err($error_code)
        } else {
            Ok(())
        }
    }};
    ($error_code: expr, $ok_value: expr) => {{
        if $error_code != 0 {
            Err($error_code)
        } else {
            Ok($ok_value)
        }
    }};
}

/// Font-face
///
/// # Notes
/// - This can be cloned with shared access to one FreeType font-face instance internally.
///   But it also means that concurrent rendering call to font-face cannot be done in parallel.
/// - No support for vertical text layout
pub struct FontFace {
    /// Raw pointer
    raw_ptr: FT_Face,
    /// Vertical dpi
    vdpi: u32,
    /// Horizontal dpi
    hdpi: u32,
    /// Font size in pt
    font_size: f32,

    /// Counter of cloned instances and the original
    counter: Arc<AtomicU8>,

    /// Mutex for protecting render_string method
    /// as critical section
    // bool is meaningless here
    render_mutex: Arc<Mutex<bool>>,
}

impl Drop for FontFace {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
        if self.counter.load(Ordering::Relaxed) == 0 {
            unsafe {
                FT_Done_Face(self.raw_ptr);
            }
        }
    }
}

impl Clone for FontFace {
    fn clone(&self) -> Self {
        self.counter.fetch_add(1, Ordering::Relaxed);

        Self {
            raw_ptr: self.raw_ptr.clone(),
            vdpi: self.vdpi.clone(),
            hdpi: self.hdpi.clone(),
            font_size: self.font_size.clone(),
            counter: self.counter.clone(),
            render_mutex: self.render_mutex.clone(),
        }
    }
}

impl FontFace {
    /// Creates FontFace instance with raw pointer
    /// font-size is 20pt by default.
    fn from_raw_ptr(ptr: FT_Face) -> FontFace {
        let mut face = FontFace {
            raw_ptr: ptr,
            vdpi: 72,
            hdpi: 72,
            font_size: 20.0,
            counter: Arc::new(AtomicU8::new(1)),
            render_mutex: Arc::new(Mutex::new(false)),
        };
        face.call_ft_set_chart_size()
            .expect("Failed to set FontFace to default dpi/size");

        face
    }

    /// Creates FontFace instance from font file
    pub fn from_file(filename: &str, face_index: i64) -> Result<FontFace, i32> {
        let library = {
            let library_init_result = init_freetype();
            if let Ok(ptr_wrapper) = library_init_result {
                ptr_wrapper.ptr
            } else if let Err(err) = library_init_result {
                return Err(*err);
            } else {
                unreachable!();
            }
        };

        unsafe {
            let mut raw_face_ptr = std::ptr::null_mut();
            let filename_c_str =
                CString::new(filename).expect("Failed to create CString from filename");
            let err = FT_New_Face(
                library,
                filename_c_str.as_ptr(),
                face_index,
                &mut raw_face_ptr,
            );

            error_if_not_zero!(err, FontFace::from_raw_ptr(raw_face_ptr))
        }
    }

    /// Sets dpi and font-size of FT_Face
    fn call_ft_set_chart_size(&mut self) -> Result<(), i32> {
        unsafe {
            let err = FT_Set_Char_Size(
                self.raw_ptr,
                (self.font_size * 64.0) as i64,
                (self.font_size * 64.0) as i64,
                self.hdpi,
                self.vdpi,
            );

            error_if_not_zero!(err)
        }
    }

    /// Sets font size in pt unit
    pub fn set_font_size(&mut self, size_in_pt: f32) {
        self.font_size = size_in_pt;
    }

    /// Sets dpi
    ///
    /// - `hdpi` : Horizontal dpi
    /// - `vdpi` : Vertical dpi
    pub fn set_dpi(&mut self, hdpi: u32, vdpi: u32) {
        self.hdpi = hdpi;
        self.vdpi = vdpi;
    }

    fn render_glpyh_with_index(&mut self, glyph_index: u32) -> Result<(), i32> {
        self.load_glpyh_with_index(glyph_index)?;
        unsafe {
            let err = FT_Render_Glyph((*self.raw_ptr).glyph, FT_Render_Mode::FT_RENDER_MODE_LCD);

            error_if_not_zero!(err)
        }
    }

    fn load_glpyh_with_index(&mut self, glyph_index: u32) -> Result<(), i32> {
        unsafe {
            let err = FT_Load_Glyph(
                self.raw_ptr,
                glyph_index,
                FT_LOAD_NO_BITMAP.try_into().unwrap(),
            );

            error_if_not_zero!(err)
        }
    }

    /// Measure size of rendered string
    fn measure_size_without_lock(&mut self, shapes: &[Shape]) -> Result<StringBitmapSize, i32> {
        let mut ymin = 0;
        let mut ymax = 0;
        let mut pen_x = 0;
        for shape in shapes {
            self.load_glpyh_with_index(shape.glyph_id)?;
            let metrics = unsafe { (*(*self.raw_ptr).glyph).metrics };
            let height = metrics.height;
            let horizontal_bearing_y = metrics.horiBearingY;
            ymin = std::cmp::max(ymin, height - horizontal_bearing_y);
            ymax = std::cmp::max(ymax, horizontal_bearing_y);
            let scale =
                unsafe { (shape.scale as f64) / (*(*self.raw_ptr).size).metrics.x_ppem as f64 }
                    * 1.2;
            pen_x += (shape.x_advance as f64 / scale) as i64;
        }

        let width = pen_x/* - last_horizontal_advance + last_char_width */;
        Ok(StringBitmapSize {
            width: (width as u64),
            height: ((ymax + ymin) as u64 >> 6) + 1,
            y_min: ymin as u64 >> 6,
            y_max: ymax as u64 >> 6,
        })
    }

    /// Measure size of rendered string
    pub fn measure_size(&mut self, shapes: &[Shape]) -> Result<StringBitmapSize, i32> {
        // Protect this method as critical section
        let mutex_cloned = self.render_mutex.clone();
        let _guard = mutex_cloned.lock();

        self.call_ft_set_chart_size()?;
        self.measure_size_without_lock(shapes)
    }

    pub fn get_ppem(&mut self) -> Result<(u16, u16), i32> {
        self.call_ft_set_chart_size()?;
        Ok(unsafe {
            (
                (*(*self.raw_ptr).size).metrics.x_ppem,
                (*(*self.raw_ptr).size).metrics.y_ppem,
            )
        })
    }

    /// Renders string
    pub fn render_string(&mut self, shapes: &[Shape]) -> Result<StringBitmap, i32> {
        // Protect this method as critical section
        let mutex_cloned = self.render_mutex.clone();
        let _guard = mutex_cloned.lock();

        self.call_ft_set_chart_size()?;
        let size = self.measure_size_without_lock(shapes)?;

        let mut result = StringBitmap::new(size);
        let mut pen_x: i64 = 0;
        let mut pen_y = 0;

        for shape in shapes {
            self.render_glpyh_with_index(shape.glyph_id)?;
            let bitmap = unsafe { (*(*self.raw_ptr).glyph).bitmap };
            let has_alpha = bitmap.pixel_mode != (FT_Pixel_Mode_::FT_PIXEL_MODE_BGRA) as u8;

            if bitmap.pixel_mode != (FT_Pixel_Mode_::FT_PIXEL_MODE_LCD) as u8 && has_alpha {
                panic!("Non-suppported font: No RGB/RGBA Rendering available");
            }

            let data_count_per_pixel = if has_alpha { 4 } else { 3 };

            for y in 0..bitmap.rows {
                for x in 0..(bitmap.width / data_count_per_pixel) {
                    let buffer_index =
                        (y as i32 * bitmap.pitch + x as i32 * data_count_per_pixel as i32) as usize;
                    let rgba = unsafe {
                        if has_alpha {
                            let a = *bitmap.buffer.add(buffer_index + 3);
                            let div_by_a = |i: u8| ((i as f32) * (255.0 / a as f32)) as u8;

                            (
                                // BGRA
                                div_by_a(*bitmap.buffer.add(buffer_index + 2)),
                                div_by_a(*bitmap.buffer.add(buffer_index + 1)),
                                div_by_a(*bitmap.buffer.add(buffer_index)),
                                a,
                            )
                        } else {
                            (
                                // RGB
                                *bitmap.buffer.add(buffer_index),
                                *bitmap.buffer.add(buffer_index + 1),
                                *bitmap.buffer.add(buffer_index + 2),
                                255,
                            )
                        }
                    };

                    let (bitmap_left, bitmap_top) = unsafe {
                        (
                            (*(*self.raw_ptr).glyph).bitmap_left,
                            (*(*self.raw_ptr).glyph).bitmap_top,
                        )
                    };

                    result.set_rgba(
                        pen_x as i64 + x as i64 + bitmap_left as i64,
                        pen_y as i64
                            + y as i64
                            + (size.height as i64 - (bitmap_top as i64 + size.y_min as i64)) as i64,
                        rgba,
                    );
                }
            }

            let scale =
                unsafe { (shape.scale as f64) / (*(*self.raw_ptr).size).metrics.x_ppem as f64 }
                    * 1.2;
            let (x_advance, y_advance) = {
                (
                    (shape.x_advance as f64 / scale) as i64,
                    (shape.y_advance as f64 / scale) as i64,
                )
            };
            pen_x += x_advance;
            pen_y += y_advance;
        }

        Ok(result)
    }
}
