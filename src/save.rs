extern crate image;

use constant::*;

pub fn save(output: Box<[[Vector; WIDTH]; HEIGHT]>, samples: usize) {
  println!("\nwriting image...");

  let mut imgbuf = image::ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
  for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    let j = x as usize;
    let i = y as usize;
    *pixel = image::Rgb([to_int(output[i][WIDTH - j - 1].x), to_int(output[i][WIDTH - j - 1].y), to_int(output[i][WIDTH - j - 1].z)]);
  }
  let ref mut f = File::create(&Path::new(&format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), samples))).unwrap();
  let _ = image::ImageRgb8(imgbuf).save(f, image::PNG);
}
