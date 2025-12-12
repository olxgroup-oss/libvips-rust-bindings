// (c) Copyright 2019-2025 OLX
fn main() {
    if cfg!(target_os = "windows") {
        //
        // Get the binaries for Windows from the link below.
        // https://github.com/libvips/build-win64-mxe/releases/
        //
        // Use Windows binaries with the suffix `-ffi.zip` .
        //
        println!("cargo:rustc-link-lib=libvips");
        println!("cargo:rustc-link-lib=libglib-2.0");
        println!("cargo:rustc-link-lib=libgobject-2.0");
        return;
    }
    println!("cargo:rustc-link-lib=vips");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rustc-link-lib=gobject-2.0");
}
