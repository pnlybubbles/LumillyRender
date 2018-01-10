#![feature(box_syntax)]
#![feature(sort_unstable)]
#![feature(test)]
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
extern crate toml;

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
mod scene_loader;

use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};
use img::*;
use math::vector::*;
use std::path::Path;
use std::sync::Arc;
use std::io::{self, Write};
use description::Description;

fn main() {
  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());
  let description = Description::new("scene.toml");
  let width = description.config.film.resolution.0;
  let height = description.config.film.resolution.1;
  let mut output = Img::new(Vector3::zero(), width, height);
  let cam = Arc::new(description.camera());
  let scene = Arc::new(description.scene());
  let spp = description.config.renderer.samples;
  println!("resolution: {}x{}", width, height);
  println!("spp: {}", spp);
  let (tx, rx): (Sender<(usize, usize, Vector3)>, Receiver<(usize, usize, Vector3)>) = channel();
  let num_cpus = num_cpus::get();
  let num_threads_config = description.config.renderer.threads.unwrap_or(0);
  let num_threads = if num_threads_config <= 0 { num_cpus } else { num_threads_config };
  println!("threads: {}", num_threads);
  let pool = ThreadPool::new(num_threads);
  let integrator = description.config.renderer.integrator.unwrap_or("pt-direct".to_string());
  println!("integrator: {}", integrator);
  // モンテカルロ積分
  output.each_pixel( |x, y, _| {
    let tx = tx.clone();
    let cam = cam.clone();
    let scene = scene.clone();
    match integrator.as_str() {
      "pt" => {
        pool.execute(move || {
          let estimated_sum = (0..spp).fold(Vector3::zero(), |sum, _| {
            // センサーの1画素に入射する放射輝度を立体角測度でモンテカルロ積分し放射照度を得る
            // カメラから出射されるレイをサンプリング
            let (ray, g_term) = cam.sample(x, y);
            // 開口部に入射する放射輝度 (W sr^-1 m^-2)
            let l_into_sensor = scene.radiance(&ray.value);
            // センサーに入射する放射照度
            let e_into_sensor = l_into_sensor * g_term;
            // 今回のサンプリングでの放射照度の推定値
            let delta_e_into_sensor = e_into_sensor * (cam.sensor_sensitivity() / ray.pdf);
            sum + delta_e_into_sensor
          });
          tx.send((x, y, estimated_sum / spp as f32)).unwrap()
        });
      },
      "pt-direct" => {
        pool.execute(move || {
          let estimated_sum = (0..spp).fold(Vector3::zero(), |sum, _| {
            // センサーの1画素に入射する放射輝度を立体角測度でモンテカルロ積分し放射照度を得る
            // カメラから出射されるレイをサンプリング
            let (ray, g_term) = cam.sample(x, y);
            // 開口部に入射する放射輝度 (W sr^-1 m^-2)
            let l_into_sensor = scene.radiance_nee(&ray.value);
            // センサーに入射する放射照度
            let e_into_sensor = l_into_sensor * g_term;
            // 今回のサンプリングでの放射照度の推定値
            let delta_e_into_sensor = e_into_sensor * (cam.sensor_sensitivity() / ray.pdf);
            sum + delta_e_into_sensor
          });
          tx.send((x, y, estimated_sum / spp as f32)).unwrap()
        });
      },
      _ => panic!(format!("Unknown integrator type `{}`", integrator)),
    }
  });

  let all = height * width;

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
  let gamma = description.config.film.gamma.unwrap_or(2.2);
  save(&output, &description.config.film.output, gamma, spp);

  let end_time = time::now();
  println!("end: {}", end_time.strftime("%+").unwrap());
  println!(
    "elapse: {}s",
    (end_time - start_time).num_milliseconds() as f32 / 1000.0
  );
}

fn save(output: &Img<Vector3>, format: &str, gamma: f32, spp: usize) {
  let file_path = &format!(
    "images/image_{}_{}.{}",
    time::now().strftime("%Y%m%d%H%M%S").unwrap(),
    spp,
    format,
  );
  match format {
    "hdr" => {
      output.save_hdr(&Path::new(file_path), |pixel| {
        [pixel.x, pixel.y, pixel.z]
      });
    },
    "png" => {
      output.save_png(&Path::new(file_path), |pixel| {
        [to_color(pixel.x, gamma), to_color(pixel.y, gamma), to_color(pixel.z, gamma)]
      });
    },
    _ => {
      panic!(format!("Unsupported output type `{}`", format));
    }
  }
}

fn to_color(x: f32, gamma: f32) -> u8 {
  (x.max(0.0).min(1.0).powf(1.0 / gamma) * 255.0) as u8
}
