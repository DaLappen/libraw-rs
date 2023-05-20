use core::ffi;

use libraw_sys::{
    self, libraw_cameraCount, libraw_cameraList, libraw_capabilities, libraw_version,
    libraw_versionNumber,
};

pub mod error;
mod libraw_data;
pub use libraw_data::*;
mod libraw_processed_image;
pub use libraw_processed_image::*;
mod libraw_decoder;
pub use libraw_decoder::*;

pub fn init() -> error::LibrawResult<LibrawData> {
    LibrawData::new()
}

pub fn version() -> &'static str {
    let version = unsafe { ffi::CStr::from_ptr(libraw_version()) };
    version.to_str().unwrap()
}
pub fn version_number() -> i32 {
    unsafe { libraw_versionNumber() }
}
pub fn capabilities() -> u32 {
    unsafe { libraw_capabilities() }
}
pub fn camera_list() -> Vec<&'static str> {
    let mut cam_list = vec![];
    let list_ptr = unsafe { libraw_cameraList() };
    let mut i = 0;
    loop {
        let c_str_ptr = unsafe { *list_ptr.offset(i) };
        if c_str_ptr.is_null() {
            break;
        }
        let c_str = unsafe { ffi::CStr::from_ptr(c_str_ptr) };
        cam_list.push(c_str);
        i += 1;
    }
    cam_list.iter().map(|s| s.to_str().unwrap()).collect()
}

pub fn camera_count() -> i32 {
    unsafe { libraw_cameraCount() }
}
