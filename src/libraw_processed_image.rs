use std::ptr::NonNull;

use libraw_sys::{data_structs::libraw_processed_image_t, libraw_dcraw_clear_mem};

pub enum LibrawImageFormats {
    JPEG = 1,
    Bitmap = 2,
}
pub struct LibrawProcessedImage<'a> {
    pub(crate) img_ptr: NonNull<libraw_processed_image_t>,
    pub data: &'a [u8],
    pub height: &'a u16,
    pub width: &'a u16,
    pub colors: &'a u16,
    pub image_type: LibrawImageFormats,
}
impl<'a> LibrawProcessedImage<'a> {}

impl Drop for LibrawProcessedImage<'_> {
    fn drop(&mut self) {
        unsafe { libraw_dcraw_clear_mem(self.img_ptr.as_ptr()) }
    }
}
