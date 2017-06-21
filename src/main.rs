extern crate time;

mod constant;
mod img;

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
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
    });
  }
  let file_name = &format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), SPP);
  output.save(file_name, |pixel| {
    let mut color = [0u8; 3];
    for (c, p) in color.iter_mut().zip(pixel.iter()) {
      *c = (p / SPP as f64 * 256.0) as u8
    }
    color
  });
}
