extern crate image;
extern crate time;

use std::fs::File;
use std::path::Path;
use vector::Vector;
use constant::*;
use util::*;

#[derive(Debug)]
pub struct Img {
}

impl Img {
  pub fn save_png(output: &Box<[[Vector; WIDTH]; HEIGHT]>, samples: usize) {
    println!("\nwriting image...");

    let mut imgbuf = image::ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
      let j = x as usize;
      let i = y as usize;
      *pixel = image::Rgb([to_int(clamp(output[i][WIDTH - j - 1].x / samples as f64)), to_int(clamp(output[i][WIDTH - j - 1].y / samples as f64)), to_int(clamp(output[i][WIDTH - j - 1].z / samples as f64))]);
    }
    let ref mut f = File::create(&Path::new(&format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), samples))).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(f, image::PNG);
  }
}
