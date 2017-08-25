#![feature(box_syntax)]
#![allow(dead_code)]

extern crate time;
extern crate threadpool;
extern crate num_cpus;

mod constant;
mod img;
mod vector;
mod vector2;
mod vector3;
mod ray;
mod sample;
mod camera;
mod intersection;
mod material;
mod scene;
mod sphere;
mod objects;
mod sky;
mod description;
mod util;

use std::io::{self, Write};
use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};
use constant::*;
use img::{Img, Color};
use vector3::Vector3;

// 0: default
// 1: normal
// 3: depth
const MODE: usize = 0;

fn main() {
  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());

  let mut output: Img = Default::default();
  let cam = description::camera();
  let scene = description::scene();
  if MODE == 0 {
    println!("spp: {}", SPP);
    let (tx, rx): (Sender<(usize, usize, Color)>, Receiver<(usize, usize, Color)>) = channel();
    let cpu_count = num_cpus::get();
    println!("cpu: {}", cpu_count);
    let pool = ThreadPool::new(cpu_count);
    // モンテカルロ積分
    for _ in 0..SPP {
      let tx = tx.clone();
      let cam = cam.clone();
      let scene = scene.clone();
      pool.execute(move || {
        Img::each( |x, y| {
          // センサーの1画素に入射する放射輝度を立体角測度でモンテカルロ積分し放射照度を得る
          // カメラから出射されるレイをサンプリング
          let (ray, g_term) = cam.sample(x, y);
          // 開口部に入射する放射輝度 (W sr^-1 m^-2)
          let l_into_sensor = scene.radiance(&ray.value, 0);
          // センサーに入射する放射照度
          let e_into_sensor = l_into_sensor * g_term;
          // 今回のサンプリングでの放射照度の推定値
          let delta_e_into_sensor = e_into_sensor * (cam.sensor_sensitivity() / ray.pdf);
          let _ = tx.send((x, y, delta_e_into_sensor.to_array()));
        });
      });
    }
    for s in 0..SPP {
      print!("\rprocessing... ({:.0}/{:.0} : {:.0}%) ", s, SPP, s as f64 / SPP as f64 * 100.0);
      io::stdout().flush().ok();
      Img::each( |_, _| {
        let (x, y, pixel) = rx.recv().unwrap();
        output.apply(x, y, |output_pixel| {
          for (op, p) in output_pixel.iter_mut().zip(pixel.iter()) {
            // 推定値を足し合わせ
            *op += *p;
          }
        });
      });
    }
    let file_name = &format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), SPP);
    output.save(file_name, |pixel| {
      let mut color = [0u8; 3];
      for (c, p) in color.iter_mut().zip(pixel.iter()) {
        // サンプル数で割って期待値を計算
        *c = ((p / SPP as f64).min(1.0).max(0.0).powf(1.0 / 2.2) * 255.0) as u8
      }
      color
    });
  } else if MODE == 1 || MODE == 3 {
    Img::each( |x, y| {
      let (ray, _) = cam.sample(x, y);
      let pixel = if MODE == 1 {
        let normal = scene.normal(&ray.value);
        (normal / 2.0 + Vector3::new(0.5, 0.5, 0.5)).to_array()
      } else if MODE == 3 {
        let depth = scene.depth(&ray.value) / 10.0;
        Vector3::new(depth, depth, depth).to_array()
      } else {
        unreachable!()
      };
      output.apply(x, y, |output_pixel| {
        for (op, p) in output_pixel.iter_mut().zip(pixel.iter()) {
          *op = *p;
        }
      });
    });
    let suffix = if MODE == 1 {
      "normal"
    } else if MODE == 3 {
      "depth"
    } else {
      unreachable!()
    };
    let file_name = &format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), suffix);
    output.save(file_name, |pixel| {
      let mut color = [0u8; 3];
      for (c, p) in color.iter_mut().zip(pixel.iter()) {
        *c = (p.min(1.0).max(0.0) * 255.0) as u8
      }
      color
    });
  }

  let end_time = time::now();
  println!("end: {}", end_time.strftime("%+").unwrap());
  println!("elapse: {}s", (end_time - start_time).num_milliseconds() as f64 / 1000.0);
}
