// (c) Copyright 2019-2023 OLX
// this is manually created because it doesn't follow the standard from the introspection output

/// VipsLinear (linear), calculate (a * in + b)
/// inp: `&VipsImage` -> Input image
/// a: `&[f64]` -> Multiply by this. Must have equal len as b
/// b: `&[f64]` -> Add this. Must have equal len as a
/// returns `VipsImage` - Output image
pub fn linear(inp: &VipsImage, a: &[f64], b: &[f64]) -> Result<VipsImage> {
    unsafe {
        if a.len() != b.len() {
            return Err(Error::LinearError);
        }
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let a_in: *const f64 = a.as_ptr();
        let b_in: *const f64 = b.as_ptr();
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let vips_op_response =
            bindings::vips_linear(inp_in, &mut out_out, a_in, b_in, b.len() as i32, NULL);
        utils::result(
            vips_op_response,
            VipsImage { ctx: out_out },
            Error::LinearError,
        )
    }
}

/// Options for linear operation
pub struct LinearOptions {
    /// uchar: `bool` -> Output should be uchar
    /// default: false
    pub uchar: bool,
}

impl std::default::Default for LinearOptions {
    fn default() -> Self {
        LinearOptions { uchar: false }
    }
}

/// VipsLinear (linear), calculate (a * in + b)
/// inp: `&VipsImage` -> Input image
/// a: `&[f64]` -> Multiply by this. Must have equal len as b
/// b: `&[f64]` -> Add this. Must have equal len as a
/// linear_options: `&LinearOptions` -> optional arguments
/// returns `VipsImage` - Output image
pub fn linear_with_opts(
    inp: &VipsImage,
    a: &[f64],
    b: &[f64],
    linear_options: &LinearOptions,
) -> Result<VipsImage> {
    unsafe {
        if a.len() != b.len() {
            return Err(Error::LinearError);
        }
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let a_in: *const f64 = a.as_ptr();
        let b_in: *const f64 = b.as_ptr();
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let uchar_in: i32 = if linear_options.uchar { 1 } else { 0 };
        let uchar_in_name = utils::new_c_string("uchar")?;

        let vips_op_response = bindings::vips_linear(
            inp_in,
            &mut out_out,
            a_in,
            b_in,
            b.len() as i32,
            uchar_in_name.as_ptr(),
            uchar_in,
            NULL,
        );
        utils::result(
            vips_op_response,
            VipsImage { ctx: out_out },
            Error::LinearError,
        )
    }
}

/// VipsGetpoint (getpoint), read a point from an image
/// inp: `&VipsImage` -> Input image
/// x: `i32` -> Point to read
/// min: 0, max: 10000000, default: 0
/// y: `i32` -> Point to read
/// min: 0, max: 10000000, default: 0
/// returns `Vec<f64>` - Array of output values
pub fn getpoint(inp: &VipsImage, x: i32, y: i32) -> Result<Vec<f64>> {
    unsafe {
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let mut out_array_size: i32 = 0;
        let mut out_array: *mut f64 = null_mut();

        let vips_op_response =
            bindings::vips_getpoint(inp_in, &mut out_array, &mut out_array_size, x, y, NULL);
        utils::result(
            vips_op_response,
            utils::new_double_array(out_array, out_array_size.try_into().unwrap()),
            Error::GetpointError,
        )
    }
}

/// VipsCase (case), use pixel values to pick cases from an array of images
/// index: `&VipsImage` -> Index image
/// cases: `&mut [VipsImage]` -> Array of case images
/// n: `i32` -> number of case images
/// returns `VipsImage` - Output image
pub fn case(index: &VipsImage, cases: &mut [VipsImage], n: i32) -> Result<VipsImage> {
    unsafe {
        let index_in: *mut bindings::VipsImage = index.ctx;
        let cases_in: *mut *mut bindings::VipsImage =
            cases.iter().map(|v| v.ctx).collect::<Vec<_>>().as_mut_ptr();
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let vips_op_response = bindings::vips_case(index_in, cases_in, &mut out_out, n, NULL);
        utils::result(
            vips_op_response,
            VipsImage { ctx: out_out },
            Error::CaseError,
        )
    }
}

pub fn max_with_xy(inp: &VipsImage, size: i32) -> Result<Vec<(f64, i32, i32)>> {
    unsafe {
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let mut out_out: f64 = f64::from(0);

        let size_in: i32 = size;
        let size_in_name = utils::new_c_string("size")?;

        let mut out_array_in: *mut bindings::VipsArrayDouble = std::ptr::null_mut();
        let out_array_in_name = utils::new_c_string("out-array")?;

        let mut x_array_in: *mut bindings::VipsArrayInt = std::ptr::null_mut();
        let x_array_in_name = utils::new_c_string("x-array")?;

        let mut y_array_in: *mut bindings::VipsArrayInt = std::ptr::null_mut();
        let y_array_in_name = utils::new_c_string("y-array")?;

        let vips_op_response = bindings::vips_max(
            inp_in,
            &mut out_out,
            size_in_name.as_ptr(),
            size_in,
            out_array_in_name.as_ptr(),
            &mut out_array_in,
            x_array_in_name.as_ptr(),
            &mut x_array_in,
            y_array_in_name.as_ptr(),
            &mut y_array_in,
            NULL,
        );
        utils::result(vips_op_response, (), Error::MaxError)?;
        let out_array = utils::VipsArrayDoubleWrapper { ctx: out_array_in };
        let x_array = utils::VipsArrayIntWrapper { ctx: x_array_in };
        let y_array = utils::VipsArrayIntWrapper { ctx: y_array_in };
        Ok(std::iter::zip(
            out_array.as_slice(),
            std::iter::zip(x_array.as_slice(), y_array.as_slice()),
        )
        .map(|(v, (x, y))| (*v, *x, *y))
        .collect())
    }
}
