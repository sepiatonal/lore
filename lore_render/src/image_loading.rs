use image::{open, DynamicImage, ImageBuffer, Rgb};

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
    let white = Rgb([255, 255, 255]);
    let img_buf = ImageBuffer::from_pixel(1, 1, white);
    DynamicImage::ImageRgb8(img_buf)
}

pub fn default_texture() -> DynamicImage {
    let black = Rgb([0, 0, 0]);
    let magenta = Rgb([0, 0, 255]);
    let mut img_buf = ImageBuffer::from_pixel(8, 8, black);
    for x in 0..8 {
        for y in 0..8 {
            if x % 2 == 0 || y % 2 == 0 {
                img_buf.put_pixel(x, y, magenta);
            }
        }
    }
    DynamicImage::ImageRgb8(img_buf)
}
