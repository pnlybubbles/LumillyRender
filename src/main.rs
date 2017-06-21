extern crate image;
extern crate time;

mod constant;
mod img;

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::path::Path;
use constant::*;
use img::{Img, Color};

fn main() {
  let mut output: Img = Default::default();
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let (tx, rx): (Sender<(usize, usize, Color)>, Receiver<(usize, usize, Color)>) = channel();
  for _ in 0..SPP {
    let tx = tx.clone();
    thread::spawn(move || {
      Img::each( |x, y| {
        let color = [x as f64 / w, y as f64 / h, (x + y) as f64 / (w + h)];
        let _ = tx.send((x, y, color));
      });
    });
  }
  for s in 0..SPP {
    print!("\rprocessing... ({:.0}/{:.0} : {:.0}%) ", s, SPP, s as f64 / SPP as f64 * 100.0);
    Img::each( |_, _| {
      let (x, y, pixel) = rx.recv().unwrap();
      output.apply(x, y, |output_pixel| {
        for (op, p) in output_pixel.iter_mut().zip(pixel.iter()) {
          *op += *p;
        }
      } );
      if (x == 100 && y == 100) {
        println!("{:?} {:?}", output.get(100, 100), pixel);
      }
    });
  }
  let mut buf = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
  let k = 1.0 / SPP as f64 * 256.0;
  for (x, y, pixel) in buf.enumerate_pixels_mut() {
    let output_pixel = output.get(x as usize, y as usize);
    let color = [(output_pixel[0] * k) as u8, (output_pixel[1] * k) as u8, (output_pixel[2] * k) as u8];
    *pixel = Rgb(color);
  }
  let file_name = &format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), SPP);
  let ref mut f = File::create(&Path::new(file_name)).unwrap();
  let _ = image::ImageRgb8(buf).save(f, image::PNG);
}
