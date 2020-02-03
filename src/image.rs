use crate::bindings;
use crate::error::Error;
use crate::ops::*;
use crate::utils;
use crate::Result;

use num_traits::FromPrimitive;
use std::ffi::*;
use std::ptr::null_mut;
use std::mem;

const NULL: *const c_void = null_mut();

#[derive(Debug)]
pub struct VipsImage {
    pub(crate) ctx: *mut bindings::VipsImage,
}

#[derive(Debug)]
pub struct VipsInterpolate {
    pub(crate) ctx: *mut bindings::VipsInterpolate,
}

#[derive(Debug)]
pub(crate) struct VipsBlob {
    pub(crate) ctx: *mut bindings::VipsBlob,
}

/// This is the main type of vips. It represents an image and most operations will take one as input and output a new one.
/// In the moment this type is not thread safe. Be careful working within thread environments.
impl VipsImage {
    pub fn new() -> VipsImage {
        VipsImage {
            ctx: unsafe { bindings::vips_image_new() },
        }
    }

    pub fn new_memory() -> Result<VipsImage> {
        unsafe {
            let res = bindings::vips_image_new_memory();
            vips_image_result(res, Error::InitializationError("Could not generate object"))
        }
    }

    pub fn new_from_file(filename: &str) -> Result<VipsImage> {
        unsafe {
            let f = utils::new_c_string(filename)?;
            let res = bindings::vips_image_new_from_file(f.as_ptr(), NULL);
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn new_from_file_rw(filename: &str) -> Result<VipsImage> {
        unsafe {
            let f = utils::new_c_string(filename)?;
            let res = bindings::vips_image_new_from_file_RW(f.as_ptr());
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn new_from_file_raw(
        filename: &str,
        x_size: i32,
        y_size: i32,
        bands: i32,
        offset: u64,
    ) -> Result<VipsImage> {
        unsafe {
            let f = utils::new_c_string(filename)?;
            let res =
                bindings::vips_image_new_from_file_raw(f.as_ptr(), x_size, y_size, bands, offset);
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn new_from_file_access(filename: &str, access: Access, memory: bool) -> Result<VipsImage> {
        unsafe {
            let access_str = utils::new_c_string("access")?;
            let memory_str = utils::new_c_string("memory")?;
            let f = utils::new_c_string(filename)?;
            let res = bindings::vips_image_new_from_file(
                f.as_ptr(),
                access_str.as_ptr(),
                access as i32,
                memory_str.as_ptr(),
                if memory { 1 } else { 0 },
                NULL,
            );
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn image_new_from_buffer(buffer: &[u8], option_str: &str) -> Result<VipsImage> {
        unsafe {
            let options = utils::new_c_string(option_str)?;
            let res = bindings::vips_image_new_from_buffer(
                buffer.as_ptr() as *const c_void,
                buffer.len() as u64,
                options.as_ptr(),
                NULL,
            );
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn image_new_matrix(width: i32, height: i32) -> Result<VipsImage> {
        unsafe {
            let res = bindings::vips_image_new_matrix(width, height);
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn image_new_matrix_from_array(
        width: i32,
        height: i32,
        array: &[f64],
    ) -> Result<VipsImage> {
        unsafe {
            let res = bindings::vips_image_new_matrix_from_array(
                width,
                height,
                array.as_ptr(),
                array.len() as i32,
            );
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from file"),
            )
        }
    }

    pub fn image_new_from_image(image: &VipsImage, array: &[f64]) -> Result<VipsImage> {
        unsafe {
            let res =
                bindings::vips_image_new_from_image(image.ctx, array.as_ptr(), array.len() as i32);
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from Object"),
            )
        }
    }

    pub fn image_new_from_image1(image: &VipsImage, c: f64) -> Result<VipsImage> {
        unsafe {
            let res = bindings::vips_image_new_from_image1(image.ctx, c);
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from Object"),
            )
        }
    }

    pub fn image_new_temp_file(format: &str) -> Result<VipsImage> {
        unsafe {
            let format_c_str = utils::new_c_string(format)?;
            let res = bindings::vips_image_new_temp_file(format_c_str.as_ptr());
            vips_image_result(
                res,
                Error::InitializationError("Could not initialise VipsImage from format"),
            )
        }
    }

    pub fn image_copy_memory(image: VipsImage) -> Result<VipsImage> {
        unsafe {
            let result = bindings::vips_image_copy_memory(image.ctx);
            vips_image_result(
                result,
                Error::InitializationError("Could not copy memory"),
            )
        }
    }

    pub fn image_wio_input(&mut self) -> Result<()> {
        unsafe {
            let result = bindings::vips_image_wio_input(self.ctx);
            utils::result(result, (), Error::InitializationError("Error on vips)image_wio_input"))
        }
    }

    pub fn get_filename(&self) -> std::result::Result<&str, std::str::Utf8Error> {
        unsafe {
            let filename = bindings::vips_image_get_filename(self.ctx);
            let res = CStr::from_ptr(filename);
            res.to_str()
        }
    }

    pub fn get_width(&self) -> i32 {
        unsafe { bindings::vips_image_get_width(self.ctx) }
    }

    pub fn get_height(&self) -> i32 {
        unsafe { bindings::vips_image_get_height(self.ctx) }
    }

    pub fn get_xoffset(&self) -> i32 {
        unsafe { bindings::vips_image_get_xoffset(self.ctx) }
    }

    pub fn get_yoffset(&self) -> i32 {
        unsafe { bindings::vips_image_get_yoffset(self.ctx) }
    }

    pub fn get_scale(&self) -> f64 {
        unsafe { bindings::vips_image_get_scale(self.ctx) }
    }

    pub fn get_offset(&self) -> f64 {
        unsafe { bindings::vips_image_get_offset(self.ctx) }
    }

    pub fn get_xres(&self) -> f64 {
        unsafe { bindings::vips_image_get_xres(self.ctx) }
    }

    pub fn get_yres(&self) -> f64 {
        unsafe { bindings::vips_image_get_yres(self.ctx) }
    }

    pub fn get_bands(&self) -> i32 {
        unsafe { bindings::vips_image_get_bands(self.ctx) }
    }

    pub fn get_page_height(&self) -> i32 {
        unsafe { bindings::vips_image_get_page_height(self.ctx) }
    }

    pub fn get_n_pages(&self) -> i32 {
        unsafe { bindings::vips_image_get_n_pages(self.ctx) }
    }

    pub fn get_coding(&self) -> Result<Coding> {
        unsafe {
            let res = bindings::vips_image_get_format(self.ctx);
            let format_enum = FromPrimitive::from_i32(res);
            format_enum.ok_or_else(|| Error::IOError("Could get format from image"))
        }
    }

    pub fn get_format(&self) -> Result<BandFormat> {
        unsafe {
            let res = bindings::vips_image_get_format(self.ctx);
            let format_enum = FromPrimitive::from_i32(res);
            format_enum.ok_or_else(|| Error::IOError("Could get format from image"))
        }
    }

    pub fn guess_format(&self) -> Result<BandFormat> {
        unsafe {
            let res = bindings::vips_image_guess_format(self.ctx);
            let format_enum = FromPrimitive::from_i32(res);
            format_enum.ok_or_else(|| Error::IOError("Could get format from image"))
        }
    }

    pub fn get_interpretation(&self) -> Result<Interpretation> {
        unsafe {
            let res = bindings::vips_image_get_interpretation(self.ctx);
            let format_enum = FromPrimitive::from_i32(res);
            format_enum.ok_or_else(|| Error::IOError("Could get format from image"))
        }
    }

    pub fn guess_interpretation(&self) -> Result<Interpretation> {
        unsafe {
            let res = bindings::vips_image_guess_interpretation(self.ctx);
            let format_enum = FromPrimitive::from_i32(res);
            format_enum.ok_or_else(|| Error::IOError("Could get format from image"))
        }
    }

    pub fn image_set_delete_on_close(&mut self, flag: bool) {
        unsafe {
            bindings::vips_image_set_delete_on_close(self.ctx, if flag { 1 } else { 0 });
        }
    }

    pub fn image_invalidate_all(&self) {
        unsafe {
            bindings::vips_image_invalidate_all(self.ctx);
        }
    }

    pub fn image_minimise_all(&self) {
        unsafe {
            bindings::vips_image_minimise_all(self.ctx);
        }
    }

    pub fn image_iskilled(&self) -> bool {
        unsafe { bindings::vips_image_iskilled(self.ctx) == 1 }
    }

    pub fn image_isMSBfirst(&self) -> bool {
        unsafe { bindings::vips_image_isMSBfirst(self.ctx) == 1 }
    }

    pub fn image_isfile(&self) -> bool {
        unsafe { bindings::vips_image_isfile(self.ctx) == 1 }
    }

    pub fn image_ispartial(&self) -> bool {
        unsafe { bindings::vips_image_ispartial(self.ctx) == 1 }
    }

    pub fn image_hasalpha(&self) -> bool {
        unsafe { bindings::vips_image_hasalpha(self.ctx) == 1 }
    }

    pub fn image_set_kill(&self, flag: bool) {
        unsafe {
            bindings::vips_image_set_kill(self.ctx, if flag { 1 } else { 0 });
        }
    }

    pub fn image_set_progress(&self, flag: bool) {
        unsafe {
            bindings::vips_image_set_progress(self.ctx, if flag { 1 } else { 0 });
        }
    }

    pub fn image_write(&self) -> Result<VipsImage> {
        unsafe {
            let out: *mut bindings::VipsImage = null_mut();
            let res = bindings::vips_image_write(self.ctx, out);
            utils::result(
                res,
                VipsImage { ctx: out },
                Error::IOError("Cannot write input to output"),
            )
        }
    }

    pub fn image_pio_input(&mut self) -> Result<()> {
        unsafe {
            let res = bindings::vips_image_pio_input(self.ctx);
            utils::result(res, (), Error::IOError("Cannot read image"))
        }
    }

    pub fn image_pio_output(&mut self) -> Result<()> {
        unsafe {
            let res = bindings::vips_image_pio_output(self.ctx);
            utils::result(res, (), Error::IOError("Cannot write image"))
        }
    }

    pub fn image_inplace(&self) -> Result<()> {
        unsafe {
            let res = bindings::vips_image_inplace(self.ctx);
            utils::result(res, (), Error::IOError("Cannot cannot be modified inplace"))
        }
    }

    pub fn image_write_to_file(&self, filename: &str) -> Result<()> {
        unsafe {
            let file_c_str = utils::new_c_string(filename)?;
            let res = bindings::vips_image_write_to_file(self.ctx, file_c_str.as_ptr(), NULL);
            utils::result(res, (), Error::IOError("Cannot write to file"))
        }
    }

    pub fn image_write_prepare(&self) -> Result<()> {
        unsafe {
            let res = bindings::vips_image_write_prepare(self.ctx);
            utils::result(res, (), Error::IOError("Cannot prepare file to write"))
        }
    }

    pub fn image_write_to_buffer(&self, suffix: &str) -> Result<Vec<u8>> {
        unsafe {
            let mut buffer_buf_size: u64 = 0;
            let mut buffer_out: *mut c_void = null_mut();
            let suffix_c_str = utils::new_c_string(suffix)?;
            let res = bindings::vips_image_write_to_buffer(
                self.ctx,
                suffix_c_str.as_ptr(),
                &mut buffer_out,
                &mut buffer_buf_size,
                NULL,
            );
            utils::result(
                res,
                utils::new_byte_array(buffer_out, buffer_buf_size),
                Error::IOError("Cannot write content to buffer"),
            )
        }
    }

    pub fn image_decode_predict(&self) -> Result<(i32, BandFormat)> {
        unsafe {
            let mut out_bands = 0;
            let mut out_format = 0;
            let res =
                bindings::vips_image_decode_predict(self.ctx, &mut out_bands, &mut out_format);
            let format_enum = FromPrimitive::from_i32(out_format);
            if format_enum.is_some() {
                utils::result(
                    res,
                    (out_bands, format_enum.unwrap()),
                    Error::IOError("Could not predict image format"),
                )
            } else {
                Err(Error::IOError("Could not predict image format"))
            }
        }
    }

    pub fn image_decode(&self) -> Result<VipsImage> {
        unsafe {
            let mut out: *mut bindings::VipsImage = null_mut();
            let res = bindings::vips_image_decode(self.ctx, &mut out);
            utils::result(
                res,
                VipsImage { ctx: out },
                Error::IOError("Cannot decode image"),
            )
        }
    }

    pub fn image_encode(&self, coding: Coding) -> Result<VipsImage> {
        unsafe {
            let mut out: *mut bindings::VipsImage = null_mut();
            let res = bindings::vips_image_encode(self.ctx, &mut out, coding as i32);
            utils::result(
                res,
                VipsImage { ctx: out },
                Error::IOError("Cannot encode image"),
            )
        }
    }
}

unsafe fn vips_image_result(res: *mut bindings::VipsImage, err: Error) -> Result<VipsImage> {
    if res.is_null() {
        Err(err)
    } else {
        Ok(VipsImage { ctx: res })
    }
}

impl VipsInterpolate {
    /// defaults to vips_interpolate_nearest_static
    pub fn new() -> VipsInterpolate {
        unsafe {
            VipsInterpolate {
                ctx: bindings::vips_interpolate_nearest_static(),
            }
        }
    }

    pub fn new_from_neasest_static() -> VipsInterpolate {
        unsafe {
            VipsInterpolate {
                ctx: bindings::vips_interpolate_nearest_static(),
            }
        }
    }

    pub fn new_from_bilinear_static() -> VipsInterpolate {
        unsafe {
            VipsInterpolate {
                ctx: bindings::vips_interpolate_bilinear_static(),
            }
        }
    }

    pub fn new_from_name(name: &str) -> Result<VipsInterpolate> {
        unsafe {
            let nickname = utils::new_c_string(name)?;
            let res = bindings::vips_interpolate_new(nickname.as_ptr());
            if res.is_null() {
                Err(Error::InitializationError(
                    "Cannot initialize interpolator with provided nickname",
                ))
            } else {
                Ok(VipsInterpolate { ctx: res })
            }
        }
    }

    pub fn get_window_size(&self) -> i32 {
        unsafe { bindings::vips_interpolate_get_window_size(self.ctx) }
    }

    pub fn get_windows_offset(&self) -> i32 {
        unsafe { bindings::vips_interpolate_get_window_offset(self.ctx) }
    }
}

impl Clone for VipsImage {
    fn clone(&self) -> Self {
        unsafe {
            let size = mem::size_of_val(&*self.ctx);
            let dest = null_mut::<bindings::VipsImage>();
            std::ptr::copy(self.ctx, dest, size);
            VipsImage {
                ctx: dest
            }
        }
    }
}

impl Clone for VipsBlob {
    fn clone(&self) -> Self {
        unsafe {
            let size = mem::size_of_val(&*self.ctx);
            let dest = null_mut::<bindings::VipsBlob>();
            std::ptr::copy(self.ctx, dest, size);
            VipsBlob {
                ctx: dest
            }
        }
    }
}

impl Clone for VipsInterpolate {
    fn clone(&self) -> Self {
        unsafe {
            let size = mem::size_of_val(&*self.ctx);
            let dest = null_mut::<bindings::VipsInterpolate>();
            std::ptr::copy(self.ctx, dest, size);
            VipsInterpolate {
                ctx: dest
            }
        }
    }
}

impl Drop for VipsImage {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                bindings::g_object_unref(self.ctx as *mut c_void);
            }
        }
    }
}

impl Drop for VipsInterpolate {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                bindings::g_object_unref(self.ctx as *mut c_void);
            }
        }
    }
}

impl Drop for VipsBlob {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                bindings::g_object_unref(self.ctx as *mut c_void);
            }
        }
    }
}

impl Into<Vec<u8>> for VipsBlob {
    fn into(self) -> Vec<u8> {
        unsafe {
            let mut size: u64 = 0;
            let bytes = bindings::vips_blob_get(self.ctx, &mut size);
            Vec::from_raw_parts(bytes as *mut u8, size as usize, size as usize)
        }
    }
}
