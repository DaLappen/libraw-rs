use libraw_sys::{
    self,
    data_structs::{libraw_data_t, libraw_decoder_info_t},
    libraw_close, libraw_dcraw_make_mem_image, libraw_dcraw_ppm_tiff_writer, libraw_dcraw_process,
    libraw_dcraw_thumb_writer, libraw_get_decoder_info, libraw_init, libraw_open_file,
    libraw_raw2image, libraw_recycle, libraw_subtract_black, libraw_unpack, libraw_unpack_thumb,
    LIBRAW_OPIONS_NO_DATAERR_CALLBACK,
};
use std::marker::PhantomData;
use std::{
    ffi::{c_int, CString},
    ptr::NonNull,
    slice,
};

use crate::{LibrawImageFormats, LibrawProcessedImage};

use super::{
    error::{LibrawError, LibrawResult},
    LibrawDecoder,
};

pub struct Processed;
pub struct Unpacked;
pub struct Loaded;
pub struct Unloaded;
pub struct Unknown;
pub struct LibrawData<State = Unloaded, ImgState = Unknown, ThumbState = Unknown> {
    data_ptr: *mut libraw_data_t,
    state: PhantomData<State>,
    img_state: PhantomData<ImgState>,
    thumb_state: PhantomData<ThumbState>,
}

impl<A, B, C> Default for LibrawData<A, B, C> {
    fn default() -> Self {
        LibrawData {
            data_ptr: 0 as *mut libraw_data_t,
            state: PhantomData::<A>,
            img_state: PhantomData::<B>,
            thumb_state: PhantomData::<C>,
        }
    }
}

impl<A, B, C> Drop for LibrawData<A, B, C> {
    fn drop(&mut self) {
        unsafe {
            libraw_close(self.data_ptr);
        }
    }
}

impl LibrawData {
    pub fn new() -> LibrawResult<Self> {
        let ptr = unsafe { NonNull::new(libraw_init(LIBRAW_OPIONS_NO_DATAERR_CALLBACK)) };

        match ptr {
            Some(ptr) => Ok(LibrawData {
                data_ptr: ptr.as_ptr(),
                ..Default::default()
            }),

            None => Err(LibrawError::from_str("Failed to init LibrawData struct!").at("new()")),
        }
    }
}

// Any state
impl<A, B, C> LibrawData<A, B, C> {
    pub fn recycle(self) -> LibrawData<Self> {
        unsafe { libraw_recycle(self.data_ptr) };
        LibrawData {
            data_ptr: self.data_ptr,
            ..Default::default()
        }
    }
}
// State = Unloaded, ImgState = Unknown, ThumbState = Unknown
impl LibrawData<Unloaded> {
    pub fn load_image_from_path(self, path: &str) -> LibrawResult<LibrawData<Loaded>> {
        let filename = match CString::new(path.as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                return Err(LibrawError::from_str(
                    format!("Failed to convert {path} to os_string",),
                )
                .at("load_image_from_path()"))
            }
        };

        match LibrawError::handle_libraw_return(unsafe {
            libraw_open_file(self.data_ptr, filename.as_ptr())
        }) {
            Ok(_) => {
                let ret = LibrawData {
                    data_ptr: self.data_ptr,
                    state: PhantomData::<Loaded>,
                    ..Default::default()
                };
                std::mem::forget(self);
                Ok(ret)
            }
            Err(e) => Err(e.at(format!("load_image_from_path({path})"))),
        }
    }
}

