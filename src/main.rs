#![feature(box_syntax)]

extern crate rand;
extern crate threadpool;
extern crate num_cpus;

use std::io::prelude::*;
use std::fs::File;
use std::ops::{Add, Sub, Mul};
// use std::num::Float;
use std::default::Default;
// use std::rand::random;
use rand::Rng;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::cmp;

#[derive(Debug, Copy, Clone, Default)]
struct Vector {
  x: f64,
  y: f64,
  z: f64
}

#[derive(Debug, Copy, Clone, Default)]
struct Ray {
  o: Vector,
  d: Vector
}

#[derive(Debug, Clone, Default)]
struct Sphere {
  radius: f64,
  position: Vector,
  emission: Vector,
  color: Vector,
}

trait Shape {
  fn intersect(self, r: Ray) -> f64;
}

impl Shape for Sphere {
  fn intersect(self, r: Ray) -> f64 {
    // Solve t^2*d.d + 2*t*(o-p).d + (o-p).(o-p)-R^2 = 0
    let eps = 1e-4;
    let op = &self.position - &r.o;
    let b = op.dot(&r.d);
    let mut det = b * b - op.dot(&op) + self.radius * self.radius;

    if det < 0.0 {
      return 0.0;
    } else {
      det = det.sqrt();
    }

    if (b - det) > eps {
      return b-det;
    }

    if (b + det) > eps {
      return b+det;
    }

    return 0.0;
  }
}

impl<'a> Add for &'a Vector {
  type Output = Vector;

  fn add(self, other: &'a Vector) -> Vector {
    Vector {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
  }
}

impl<'a> Sub for &'a Vector {
  type Output = Vector;

  fn sub(self, other: &'a Vector) -> Vector {
    Vector {x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}
  }
}

impl<'a> Mul for &'a Vector {
  type Output = Vector;

  fn mul(self, other: &'a Vector) -> Vector {
    Vector {x: self.x * other.x, y: self.y * other.y, z: self.z * other.z}
  }
}

trait VectorOps {
  fn smul(self, rhs: f64) -> Vector;
  fn norm(self) -> Vector;
  fn cross(self, rhs: Vector) -> Vector;
  fn dot(&self, rhs: &Vector) -> f64;
}

impl VectorOps for Vector {
  fn smul(self, other: f64) -> Vector {
    Vector {x: self.x * other, y: self.y * other, z: self.z * other}
  }

  fn norm(self) -> Vector {
    let normalize = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt() ;
    self.smul( normalize )
  }

  fn cross(self, b: Vector) -> Vector {
    Vector{x: self.y * b.z - self.z * b.y, y: self.z * b.x - self.x * b.z, z: self.x * b.y - self.y * b.x}
  }

  fn dot(&self, other: &Vector) -> f64 {
    (*self).x * (*other).x + (*self).y * (*other).y + (*self).z * (*other).z
  }
}

fn clamp(x: f64) -> f64 {
  if x < 0.0 {
    return 0.0;
  }
  if x > 1.0 {
    return 1.0;
  }
  return x
}

fn to_int(x: f64) -> i64 {
  return (clamp(x) * 255.0) as i64
}

fn get_light(ray: Ray, depth: usize) -> Vector{
  return ray.d;
}

const MAX_DEPTH: usize = 3;
const WIDTH: usize = 256;
const HEIGHT: usize = 256;
static PI: f64 = 3.14159265358979323846264338327950288_f64;

fn main() {
  let pool = ThreadPool::new(num_cpus::get());
  let (tx, rx):  (Sender<(usize, usize, Vector)>, Receiver<(usize, usize, Vector)>) = channel();

  let samples: usize = 1;
  let mut output = box [[Vector{x: 0.0, y: 0.0, z: 0.0}; WIDTH]; HEIGHT];
  let min_rsl: f64 = cmp::min(WIDTH, HEIGHT) as f64;

  for i in 0..HEIGHT {
    for j in 0..WIDTH {
      let tx = tx.clone();
      pool.execute(move || {
        let mut r: Vector = Default::default();
        for _ in 0..samples {
          let mut ray: Ray = Default::default();
          ray.o = Vector{x: 0.0, y: 0.0, z: 5.9};
          ray.d = Vector{x: ((j as f64) * 2.0 - (WIDTH as f64)) / min_rsl, y: ((i as f64) * 2.0 - (HEIGHT as f64)) / min_rsl, z: -1.0}.norm();
          r = &r + &get_light(ray, 0).smul(1.0 / samples as f64);
        }
        tx.send((i, j, Vector{x: clamp(r.x), y: clamp(r.y), z: clamp(r.z)})).unwrap();
      });
    }
  }

  for p in 0..WIDTH * HEIGHT - 1 {
    print!("\rRaytracing... ({:.0}%)", (p as f64) / ((WIDTH*HEIGHT) as f64) * 100.0);
    let (i, j, color) = rx.recv().unwrap();
    output[i][j] = color;
  }

  println!("\nWriting Image...");
  let mut f = File::create("image.ppm").unwrap();
  f.write_all( format!("P3\n{} {}\n{}\n", WIDTH, HEIGHT, 255).as_bytes() ).ok();
  for i in 0..HEIGHT {
    for j in 0..WIDTH {
      f.write_all( format!("{} {} {} ", to_int(output[HEIGHT - i - 1][j].x), to_int(output[HEIGHT - i - 1][j].y), to_int(output[HEIGHT - i - 1][j].z)).as_bytes() ).ok();
    }
  }
}
