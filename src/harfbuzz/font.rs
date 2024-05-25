use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

use harfbuzz_sys::{
    hb_blob_create_from_file, hb_blob_destroy, hb_blob_t, hb_face_create, hb_face_destroy,
    hb_face_get_upem, hb_face_t, hb_font_create, hb_font_destroy, hb_font_t,
};

pub struct Font {
    pub(super) blob_ptr: *mut hb_blob_t,
    pub(super) face_ptr: *mut hb_face_t,
    pub(super) font_ptr: *mut hb_font_t,
    pub(super) ppem: (u16, u16),
    pub(super) upem: u32,

    counter: Arc<AtomicU8>,
    pub(super) lock: Arc<Mutex<bool>>,
}

impl Clone for Font {
    fn clone(&self) -> Self {
        self.counter.fetch_add(1, Ordering::Relaxed);

        Self {
            blob_ptr: self.blob_ptr.clone(),
            face_ptr: self.face_ptr.clone(),
            font_ptr: self.font_ptr.clone(),
            upem: self.upem.clone(),
            counter: self.counter.clone(),
            ppem: self.ppem.clone(),
            lock: self.lock.clone(),
        }
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);

        if self.counter.load(Ordering::Relaxed) == 0 {
            unsafe {
                hb_font_destroy(self.font_ptr);
                hb_face_destroy(self.face_ptr);
                hb_blob_destroy(self.blob_ptr);
            }
        }
    }
}

impl Font {
    pub fn new(path: &str, index: u32) -> Font {
        let (blob_ptr, face_ptr, font_ptr, upem) = unsafe {
            let c_str = CString::new(path).unwrap();
            let blob = hb_blob_create_from_file(c_str.as_ptr());
            let face = hb_face_create(blob, index);
            let font = hb_font_create(face);
            let upem = hb_face_get_upem(face);

            (blob, face, font, upem)
        };

        Font {
            blob_ptr: blob_ptr,
            face_ptr: face_ptr,
            font_ptr: font_ptr,
            upem: upem,
            ppem: (64, 64),
            counter: Arc::new(AtomicU8::new(1)),
            lock: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_ppem(&mut self, x_ppem: u16, y_ppem: u16) {
        self.ppem = (x_ppem, y_ppem);
    }
}
