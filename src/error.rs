use core::fmt;
use std::{error::Error, ffi::CStr};

use libc::c_int;
use libraw_sys::LIBRAW_SUCCESS;

// /// The error type for libraw functions.
// #[derive(Debug)]
// pub struct LibrawError {
//     repr: Repr,
//     message: String,
// }

// #[derive(Debug)]
// enum Repr {
//     LibRaw(c_int),
//     Os(i32),
// }

// impl fmt::Display for LibrawError {
//     fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         fmt.write_str(&self.message)
//     }
// }

// impl Error for LibrawError {
//     fn description(&self) -> &str {
//         &self.message
//     }
// }

// #[doc(hidden)]
// pub fn from_libraw(error: c_int) -> LibrawError {
//     let message = String::from_utf8_lossy(
//         unsafe { CStr::from_ptr(libraw_sys::libraw_strerror(error)) }.to_bytes(),
//     )
//     .into_owned();

//     LibrawError {
//         repr: Repr::LibRaw(error),
//         message: message,
//     }
// }

// pub fn from_raw_os_error(errno: i32) -> LibrawError {
//     LibrawError {
//         repr: Repr::Os(errno),
//         message: os::error_string(errno),
//     }
// }

// Libraw returns error codes
// e.g. let ret = libraw_open_file(...) -> c_int
// if ret == 0 or. LIBRAW_SUCCESS -> no error
// if ret > 0 -> error in system call -> is errno
// if ret < 0 -> LibRaw error

// LibRaw error types:
// - Non-fatal errors
// - Fatal errors
// --> Check by LIBRAW_FATAL_ERROR(err)

#[derive(Debug)]
pub enum LibrawError {
    SysCallErr(String),
    LibRawErr(String),
    Err(String),
}

impl Error for LibrawError {}
impl fmt::Display for LibrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SysCallErr(err_msg) => {
                write!(f, "SysCall made by LibRaw resulted in error: {:?}", err_msg)
            }
            Self::LibRawErr(err_msg) => {
                write!(f, "Libraw fn call resulted in error: {:?}", err_msg)
            }
            Self::Err(err_msg) => {
                write!(f, "Failed with error: {:?}", err_msg)
            }
        }
    }
}
pub type LibrawResult<T> = Result<T, LibrawError>;
impl LibrawError {
    pub fn handle_libraw_return(err: c_int) -> Result<(), Self> {
        use std::str;
        if err == LIBRAW_SUCCESS {
            Ok(())
        } else if err < 0 {
            Err(Self::LibRawErr(Self::from_libraw(err)))
        } else {
            let err_msg = unsafe {
                str::from_utf8(CStr::from_ptr(libc::strerror(err)).to_bytes())
                    .unwrap()
                    .to_string()
            };
            Err(Self::SysCallErr(err_msg))
        }
    }
    fn from_libraw(error: c_int) -> String {
        let message = String::from_utf8_lossy(
            unsafe { CStr::from_ptr(libraw_sys::libraw_strerror(error)) }.to_bytes(),
        )
        .into_owned();
        message
    }
    pub fn from_str<S: AsRef<str>>(err_msg: S) -> Self {
        Self::Err(format!("{}", err_msg.as_ref()))
    }
    pub fn at<S: AsRef<str>>(self, at: S) -> Self {
        match self {
            Self::Err(e) => Self::Err(e + " at " + at.as_ref()),
            Self::LibRawErr(e) => Self::LibRawErr(e + " at " + at.as_ref()),
            Self::SysCallErr(e) => Self::SysCallErr(e + " at " + at.as_ref()),
        }
    }
}
