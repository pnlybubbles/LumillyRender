#![feature(box_syntax)]

extern crate threadpool;
extern crate num_cpus;
extern crate time;
extern crate image;

mod vector;
mod ray;
mod constant;
mod camera;
mod intersection;
mod material;
mod objects;
mod shape;
mod sphere;
mod triangle;
mod scene;
mod util;

use std::io::{BufReader};
use std::fs::File;
use std::path::Path;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use vector::Vector;
use camera::Camera;
use material::Material;
use objects::Objects;
use triangle::Triangle;
use sphere::Sphere;
use scene::{Scene, Background};
use util::*;

const HEIGHT: usize = 800;
const WIDTH: usize = 1200;

const CROP_OFFSET_BOTTOM: usize = 0;
const CROP_OFFSET_LEFT: usize = 0;
const CROP_HEIGHT: usize = 800;
const CROP_WIDTH: usize = 1200;

fn main() {
  let camera_position = Vector{x: -11.5, y: 1.0, z: 13.0};
  let screen_direction = Vector{x: 8.18, y: -2.0, z: -9.0};
  let focus_distance = 3.0 + screen_direction.len();
  let lens_radius = 0.3;
  // let lens_radius = 10e-5;
  let sensor_sensitivity = 1.0;
  let screen_height = 10.0;
  let screen_width = screen_height * (WIDTH as f64 / HEIGHT as f64);
  let cam = Camera::new(camera_position, screen_direction, HEIGHT, WIDTH, screen_height, screen_width, focus_distance, lens_radius, sensor_sensitivity);

  let yellow_material: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.75, 0.75, 0.25)};
  let blue_material: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.25, 0.25, 0.75)};
  let white_material: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.75, 0.75, 0.75)};
  let reflection_material: Material = Material{diffuse: 0.0, reflection: 1.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  let refraction_material: Material = Material{diffuse: 0.0, reflection: 0.0, refraction: 1.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  let emission_material: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector::new(12.0, 12.0, 12.0), color: Vector::new(1.0, 1.0, 1.0)};

  let triangle_objects = vec![
    // Triangle::new(Vector{x: -5.0, y: 5.0, z: 6.0}, Vector{x: -5.0, y: -5.0, z: 6.0}, Vector{x: -5.0, y: 5.0, z: -10.0}, yellow_material),
    // Triangle::new(Vector{x: -5.0, y: -5.0, z: 6.0}, Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: -5.0, y: 5.0, z: -10.0}, yellow_material),
    // Triangle::new(Vector{x: 5.0, y: -5.0, z: 6.0}, Vector{x: 5.0, y: 5.0, z: 6.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, blue_material),
    // Triangle::new(Vector{x: 5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: -5.0, z: 6.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, blue_material),
    // Triangle::new(Vector{x: -5.0, y: 5.0, z: -10.0}, Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, white_material),
    // Triangle::new(Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, white_material),
    // Triangle::new(Vector{x: -5.0, y: -5.0, z: 6.0}, Vector{x: -5.0, y: 5.0, z: 6.0}, Vector{x: 5.0, y: 5.0, z: 6.0}, white_material),
    // Triangle::new(Vector{x: 5.0, y: -5.0, z: 6.0}, Vector{x: -5.0, y: -5.0, z: 6.0}, Vector{x: 5.0, y: 5.0, z: 6.0}, white_material),
    Triangle::new(Vector{x: -8.0, y: -5.0, z: -8.0}, Vector{x: -8.0, y: -5.0, z: 8.0}, Vector{x: 8.0, y: -5.0, z: -8.0}, white_material),
    Triangle::new(Vector{x: -8.0, y: -5.0, z: 8.0}, Vector{x: 8.0, y: -5.0, z: 8.0}, Vector{x: 8.0, y: -5.0, z: -8.0}, white_material),
    // Triangle::new(Vector{x: -5.0, y: 5.0, z: 6.0}, Vector{x: -5.0, y: 5.0, z: -10.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, white_material),
    // Triangle::new(Vector{x: 5.0, y: 5.0, z: 6.0}, Vector{x: -5.0, y: 5.0, z: 6.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, white_material),
    // Triangle::new(Vector{x: -1.5, y: 4.99, z: -3.5}, Vector{x: -1.5, y: 4.99, z: -6.5}, Vector{x: 1.5, y: 4.99, z: -6.5}, emission_material),
    // Triangle::new(Vector{x: 1.5, y: 4.99, z: -3.5}, Vector{x: -1.5, y: 4.99, z: -3.5}, Vector{x: 1.5, y: 4.99, z: -6.5}, emission_material),
  ];

  let sphere_objects = vec![
    Sphere::new(Vector{x: -4.0, y: -3.2, z: 0.5}, 1.8, refraction_material),
    Sphere::new(Vector{x: 0.8, y: -3.2, z: -0.5}, 1.8, reflection_material),
    Sphere::new(Vector{x: 2.0, y: -3.2, z: 4.0}, 1.8, white_material),
  ];

  let objects = Objects::new(triangle_objects, sphere_objects);

  let hdr_image = image::hdr::HDRDecoder::new(BufReader::new(File::open("ibl.hdr").unwrap())).unwrap();

  println!("{:?}", hdr_image.metadata());

  let depth = 5;
  let limit_depth = 64;
  let hdr_image_height = hdr_image.metadata().height as usize;
  let hdr_image_data = hdr_image.read_image_hdr().unwrap();
  let hdr_image_longitude_offset = 2500;
  let background = Background::Ibl(hdr_image_data, hdr_image_height, hdr_image_longitude_offset);
  let scene_shared = Arc::new(Scene::new(objects, depth, limit_depth, background));

  let cpu_count = num_cpus::get();
  println!("cpu: {}", cpu_count);
  let pool = ThreadPool::new(cpu_count);
  let (tx, rx): (Sender<(usize, usize, Vector)>, Receiver<(usize, usize, Vector)>) = channel();

  let samples: usize = 10000;
  println!("samples: {}", samples);
  let mut output = box [[Vector{x: 0.0, y: 0.0, z: 0.0}; WIDTH]; HEIGHT];

  for i in CROP_OFFSET_BOTTOM..(CROP_OFFSET_BOTTOM + CROP_HEIGHT) {
    for j in CROP_OFFSET_LEFT..(CROP_OFFSET_LEFT + CROP_WIDTH) {
      let tx = tx.clone();
      let scene = scene_shared.clone();
      pool.execute(move || {
        let mut r: Vector = Default::default();
        for _ in 0..samples {
          let sample = cam.sample(i, j);
          // radiance: レンズ上の点までの放射輝度を計算
          // sensor_flux: レンズからイメージセンサー1ピクセルでの放射束を計算
          r = r + sample.sensor_flux(scene.radiance(sample.ray, 0, false)) * (1.0 / samples as f64);
          // r = &r + &radiance(ray, 0, false).smul(1.0 / samples as f64);
        }
        tx.send((i, j, Vector{x: clamp(r.x), y: clamp(r.y), z: clamp(r.z)})).unwrap();
      });
    }
  }

  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());

  for p in 0..CROP_WIDTH * CROP_HEIGHT - 1 {
    if p % CROP_WIDTH == 0 {
      print!("\rraytracing... ({:.0}/{:.0} : {:.0}%)", p, CROP_WIDTH * CROP_HEIGHT, (p as f64) / ((CROP_WIDTH * CROP_HEIGHT) as f64) * 100.0);
    }
    let (i, j, color) = rx.recv().unwrap();
    output[i][j] = color;
  }

  println!("\nwriting image...");

  let mut imgbuf = image::ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
  for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    let j = x as usize;
    let i = y as usize;
    *pixel = image::Rgb([to_int(output[i][WIDTH - j - 1].x), to_int(output[i][WIDTH - j - 1].y), to_int(output[i][WIDTH - j - 1].z)]);
  }
  let ref mut f = File::create(&Path::new(&format!("image_{}_{}.png", time::now().strftime("%Y%m%d%H%M%S").unwrap(), samples))).unwrap();
  let _ = image::ImageRgb8(imgbuf).save(f, image::PNG);

  let end_time = time::now();
  println!("end: {}", end_time.strftime("%+").unwrap());
  println!("elapse: {}s", (end_time - start_time).num_milliseconds() as f64 / 1000.0);
}