// State = Loaded, ImageState = Any, ThumbState = Any
impl<B, C> LibrawData<Loaded, B, C> {
    pub fn unpack(self) -> LibrawResult<LibrawData<Loaded, Unpacked, C>> {
        match LibrawError::handle_libraw_return(unsafe { libraw_unpack(self.data_ptr) }) {
            Ok(_) => {
                let ret = LibrawData {
                    data_ptr: self.data_ptr,
                    ..Default::default()
                };
                std::mem::forget(self);
                Ok(ret)
            }
            Err(e) => Err(e.at("unpack")),
        }
    }
    pub fn raw2image(self) -> LibrawResult<LibrawData<Loaded, Unpacked, C>> {
        match LibrawError::handle_libraw_return(unsafe { libraw_raw2image(self.data_ptr) }) {
            Ok(_) => {
                let ret = LibrawData {
                    data_ptr: self.data_ptr,

                    ..Default::default()
                };
                std::mem::forget(self);
                Ok(ret)
            }
            Err(e) => Err(e.at("raw2image")),
        }
    }
    pub fn unpack_thumb(self) -> LibrawResult<LibrawData<Loaded, B, Unpacked>> {
        match LibrawError::handle_libraw_return(unsafe { libraw_unpack_thumb(self.data_ptr) }) {
            Ok(_) => {
                let ret = LibrawData {
                    data_ptr: self.data_ptr,
                    ..Default::default()
                };
                std::mem::forget(self);
                Ok(ret)
            }
            Err(e) => Err(e.at("unpack_thumb()")),
        }
    }
    pub fn get_decoder_info(&self) -> LibrawResult<LibrawDecoder> {
        let mut decoder = libraw_decoder_info_t {
            decoder_name: [0; 256].as_ptr(),
            decoder_flags: 0,
        };

        match LibrawError::handle_libraw_return(unsafe {
            libraw_get_decoder_info(self.data_ptr, &mut decoder)
        }) {
            Ok(_) => Ok(LibrawDecoder::from_libraw_raw(decoder)),
            Err(e) => Err(e.at("get_decoder_info()")),
        }
    }
}

// State = Loaded, ImageState = Any, ThumbState = UnpackedOrProcessed
impl<ImageState, ThumbState: UnpackedOrProcessed> LibrawData<Loaded, ImageState, ThumbState> {
    pub fn dcraw_thumb_writer(&mut self, path: &str, tiff: bool) -> LibrawResult<()> {
        unsafe {
            (*self.data_ptr).params.output_tiff = if tiff { 1 } else { 0 };
        }
        let filename = match CString::new(path.as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                return Err(LibrawError::from_str(
                    format!("Failed to convert {path} to os_string",),
                )
                .at("write_to_tiff()"))
            }
        };
        let ret = unsafe { libraw_dcraw_thumb_writer(self.data_ptr, filename.as_ptr()) };

        match LibrawError::handle_libraw_return(ret) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.at(format!("write_to_tiff({path})"))),
        }
    }
    pub fn dcraw_make_mem_thumb(&self) -> LibrawResult<LibrawProcessedImage> {
        let mut errc: c_int = 0;
        let img_ptr =
            NonNull::new(unsafe { libraw_dcraw_make_mem_image(self.data_ptr, &mut errc) });

        match LibrawError::handle_libraw_return(errc) {
            Err(e) => return Err(e.at("dcraw_make_mem_thumb()")),
            _ => {}
        }
        let img_ptr = img_ptr.unwrap();
        let slice = unsafe {
            slice::from_raw_parts(
                img_ptr.as_ref().data.as_ptr(),
                img_ptr.as_ref().data_size as usize,
            )
        };

        Ok(LibrawProcessedImage {
            img_ptr,
            data: slice,
            height: unsafe { &img_ptr.as_ref().height },
            width: unsafe { &img_ptr.as_ref().width },
            colors: unsafe { &img_ptr.as_ref().colors },
            image_type: match unsafe { img_ptr.as_ref().image_type } {
                1 => LibrawImageFormats::JPEG,
                2 => LibrawImageFormats::Bitmap,
                _ => {
                    return Err(LibrawError::from_str(format!(
                    "Unexpected value at ProcessedImage.image_type! Value: {}. Expected 1 or 2 ",
                    unsafe { img_ptr.as_ref().image_type }
                ))
                    .at("make_mem_image()"))
                }
            },
        })
    }
}

