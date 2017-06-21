use constant::*;

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
}
