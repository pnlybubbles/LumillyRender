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

use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use constant::*;
use img::{Img, Color};
use vector::{Vector, VectorFloat};
use vector2::Vector2;
use vector3::Vector3;
use camera::Camera;
use material::Material;
use scene::Scene;
use sphere::Sphere;
use objects::Objects;

fn main() {
  let mut output: Img = Default::default();
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let cam_pos = Vector3::new(50.0, 52.0, 295.6);
  let cam_dir = Vector3::new(0.0, -0.042612, -1.0).norm();
  let cam = Arc::new(Camera::new(
    // sensor position
    cam_pos - cam_dir * 40.0,
    // aperture position
    cam_pos,
    // sensor size
    Vector2::new(30.0 * w / h, 30.0),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    10e-5,
  ));
  let red_mat = Material {
    diffuse: Vector3::new(0.75, 0.25, 0.25),
    emission: Vector3::new(0.0, 0.0, 0.0),
  };
  let blue_mat = Material {
    diffuse: Vector3::new(0.25, 0.25, 0.75),
    emission: Vector3::new(0.0, 0.0, 0.0),
  };
  let white_mat = Material {
    diffuse: Vector3::new(0.75, 0.75, 0.75),
    emission: Vector3::new(0.0, 0.0, 0.0),
  };
  let light_mat = Material {
    diffuse: Vector3::new(0.0, 0.0, 0.0),
    emission: Vector3::new(50.0, 50.0, 50.0),
  };
  let spheres = vec![
    Sphere { position: Vector3::new(1e5 + 1.0, 40.8, 81.6), radius: 1e5, material: red_mat.clone() }, // 左
    Sphere { position: Vector3::new(-1e5 + 99.0, 40.8, 81.6), radius: 1e5, material: blue_mat.clone() }, // 右
    Sphere { position: Vector3::new(50.0, 40.8, 1e5), radius: 1e5, material: white_mat.clone() }, // 奥
    Sphere { position: Vector3::new(50.0, 1e5, 81.6), radius: 1e5, material: white_mat.clone() }, // 床
    // Sphere { position: Vector3::new(50.0, -1e5 + 81.6, 81.6), radius: 1e5, material: white_mat.clone() },
    Sphere { position: Vector3::new(27.0, 16.5, 47.0), radius: 16.5, material: white_mat.clone() },
    Sphere { position: Vector3::new(73.0, 16.5, 78.0), radius: 16.5, material: white_mat.clone() },
    Sphere { position: Vector3::new(50.0, 90.0, 81.6), radius: 15.0, material: light_mat.clone() },
  ];
  let objects = Objects {
    objects: spheres,
  };
  let scene = Arc::new(Scene {
    depth: 4,
    depth_limit: 64,
    background: Vector3::new(0.0, 0.0, 0.0),
    objects: objects,
  });
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
        let ray = cam.sample(x, y);
        // 開口部に入射する放射輝度 (W sr^-1 m^-2)
        let l_into_sensor = scene.radiance(&ray.value, 0);
        // cos項
        let cos_term = ray.value.direction.dot(cam.forward);
        // ジオメトリ項(m^-2)
        let g_term = cos_term * cos_term * cam.aperture_sensor_distance / (cam.aperture_sensor_distance);
        // 開口部に入射する放射照度
        let e_into_sensor = l_into_sensor * g_term;
        // 今回のサンプリングでの放射照度の推定値
        let delta_e_into_sensor = e_into_sensor * (cam.sensor_sensitivity / ray.pdf);
        let _ = tx.send((x, y, delta_e_into_sensor.to_array()));
      });
    });
  }
  for s in 0..SPP {
    print!("\rprocessing... ({:.0}/{:.0} : {:.0}%) ", s, SPP, s as f64 / SPP as f64 * 100.0);
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
}
