#![feature(box_syntax)]

extern crate threadpool;
extern crate num_cpus;
extern crate time;
extern crate tobj;
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
mod img;

use std::fs::File;
use std::io::{BufReader};
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
use constant::*;
use img::Img;

// const HEIGHT: usize = 270 * 2;
// const WIDTH: usize = 480 * 2;

// const CROP_OFFSET_TOP: usize = 255;
// const CROP_OFFSET_RIGHT: usize = 230;
// const CROP_HEIGHT: usize = 165;
// const CROP_WIDTH: usize = 165;

// const HEIGHT: usize = 270 * 2;
// const WIDTH: usize = 480 * 2;

// const CROP_OFFSET_TOP: usize = 0;
// const CROP_OFFSET_RIGHT: usize = 0;
// const CROP_HEIGHT: usize = 270 * 2;
// const CROP_WIDTH: usize = 480 * 2;

fn main() {
  let camera_position = Vector{x: -11.5, y: 1.0, z: 13.0};
  let screen_direction = Vector{x: 8.18, y: -2.0, z: -9.0};
  let focus_distance = 3.0 + screen_direction.len();
  // let camera_position = Vector{x: -5.0, y: 1.5, z: 6.0};
  // let screen_direction = Vector{x: 2.5, y: -0.5, z: -3.05};
  // let focus_distance = 3.5 + screen_direction.len();
  let lens_radius = 0.3;
  // let lens_radius = 10e-5;
  let sensor_sensitivity = 1.0;
  // let screen_height = 2.5;
  let screen_height = 10.0;
  let screen_width = screen_height * (WIDTH as f64 / HEIGHT as f64);
  let cam = Camera::new(camera_position, screen_direction, HEIGHT, WIDTH, screen_height, screen_width, focus_distance, lens_radius, sensor_sensitivity);

  // let yellow_material: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.75, 0.75, 0.25)};
  // let blue_material: Material = Material{diffuse: 1.0, reflection: 0.0, roughness: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.25, 0.25, 0.75)};
  let white_material: Material = Material{diffuse: 1.0, reflection: 0.0, roughness: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.75, 0.75, 0.75)};
  let reflection_material_0: Material = Material{diffuse: 0.0, reflection: 1.0, roughness: 0.0, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let reflection_material_1: Material = Material{diffuse: 0.0, reflection: 1.0, roughness: 0.2, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let reflection_material_2: Material = Material{diffuse: 0.0, reflection: 1.0, roughness: 0.4, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let reflection_material_3: Material = Material{diffuse: 0.0, reflection: 1.0, roughness: 0.6, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let reflection_material_4: Material = Material{diffuse: 0.0, reflection: 1.0, roughness: 0.8, refraction: 0.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  let refraction_material_0: Material = Material{diffuse: 0.0, reflection: 0.0, roughness: 0.0, refraction: 1.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let refraction_material_1: Material = Material{diffuse: 0.0, reflection: 0.0, roughness: 0.01, refraction: 1.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let refraction_material_2: Material = Material{diffuse: 0.0, reflection: 0.0, roughness: 0.1, refraction: 1.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let refraction_material_3: Material = Material{diffuse: 0.0, reflection: 0.0, roughness: 0.2, refraction: 1.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let refraction_material_4: Material = Material{diffuse: 0.0, reflection: 0.0, roughness: 0.4, refraction: 1.0, emission: Vector::new(0.0, 0.0, 0.0), color: Vector::new(0.99, 0.99, 0.99)};
  // let emission_material: Material = Material{diffuse: 1.0, reflection: 0.0, roughness: 0.0, refraction: 0.0, emission: Vector::new(12.0, 12.0, 12.0), color: Vector::new(1.0, 1.0, 1.0)};
  // let emission_material: Material = Material{diffuse: 1.0, reflection: 0.0, roughness: 0.0, refraction: 0.0, emission: Vector::new(1.0, 1.0, 1.0), color: Vector::new(1.0, 1.0, 1.0)};

  let mut triangle_objects = vec![
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
    // Sphere::new(Vector{x: -4.0, y: -3.2, z: 0.5}, 1.8, refraction_material),
    // Sphere::new(Vector{x: 0.8, y: -3.2, z: -0.5}, 1.8, white_material),
    // Sphere::new(Vector{x: -5.0, y: -4.0, z: 4.0}, 1.0, refraction_material_0),
    // Sphere::new(Vector{x: -2.5, y: -4.0, z: 4.0}, 1.0, refraction_material_1),
    // Sphere::new(Vector::new(0.0, -1.0, 4.0), 1.0, refraction_material_0),
    // Sphere::new(Vector{x: 2.5, y: -4.0, z: 4.0}, 1.0, refraction_material_3),
    // Sphere::new(Vector{x: 5.0, y: -4.0, z: 4.0}, 1.0, refraction_material_4),
    // Sphere::new(Vector{x: -7.0, y: -3.84, z: 6.0}, 0.16, reflection_material_0),
    // Sphere::new(Vector{x: 0.8, y: -3.2, z: -0.5}, 0.36, refraction_material_0),
    // Sphere::new(Vector{x: 2.0, y: -3.2, z: 4.0}, 0.64, refraction_material_0),
    // Sphere::new(Vector{x: 2.0, y: -3.2, z: 4.0}, 1.0, refraction_material_0),
    // Sphere::new(Vector{x: 2.0, y: -3.2, z: 4.0}, 2.0, refraction_material_0),
    Sphere::new(Vector{x: -4.0, y: -3.2, z: 0.5}, 1.8, refraction_material_0),
    Sphere::new(Vector{x: 0.8, y: -3.2, z: -0.5}, 1.8, white_material),
    Sphere::new(Vector{x: 2.0, y: -3.2, z: 4.0}, 1.8, reflection_material_0),
  ];

  // let cube = tobj::load_obj(&Path::new("models/cube/cube.obj"));
  // assert!(cube.is_ok());
  // let (models, materials) = cube.unwrap();
  // println!("# of models: {}", models.len());
  // println!("# of materials: {}", materials.len());
  // let x_rotate = PI / 3.3;
  // for (i, m) in models.iter().enumerate() {
  //   let mesh = &m.mesh;
  //   println!("model[{}].name = \'{}\'", i, m.name);
  //   println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);
  //   println!("Size of model[{}].indices: {}", i, mesh.indices.len());
  //   println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
  //   for f in 0..mesh.indices.len() / 3 {
  //     let mut polygon = [Vector::default(); 3];
  //     for i in 0..3 {
  //       let index: usize = f * 3 + i;
  //       polygon[i] = Vector::new(
  //         mesh.positions[mesh.indices[index] as usize * 3] as f64,
  //         mesh.positions[mesh.indices[index] as usize * 3 + 1] as f64,
  //         mesh.positions[mesh.indices[index] as usize * 3 + 2] as f64,
  //       );
  //       // polygon[i] = Vector::new(
  //       //   polygon[i].x,
  //       //   x_rotate.cos() * polygon[i].y - x_rotate.sin() * polygon[i].z,
  //       //   x_rotate.sin() * polygon[i].y + x_rotate.cos() * polygon[i].z,
  //       // )
  //     }
  //     triangle_objects.push(Triangle::new(polygon[0], polygon[1], polygon[2], white_material));
  //   }
  // }

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

  let samples: usize = 100000;
  println!("samples: {}", samples);
  let mut output = box [[Vector::new(0.0, 0.0, 0.0); WIDTH]; HEIGHT];

  for _ in 0..samples {
    for i in CROP_OFFSET_TOP..(CROP_OFFSET_TOP + CROP_HEIGHT) {
      let tx = tx.clone();
      let scene = scene_shared.clone();
      pool.execute(move || {
        for j in CROP_OFFSET_RIGHT..(CROP_OFFSET_RIGHT + CROP_WIDTH) {
          let sample = cam.sample(i, j);
          // radiance: レンズ上の点までの放射輝度を計算
          // sensor_flux: レンズからイメージセンサー1ピクセルでの放射束を計算
          let r = sample.sensor_flux(scene.radiance(sample.ray, 0, false));
          // r = &r + &radiance(ray, 0, false).smul(1.0 / samples as f64);
          tx.send((i, j, Vector::new(r.x, r.y, r.z))).unwrap();
        }
      });
    }
  }

  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());
  let mut next_save_time = SAVE_IMAGE_INTERVAL - SAVE_IMAGE_INTERVAL_ERROR;
  let time_limit_with_error = TIME_LIMIT - SAVE_IMAGE_INTERVAL_ERROR;

  for s in 0..samples {
    print!("\rraytracing... ({:.0}/{:.0} : {:.0}%) ", s, samples, s as f64 / samples as f64 * 100.0);
    for _ in 0..CROP_HEIGHT {
      for _ in 0..CROP_WIDTH {
        let (i, j, color) = rx.recv().unwrap();
        output[i][j] = output[i][j] + color;
      }
    }
    let save_time = time::now();
    let elapse_time = (save_time - start_time).num_seconds() as f64;
    if elapse_time > next_save_time {
      println!("save: {}s", elapse_time);
      Img::save_png(&output, s);
      next_save_time += SAVE_IMAGE_INTERVAL;
    }
    if elapse_time > time_limit_with_error {
      Img::save_png(&output, s);
      println!("exceeded the time limit: {}s", elapse_time);
      std::process::exit(0);
    }
  }

  let end_time = time::now();
  println!("end: {}", end_time.strftime("%+").unwrap());
  println!("elapse: {}s", (end_time - start_time).num_milliseconds() as f64 / 1000.0);
}
