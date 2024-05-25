use std::ffi::CString;

use harfbuzz_sys::{
    hb_buffer_add_utf8, hb_buffer_create, hb_buffer_destroy, hb_buffer_guess_segment_properties,
    hb_buffer_t,
};

pub struct Buffer {
    pub(super) raw_ptr: *mut hb_buffer_t,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            hb_buffer_destroy(self.raw_ptr);
        }
    }
}

impl Buffer {
    pub fn new(str: &str) -> Buffer {
        let buf = unsafe {
            let buf = hb_buffer_create();
            let str_len = str.len();
            let c_str = CString::new(str).unwrap();
            hb_buffer_add_utf8(buf, c_str.as_ptr(), str_len as i32, 0, str_len as i32);
            hb_buffer_guess_segment_properties(buf);

            buf
        };

        return Buffer { raw_ptr: buf };
    }
}
