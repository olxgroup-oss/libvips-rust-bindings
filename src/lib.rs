#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(dead_code)]
#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod bindings;
pub mod error;
mod image;
pub mod ops;
mod utils;

use error::Error;
use std::ffi::*;

pub use image::*;

pub type Result<T> = std::result::Result<T, error::Error>;

pub struct VipsApp;

/// That's the main type of this crate. Use it to initialize the system 
impl VipsApp {
    /// default constructor of a VIpsApp instance which will disable memory leak debugging
    pub fn default(name: &str) -> Result<VipsApp> {
        init(name, false)?;
        Ok(VipsApp)
    }

    /// new instance of VipsApp takes the application name and a flag indicating if the library should debug memory leak (good for testing purposes)
    pub fn new(name: &str, detect_leak: bool) -> Result<VipsApp> {
        init(name, detect_leak)?;
        Ok(VipsApp)
    }

    pub fn progress_set(&self, flag: bool) {
        unsafe {
            bindings::vips_progress_set(if flag { 1 } else { 0 });
        }
    }
    
    pub fn get_disc_threshold(&self) -> u64 {
        unsafe { bindings::vips_get_disc_threshold() }
    }
    
    pub fn version_string(&self) -> Result<&str> {
        unsafe {
            let version = CStr::from_ptr(bindings::vips_version_string());
            version
            .to_str()
            .map_err(|_| Error::InitializationError("Error initializing string"))
        }
    }
    
    pub fn thread_shutdown(&self) {
        unsafe {
            bindings::vips_thread_shutdown();
        }
    }

    pub fn error_buffer(&self) -> Result<&str> {
        unsafe {
            let buffer = CStr::from_ptr(bindings::vips_error_buffer());
            buffer
                .to_str()
                .map_err(|_| Error::InitializationError("Error initializing string"))
        }
    }

    pub fn error(&self, domain: &str, error: &str) -> Result<()> {
        unsafe {
            let c_str_error = utils::new_c_string(error)?;
            let c_str_domain = utils::new_c_string(domain)?;
            bindings::vips_error(c_str_domain.as_ptr(), c_str_error.as_ptr());
            Ok(())
        }
    }

    pub fn error_system(&self, code: i32, domain: &str, error: &str) -> Result<()> {
        unsafe {
            let c_str_error = utils::new_c_string(error)?;
            let c_str_domain = utils::new_c_string(domain)?;
            bindings::vips_error_system(code, c_str_domain.as_ptr(), c_str_error.as_ptr());
            Ok(())
        }
    }

    pub fn freeze_error_buffer(&self) {
        unsafe {
            bindings::vips_error_freeze();
        }
    }

    pub fn error_clear(&self) {
        unsafe {
            bindings::vips_error_clear();
        }
    }

    pub fn error_thaw(&self) {
        unsafe {
            bindings::vips_error_thaw();
        }
    }

    pub fn error_exit(&self, error: &str) -> Result<()> {
        unsafe {
            let c_str_error = utils::new_c_string(error)?;
            bindings::vips_error_exit(c_str_error.as_ptr());
            Ok(())
        }
    }
}

impl Drop for VipsApp {
    fn drop(&mut self) {
        unsafe {
            bindings::vips_shutdown();
        }
    }
}

fn init(name: &str, detect_leak: bool) -> Result<i32> {
    let cstring = utils::new_c_string(name);
    if let Ok(c_name) = cstring {
        let res = unsafe { bindings::vips_init(c_name.as_ptr()) };
        let result = if res == 0 {
            Ok(res)
        } else {
            Err(Error::InitializationError("Failed to init libvips"))
        };
        unsafe {
            if detect_leak {
                bindings::vips_leak_set(1);
            };
        }
        result
    } else {
        Err(Error::InitializationError(
            "Failed to convert rust string to C string",
        ))
    }
}
