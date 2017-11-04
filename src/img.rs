extern crate image;

use constant::*;
use std::fs::File;
use std::path::Path;
// use std::io::prelude::*;
// use std::io;

pub type Color = [f64; 3];

pub struct Img<T> {
  data: Vec<Vec<T>>,
  height: usize,
  width: usize,
}

impl<T: Copy> Img<T> {
  pub fn new(init: T, width: usize, height: usize) -> Img<T> {
    Img {
      data: vec![vec![init; width]; height],
      height: height,
      width: width,
    }
  }

  pub fn get(&self, x: usize, y: usize) -> T {
    self.data[y][x]
  }

  pub fn set(&mut self, x: usize, y: usize, v: T) {
    self.data[y][x] = v;
  }

  pub fn each_pixel<F>(&self, f: F)
  where
    F: Fn(usize, usize, usize),
  {
    for x in 0..self.width {
      for y in 0..self.height {
        f(x, y, x * self.width + y)
      }
    }
  }

  pub fn save<F>(&self, path: &Path, f: F)
  where
    F: Fn(T) -> [u8; 3],
  {
    let mut buf = image::ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
    for (x, y, pixel) in buf.enumerate_pixels_mut() {
      let output_pixel = self.get(x as usize, y as usize);
      *pixel = image::Rgb(f(output_pixel));
    }
    let ref mut f = File::create(path).unwrap();
    image::ImageRgb8(buf).save(f, image::PNG).unwrap();
  }
}
