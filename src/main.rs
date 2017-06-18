extern crate image;
extern crate time;

mod constant;

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::path::Path;
use constant::*;

type Pixel = [f64; 3];
type Img = [[Pixel; WIDTH]; WIDTH];

fn main() {
  let mut output: Img = [[[0.0; 3]; HEIGHT]; WIDTH];
  let (tx, rx): (Sender<(usize, usize, Pixel)>, Receiver<(usize, usize, Pixel)>) = channel();
  for _ in 0..SPP {
    let tx = tx.clone();
    thread::spawn(move || {
      for x in 0..WIDTH {
        for y in 0..HEIGHT {
          let color = [x as f64 / WIDTH as f64, y as f64 / HEIGHT as f64, (x + y) as f64 / (WIDTH + HEIGHT) as f64];
          let _ = tx.send((x, y, color));
        }
      }
    });
  }
  for s in 0..SPP {
    print!("\rprocessing... ({:.0}/{:.0} : {:.0}%) ", s, SPP, s as f64 / SPP as f64 * 100.0);
    for _ in 0..WIDTH {
      for _ in 0..HEIGHT {
        let (x, y, pixel) = rx.recv().unwrap();
        for i in 0..3 {
          output[x][y][i] += pixel[i];
        }
      }
    }
  }
  println!("{:?}", output[0][0]);
  let mut buf = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
  for (x_, y_, pixel) in buf.enumerate_pixels_mut() {
    let x = x_ as usize;
    let y = y_ as usize;
    let color = [(output[x][y][0] / SPP as f64 * 256.0) as u8, (output[x][y][1] / SPP as f64 * 256.0) as u8, (output[x][y][2] / SPP as f64 * 256.0) as u8];
    *pixel = Rgb(color);
  }
  let file_name = &format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), SPP);
  let ref mut f = File::create(&Path::new(file_name)).unwrap();
  let _ = image::ImageRgb8(buf).save(f, image::PNG);
}
