# libvips-rust-bindings
Rust bindings for libvips

This is a safe wrapper for [libvips](https://libvips.github.io/libvips/) C library. It is made on top of the C API and based on the introspection API results.

THIS IS EXPERIMENTAL AND NOT YET TESTED.

This crate itself is not documented, but it has no logic or special behavior in comparison to libvips itself. All calls and types described in the official libvips [docs](https://libvips.github.io/libvips/API/current/) are just translated to rust types. All defaults also respected.

## How the crate was written

As a first step, it runs the bindgen to generate unsafe calls to the C libvips library. After this is generated, a C code is compiled and executed. This code introspects the operations and outputs them as text. This text is parsed and then generates the `error.rs` and the `ops.rs` modules.

Those are basically safe wrappers on top of the also genereated bindings. Though not widely tested, all the memory cleaning should be working as expected. Important to note that all "vips" prefixes in the naming were removed from the operations's names.

The version numbers from this crate will follow the main libvips versions for major and minor numbers. Meaning that the version from this crate correspond to the respective from libvips. The patch numbers will be independent from libvips versions. 

## How to use it

The main entity from this crate is the `VipsApp` struct. It doesn't store any information, but as long as it is not dropped, vips should be working as expected.

Vips needs to be initialized and shut down, this struct does this job, though you don't have to shut it down, it will be done automatically when the variable holding the value of a `VipsApp` struct is droped.

Not all functions were implemented, so if you need some that are not yet there, feel free to open a PR or an issue (it is pretty straight forward to add the ones that needs to be manual).

Many vips operations have optional arguments. The ones that have have been implemented with too variants by this crate. Basically there'll be a regular call with only the required parameters and an additional with the suffix `with_opts` which will take a struct holding the defaults. 

The structs's names for those defaults are named after the operation name in `class case` plus the suffix `Options`. All the struct implements the `Default` trait, so you can construct them like this for example: 

```rust
let options = ops::Composite2Options {
    x: 10,
    y: 10,
    .. Composite2Options::default()
}
```

In the moment the error messages are not being appended to the errors themselves. They're in the libvips error buffer. The error buffer operations are implented inside the `VipsApps` struct. 

Most (if not all) vips operations don't mutate the `VipsImage` object, so they'll return a new object for this. The implementation of `VipsImage` in this crate takes care of freeing the internal pointer after it is dropped. <span style="color:red">Be aware that the VipsImage object is not thread safe in the moment.</span> I'll investigate what is happening and provide a solution for it in the future. 