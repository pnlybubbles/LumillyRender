#![feature(box_syntax)]

extern crate time;

mod constant;
mod img;
mod vector;
mod vector2;
mod vector3;
mod ray;
mod sample;
mod camera;

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use constant::*;
use img::{Img, Color};
use vector::{Vector, VectorFloat};
use vector2::Vector2;
use vector3::Vector3;
use camera::Camera;

fn main() {
  let mut output: Img = Default::default();
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let cam = Camera::new(
    // sensor position
    Vector3::new(0.0, 0.0, 5.0),
    // aperture position
    Vector3::new(0.0, 0.0, 4.0),
    // sensor size
    Vector2::new(2.0, 2.0 * (Img::width() as f64 / Img::height() as f64)),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
  );
  let (tx, rx): (Sender<(usize, usize, Color)>, Receiver<(usize, usize, Color)>) = channel();
  for _ in 0..SPP {
    let tx = tx.clone();
    let cam = cam.clone();
    thread::spawn(move || {
      Img::each( |x, y| {
        let ray = cam.sample(x, y);
        let color = ray.value.direction.to_array();
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
      *c = (p / SPP as f64 * 255.0) as u8
    }
    color
  });
}
