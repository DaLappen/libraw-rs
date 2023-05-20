use libraw_sys::data_structs::libraw_decoder_info_t;
use std::ffi;

#[derive(Debug, Clone, Copy)]
pub struct LibrawDecoder {
    pub decoder_name: &'static str,
    pub decoder_flags: u32,
}

impl LibrawDecoder {
    pub fn from_libraw_raw(value: libraw_decoder_info_t) -> Self {
        LibrawDecoder {
            decoder_name: unsafe { ffi::CStr::from_ptr(value.decoder_name).to_str().unwrap() },
            decoder_flags: value.decoder_flags,
        }
    }
}
