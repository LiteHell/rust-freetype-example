use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

use freetype::freetype::{
    FT_Done_Face, FT_Face, FT_Get_Char_Index, FT_Get_Kerning, FT_Load_Glyph, FT_New_Face,
    FT_Render_Glyph, FT_Set_Char_Size, FT_Vector_, FT_LOAD_NO_BITMAP,
};
use freetype::freetype::{FT_Pixel_Mode_, FT_Render_Mode};

use crate::init_freetype;

/// Measured size of string bitmap
#[derive(Clone, Copy)]
pub struct StringBitmapSize {
    pub width: u64,
    pub height: u64,
    y_min: u64,
    y_max: u64,
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

/// Default vertical dpi for new FontFace instances
static mut DEFAULT_VDPI: u32 = 72;

/// Default horizontal dpi for new FontFace instances
static mut DEFAULT_HDPI: u32 = 72;

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
        unsafe {
            let mut face = FontFace {
                raw_ptr: ptr,
                vdpi: DEFAULT_VDPI,
                hdpi: DEFAULT_HDPI,
                font_size: 20.0,
                counter: Arc::new(AtomicU8::new(1)),
                render_mutex: Arc::new(Mutex::new(false)),
            };
            face.call_ft_set_chart_size()
                .expect("Failed to set FontFace to default dpi/size");

            face
        }
    }

    /// Sets default dpi for new FontFace instances
    ///
    /// This doesn't affect existing FontFace instances
    pub fn set_default_dpi(hdpi: u32, vdpi: u32) {
        unsafe {
            DEFAULT_HDPI = hdpi;
            DEFAULT_VDPI = vdpi;
        }
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
                0,
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

    /// Renders glyph corresponding to `char` parameter
    fn render_glpyh(&mut self, char: char) -> Result<(), i32> {
        self.load_glpyh(char)?;
        unsafe {
            let err = FT_Render_Glyph((*self.raw_ptr).glyph, FT_Render_Mode::FT_RENDER_MODE_LCD);

            error_if_not_zero!(err)
        }
    }

    /// Loads glyph corresponding to `char` parameter without bitmap rendering
    ///
    /// Used when calculating font size
    fn load_glpyh(&mut self, char: char) -> Result<(), i32> {
        unsafe {
            let glyph_index = FT_Get_Char_Index(self.raw_ptr, char.into());
            let err = FT_Load_Glyph(
                self.raw_ptr,
                glyph_index,
                FT_LOAD_NO_BITMAP.try_into().unwrap(),
            );

            error_if_not_zero!(err)
        }
    }

    fn get_kerning(&mut self, char: char, previous_char: char) -> Result<(i64, i64), i32> {
        let mut vector = FT_Vector_ { x: 0, y: 0 };

        let err = unsafe {
            let previous_glyph_index = FT_Get_Char_Index(self.raw_ptr, previous_char.into());
            let current_glyph_index = FT_Get_Char_Index(self.raw_ptr, char.into());

            FT_Get_Kerning(
                self.raw_ptr,
                previous_glyph_index,
                current_glyph_index,
                0, /* FT_KERNING_DEFAULT */
                &mut vector,
            )
        };

        println!("kering = {}", (vector).x);
        error_if_not_zero!(err, ((vector).x, (vector).y))
    }

    /// Measure size of rendered string
    fn measure_size_without_lock(&mut self, str: &str) -> Result<StringBitmapSize, i32> {
        let mut ymin = 0;
        let mut ymax = 0;
        let mut pen_x = 0;
        let mut last_char_width = 0;
        let mut last_horizontal_advance = 0;
        let mut previous_char = None;
        for char in str.chars() {
            if let Some(previous_char) = previous_char {
                pen_x += self.get_kerning(char, previous_char).unwrap().0;
            }

            self.load_glpyh(char)?;
            let metrics = unsafe { (*(*self.raw_ptr).glyph).metrics };
            let height = metrics.height;
            let horizontal_bearing_y = metrics.horiBearingY;
            ymin = std::cmp::max(ymin, height - horizontal_bearing_y);
            ymax = std::cmp::max(ymax, horizontal_bearing_y);
            let advance = unsafe { (*(*self.raw_ptr).glyph).advance };
            pen_x += advance.x;

            last_horizontal_advance = advance.x;
            last_char_width = metrics.width;
            previous_char = Some(char);
        }

        let width = pen_x/* - last_horizontal_advance + last_char_width */;
        Ok(StringBitmapSize {
            width: (width as u64) >> 6,
            height: ((ymax + ymin) as u64 >> 6) + 1,
            y_min: ymin as u64 >> 6,
            y_max: ymax as u64 >> 6,
        })
    }

    /// Measure size of rendered string
    pub fn measure_size(&mut self, str: &str) -> Result<StringBitmapSize, i32> {
        // Protect this method as critical section
        let mutex_cloned = self.render_mutex.clone();
        let _guard = mutex_cloned.lock();

        self.call_ft_set_chart_size()?;
        self.measure_size_without_lock(str)
    }

    /// Renders string
    pub fn render_string(&mut self, str: &str) -> Result<StringBitmap, i32> {
        // Protect this method as critical section
        let mutex_cloned = self.render_mutex.clone();
        let _guard = mutex_cloned.lock();

        self.call_ft_set_chart_size()?;
        let size = self.measure_size_without_lock(str)?;

        let mut result = StringBitmap::new(size);
        let mut pen_x: i64 = 0;
        let mut pen_y = 0;

        let mut previous_char = None;

        for char in str.chars() {
            if let Some(previous_char) = previous_char {
                println!("kerning before: {}", pen_x);
                pen_x += self.get_kerning(char, previous_char).unwrap().0 >> 6;
                println!("after: {}", pen_x);
            }

            self.render_glpyh(char)?;
            let bitmap = unsafe { (*(*self.raw_ptr).glyph).bitmap };
            let has_alpha = bitmap.pixel_mode != (FT_Pixel_Mode_::FT_PIXEL_MODE_BGRA) as u8;

            if bitmap.pixel_mode != (FT_Pixel_Mode_::FT_PIXEL_MODE_LCD) as u8 && has_alpha {
                println!("Non-suppported font: No RGB/RGBA Rendering available");
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

            let advance = unsafe { (*(*self.raw_ptr).glyph).advance };
            pen_x += advance.x >> 6;
            pen_y += advance.y >> 6;

            previous_char = Some(char);
        }

        Ok(result)
    }
}

#[test]
pub fn test_rendering() {
    let text = "Hello, World!";
    let mut face = FontFace::from_file("/tmp/sans.ttf", 0).unwrap();
    face.set_dpi(300, 300);

    let size = face.measure_size(text).unwrap();
    println!("size: {} {} {}", size.width, size.y_max, size.y_min);

    let result = face.render_string(text).unwrap();

    let mut imgbuf = image::ImageBuffer::new(size.width as u32, size.height as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let rgba = result.get_rgba(x as i64, y as i64);
        *pixel = image::Rgba([rgba.0, rgba.1, rgba.2, rgba.3]);
    }

    imgbuf.save("test.png").expect("Failed to save image");
}
