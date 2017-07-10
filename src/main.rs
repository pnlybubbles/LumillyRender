// extern crate time;

mod constant;
// mod img;
mod vector;
mod vector2;
mod vector3;

// use std::thread;
// use std::sync::mpsc::{channel, Sender, Receiver};
// use constant::*;
// use img::{Img, Color};
// use vector3::Vector3;
use vector::{Vector, VectorFloat};
use vector2::Vector2;

fn main() {
  let a = Vector2 { x: 1.0, y: 2.0 };
  let b = Vector2 { x: 2.0, y: 5.0 };
  println!("{:?}", a.dot(b));
  println!("{:?}", a.norm());
  // let a = Vector3 { x: 1, y: 2, z: 3 };
  // let b = Vector3 { x: 2, y: 0, z: 1 };
  // println!("{:?}", a.dot(b));
  // let mut output: Img = Default::default();
  // let w = Img::width() as f64;
  // let h = Img::height() as f64;
  // let (tx, rx): (Sender<(usize, usize, Color)>, Receiver<(usize, usize, Color)>) = channel();
  // for _ in 0..SPP {
  //   let tx = tx.clone();
  //   thread::spawn(move || {
  //     Img::each( |x, y| {
  //       let color = [x as f64 / w, y as f64 / h, (x + y) as f64 / (w + h)];
  //       let _ = tx.send((x, y, color));
  //     });
  //   });
  // }
  // for s in 0..SPP {
  //   print!("\rprocessing... ({:.0}/{:.0} : {:.0}%) ", s, SPP, s as f64 / SPP as f64 * 100.0);
  //   Img::each( |_, _| {
  //     let (x, y, pixel) = rx.recv().unwrap();
  //     output.apply(x, y, |output_pixel| {
  //       for (op, p) in output_pixel.iter_mut().zip(pixel.iter()) {
  //         *op += *p;
  //       }
  //     } );
  //   });
  // }
  // let file_name = &format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), SPP);
  // output.save(file_name, |pixel| {
  //   let mut color = [0u8; 3];
  //   for (c, p) in color.iter_mut().zip(pixel.iter()) {
  //     *c = (p / SPP as f64 * 255.0) as u8
  //   }
  //   color
  // });
}
