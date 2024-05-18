use libvips::{ops, VipsApp, VipsImage};
use std::env;
use std::path::Path;

// Load a JPEG file, transform and convert it to PNG
// Use the following cargo command to run this example
// cargo run --examples jpeg_to_png

fn main() {
    // this initializes the libvips library. it has to live as long as the application lives (or as long as you want to use the library within your app)
    // you can't have multiple objects of this type and when it is dropped it will call the libvips functions to free all internal structures.
    let app = VipsApp::new(
        "Test Libvips",
        false,
    )
    .expect("Cannot initialize libvips");

    //set number of threads in libvips's threadpool
    app.concurrency_set(2);
    let cargo_toml_dir = env::current_dir()
        .expect("Where is my Cargo.toml file?")
        .display()
        .to_string();
    let image_path = Path::new("resources/test.png")
        .display()
        .to_string();

    // loads an image from file
    let image = VipsImage::new_from_file(&format!(
        "{}/{}",
        cargo_toml_dir, image_path
    ))
    .unwrap();

    // will resize the image and return a new instance.
    // libvips works most of the time with immutable objects, so it will return a new object
    // the VipsImage struct implements Drop, which will free the memory
    let resized = ops::resize(
        &image,
        0.5,
    )
    .unwrap();

    //optional parameters
    let options = ops::JpegsaveOptions {
        q: 90,
        background: vec![255.0],
        strip: true,
        interlace: true,
        optimize_coding: true,
        optimize_scans: true,
        ..ops::JpegsaveOptions::default()
    };

    // alternatively you can use `jpegsave` that will use the default options
    match ops::jpegsave_with_opts(
        &resized,
        &format!(
            "{}/{}",
            cargo_toml_dir, "png_to_jpeg.jpg"
        ),
        &options,
    ) {
        Err(_) => println!(
            "error: {}",
            app.error_buffer()
                .unwrap()
        ),
        Ok(_) => println!("./png_to_jpeg.jpg was created in the repository root!"),
    }
}
