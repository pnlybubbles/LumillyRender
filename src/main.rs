#![feature(box_syntax)]
#![feature(sort_unstable)]
#![feature(test)]
#![allow(dead_code)]

extern crate time;
extern crate threadpool;
extern crate num_cpus;

mod constant;
mod img;
mod math;
mod ray;
mod sample;
mod camera;
mod intersection;
mod material;
mod scene;
mod sphere;
mod triangle;
mod objects;
mod sky;
mod description;
mod util;
mod shape;
mod aabb;
mod bvh;

use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};
use constant::*;
use img::*;
use math::vector::*;
use std::path::Path;
use std::sync::Arc;
use std::io::{self, Write};

fn main() {
  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());

  let mut output = Img::new(Vector3::zero(), WIDTH, HEIGHT);
  let cam = Arc::new(description::camera(WIDTH, HEIGHT));
  let scene = Arc::new(description::scene());
  println!("{:?}", cam.info());
  println!("spp: {}", SPP);
  let (tx, rx): (Sender<(usize, usize, Vector3)>, Receiver<(usize, usize, Vector3)>) = channel();
  let cpu_count = num_cpus::get();
  println!("cpu: {}", cpu_count);
  let pool = ThreadPool::new(cpu_count);
  // モンテカルロ積分
  output.each_pixel( |x, y, _| {
    let tx = tx.clone();
    let cam = cam.clone();
    let scene = scene.clone();
    pool.execute(move || {
      let estimated_sum = (0..SPP).fold(Vector3::zero(), |sum, _| {
        // センサーの1画素に入射する放射輝度を立体角測度でモンテカルロ積分し放射照度を得る
        // カメラから出射されるレイをサンプリング
        let (ray, g_term) = cam.sample(x, y);
        // 開口部に入射する放射輝度 (W sr^-1 m^-2)
        let l_into_sensor = scene.radiance_nee(&ray.value, 0, false);
        // センサーに入射する放射照度
        let e_into_sensor = l_into_sensor * g_term;
        // 今回のサンプリングでの放射照度の推定値
        let delta_e_into_sensor = e_into_sensor * (cam.sensor_sensitivity() / ray.pdf);
        sum + delta_e_into_sensor
      });
      tx.send((x, y, estimated_sum / SPP as f32)).unwrap()
    });
  });

  let all = HEIGHT * WIDTH;

  for i in 0..all {
    print!(
      "\rprocessing... ({}/{} : {:.0}%) ",
      i,
      all,
      i as f32 / all as f32 * 100.0
    );
    io::stdout().flush().ok();
    let (x, y, pixel) = rx.recv().unwrap();
    output.set(x, y, pixel);
  }

  println!("");
  println!("saving...");
  save(&output, SPP);

  let end_time = time::now();
  println!("end: {}", end_time.strftime("%+").unwrap());
  println!(
    "elapse: {}s",
    (end_time - start_time).num_milliseconds() as f32 / 1000.0
  );
}

fn save(output: &Img<Vector3>, spp: usize) {
  let file_path = &format!(
    "images/image_{}_{}.hdr",
    time::now().strftime("%Y%m%d%H%M%S").unwrap(),
    spp
  );
  output.save_hdr(&Path::new(file_path), |pixel| {
    [pixel.x, pixel.y, pixel.z]
  });
}

fn to_color(x: f32) -> u8 {
  (x.max(0.0).min(1.0).powf(1.0 / 2.2) * 255.0) as u8
}
