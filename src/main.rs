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

// 0: default
// 1: scene normal
// 2: camera normal
// 3: depth
const MODE: usize = 0;

fn main() {
  let mut output: Img = Default::default();
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let cam = Arc::new(Camera::new(
    // sensor position
    Vector3::new(0.0, 0.5, 5.0),
    // aperture position
    Vector3::new(0.0, 0.4, 4.0),
    // sensor size
    Vector2::new(2.0 * w / h, 2.0),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    1e-5,
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
    Sphere { radius: 1.0, position: Vector3::new(0.0, 0.0, 1.0), material: white_mat.clone() },
    Sphere { radius: 1.0, position: Vector3::new(-2.0, 0.0, 0.0), material: red_mat.clone() },
    Sphere { radius: 1.0, position: Vector3::new(2.0, 0.0, 0.0), material: blue_mat.clone() },
    Sphere { radius: 1e5, position: Vector3::new(0.0, -1e5 - 1.0, 0.0), material: white_mat.clone() },
  ];
  let objects = Objects {
    objects: spheres,
  };
  let scene = Arc::new(Scene {
    depth: 4,
    depth_limit: 64,
    background: Vector3::new(1.0, 1.0, 1.0),
    objects: objects,
  });
  if MODE == 0 {
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
          // センサー面と開口部それぞれのサンプリング点同士の距離
          let d = cam.aperture_sensor_distance / cos_term;
          // ジオメトリ項(m^-2)
          let g_term = cos_term * cos_term / (d * d);
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
  } else if MODE == 1 || MODE == 2 || MODE == 3 {
    Img::each( |x, y| {
      let ray = cam.sample(x, y);
      let pixel = if MODE == 1 || MODE == 2 {
        let scene_normal = scene.normal(&ray.value);
        let normal = if MODE == 1 {
          scene_normal
        } else {
          // カメラ行列 (直交行列)
          let c = [cam.right, cam.up, cam.forward];
          // カメラ行列の逆行列
          let ic = [
            Vector3::new(c[0].x, c[1].x, c[2].x),
            Vector3::new(c[0].y, c[1].y, c[2].y),
            Vector3::new(c[0].z, c[1].z, c[2].z),
          ];
          ic[0] * scene_normal.x + ic[1] * scene_normal.y - ic[2] * scene_normal.z
        };
        (normal / 2.0 + Vector3::new(0.5, 0.5, 0.5)).to_array()
      } else if MODE == 3 {
        let depth = scene.depth(&ray.value) / 10.0;
        Vector3::new(depth, depth, depth).to_array()
      } else {
        [0.0, 0.0, 0.0]
      };
      output.apply(x, y, |output_pixel| {
        for (op, p) in output_pixel.iter_mut().zip(pixel.iter()) {
          // 推定値を足し合わせ
          *op = *p;
        }
      });
    });
    let file_name = &format!("image_{}_normal.png", time::now().strftime("%Y%m%d%H%M%S").unwrap());
    output.save(file_name, |pixel| {
      let mut color = [0u8; 3];
      for (c, p) in color.iter_mut().zip(pixel.iter()) {
        *c = (p.min(1.0).max(0.0) * 255.0) as u8
      }
      color
    });
  }
}
