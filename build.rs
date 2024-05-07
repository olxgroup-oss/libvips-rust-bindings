// (c) Copyright 2019-2024 OLX
fn main() {
    //
    // Get the binaries for Windows from the link below.
    // https://github.com/libvips/build-win64-mxe/releases/
    //
    // Use Windows binaries with the suffix `-ffi.zip` .
    //
    println!("cargo:rustc-link-lib=libvips");
    println!("cargo:rustc-link-lib=libglib-2.0");
    println!("cargo:rustc-link-lib=libgobject-2.0");
}