// State = Loaded, ImageState = Unpacked, ThumbState = Any
impl<C> LibrawData<Loaded, Unpacked, C> {
    pub fn dcraw_process(self) -> LibrawResult<LibrawData<Loaded, Processed, C>> {
        match LibrawError::handle_libraw_return(unsafe { libraw_dcraw_process(self.data_ptr) }) {
            Ok(_) => {
                let ret = LibrawData {
                    data_ptr: self.data_ptr,
                    ..Default::default()
                };
                std::mem::forget(self);
                Ok(ret)
            }
            Err(e) => Err(e.at("dcraw_process()")),
        }
    }
    pub fn subtract_black(&mut self) {
        unsafe {
            libraw_subtract_black(self.data_ptr);
        }
    }
}

// State = Loaded, ImageState = Processed, ThumbState = Any
impl<C> LibrawData<Loaded, Processed, C> {
    pub fn dcraw_ppm_tiff_writer(
        &mut self,
        path: &str,
        tiff: bool,
    ) -> LibrawResult<LibrawData<Loaded, Processed, C>> {
        unsafe {
            (*self.data_ptr).params.output_tiff = if tiff { 1 } else { 0 };
        }
        let filename = match CString::new(path.as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                return Err(LibrawError::from_str(
                    format!("Failed to convert {path} to os_string",),
                )
                .at("write_to_tiff()"))
            }
        };

        let ret = unsafe { libraw_dcraw_ppm_tiff_writer(self.data_ptr, filename.as_ptr()) };
        match LibrawError::handle_libraw_return(ret) {
            Ok(_) => {
                let ret = LibrawData {
                    data_ptr: self.data_ptr,
                    ..Default::default()
                };
                std::mem::forget(self);
                Ok(ret)
            }

            Err(e) => Err(e.at(format!("write_to_tiff({path})"))),
        }
    }
}

// State = Loaded, ImgState = UnpackedOrProcessed, ThumbState = Any
impl<'a, B: UnpackedOrProcessed, C> LibrawData<Loaded, B, C> {
    pub fn dcraw_make_mem_image<'b>(&mut self) -> LibrawResult<LibrawProcessedImage<'b>> {
        let mut err_code = 0;

        let img_ptr = unsafe {
            NonNull::new(libraw_dcraw_make_mem_image(
                self.data_ptr,
                (&mut err_code) as *mut c_int,
            ))
        };

        match LibrawError::handle_libraw_return(err_code) {
            Ok(_) => {}
            Err(e) => {
                return Err(e.at("create_rgb_bitmap"));
            }
        }
        let img_ptr = img_ptr.expect("Failed to get image bitmap!");

        let slice = unsafe {
            slice::from_raw_parts(
                img_ptr.as_ref().data.as_ptr(),
                img_ptr.as_ref().data_size as usize,
            )
        };

        Ok(LibrawProcessedImage {
            img_ptr,
            data: slice,
            height: unsafe { &img_ptr.as_ref().height },
            width: unsafe { &img_ptr.as_ref().width },
            colors: unsafe { &img_ptr.as_ref().colors },
            image_type: match unsafe { img_ptr.as_ref().image_type } {
                1 => LibrawImageFormats::JPEG,
                2 => LibrawImageFormats::Bitmap,
                _ => {
                    return Err(LibrawError::from_str(format!(
                    "Unexpected value at ProcessedImage.image_type! Value: {}. Expected 1 or 2 ",
                    unsafe { img_ptr.as_ref().image_type }
                ))
                    .at("make_mem_image()"))
                }
            },
        })
    }
}
// Hacky solution to implement a function on different states of LibrawData
pub trait UnpackedOrProcessed {}
impl UnpackedOrProcessed for Unpacked {}
impl UnpackedOrProcessed for Processed {}
