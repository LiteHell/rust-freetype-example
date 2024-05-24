mod face;
use std::sync::OnceLock;

use freetype::freetype::{FT_Init_FreeType, FT_Library};

/// Wrapper of FT_Library to bypass rust compiler errors
#[derive(Clone, Copy)]
struct FreeTypeLibraryPointerWrapper {
    pub(crate) ptr: FT_Library,
}

unsafe impl Send for FreeTypeLibraryPointerWrapper {}

unsafe impl Sync for FreeTypeLibraryPointerWrapper {}

/// Initializes FreeType library
///
/// FreeType library is initialized only one time even when
/// `init_freetype` method function is called multiple times
fn init_freetype() -> &'static Result<FreeTypeLibraryPointerWrapper, i32> {
    static RAW_LIBRARY_PTR_INIT: OnceLock<Result<FreeTypeLibraryPointerWrapper, i32>> =
        OnceLock::new();
    unsafe {
        RAW_LIBRARY_PTR_INIT.get_or_init(|| {
            let mut raw_library_ptr = std::ptr::null_mut();
            let err = FT_Init_FreeType(&mut raw_library_ptr);

            if err != 0 {
                Err(err)
            } else {
                Ok(FreeTypeLibraryPointerWrapper {
                    ptr: raw_library_ptr,
                })
            }
        })
    }
}
