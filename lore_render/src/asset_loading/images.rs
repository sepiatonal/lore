use image::{open, DynamicImage, ImageBuffer, Rgba};

pub fn load(path: &str) -> DynamicImage {
    if path.ends_with(".png") {
        load_png(path)
    } else {
        panic!("Filetype not supported");
    }
}

pub fn load_png(path: &str) -> DynamicImage {
    open(path).expect("Error loading image")
}

pub fn white_texture() -> DynamicImage {
    let white = Rgba([255, 255, 255, 255]);
    let img_buf = ImageBuffer::from_pixel(1, 1, white);
    DynamicImage::ImageRgba8(img_buf)
}

pub fn default_texture() -> DynamicImage {
    let black = Rgba([0, 0, 0, 255]);
    let magenta = Rgba([255, 0, 255, 255]);
    let mut img_buf = ImageBuffer::from_pixel(8, 8, black);
    for x in (0..8) {
        for y in (0..7).step_by(2) {
            img_buf.put_pixel(x, y + (if x % 2 == 0 { 0 } else { 1 }), magenta);
        }
    }
    DynamicImage::ImageRgba8(img_buf)
}
