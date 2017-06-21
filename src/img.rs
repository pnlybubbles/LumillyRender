extern crate image;

use constant::*;
use std::fs::File;
use std::path::Path;

pub type Color = [f64; 3];

pub struct Img {
  data: [[Color; HEIGHT]; WIDTH],
}

impl Default for Img {
  fn default() -> Img {
    Img {
      data: [[Default::default(); HEIGHT]; WIDTH],
    }
  }
}

impl Img {
  pub fn width() -> usize {
    WIDTH
  }

  pub fn height() -> usize {
    HEIGHT
  }

  pub fn get(&self, x: usize, y: usize) -> &Color {
    &self.data[x][y]
  }

  pub fn apply<F>(&mut self, x: usize, y: usize, mut f: F) where F: FnMut(&mut Color) {
    f(&mut self.data[x][y])
  }

  pub fn each<F>(mut f: F) where F: FnMut(usize, usize) {
    for x in 0..WIDTH {
      for y in 0..HEIGHT {
        f(x, y)
      }
    }
  }

  pub fn each_pixel<F>(&self, mut f: F) where F: FnMut(&Color, usize, usize) {
    for (x, col) in self.data.iter().enumerate() {
      for (y, pixel) in col.iter().enumerate() {
        f(&pixel, x, y)
      }
    }
  }

  pub fn each_pixel_mut<F>(&mut self, mut f: F) where F: FnMut(&Color, usize, usize) {
    for (x, col) in self.data.iter().enumerate() {
      for (y, pixel) in col.iter().enumerate() {
        f(&pixel, x, y)
      }
    }
  }

  pub fn save<F>(&self, file_name: &String, mut f: F) where F: FnMut(&Color) -> [u8; 3] {
    let mut buf = image::ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
    for (x, y, pixel) in buf.enumerate_pixels_mut() {
      let output_pixel = self.get(x as usize, y as usize);
      *pixel = image::Rgb(f(output_pixel));
    }
    let ref mut f = File::create(&Path::new(file_name)).unwrap();
    let _ = image::ImageRgb8(buf).save(f, image::PNG);
  }
}
