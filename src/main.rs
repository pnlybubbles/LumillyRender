#![feature(box_syntax)]

extern crate rand;
extern crate threadpool;
extern crate num_cpus;
extern crate time;

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
    let normalize = 1.0 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
    self.smul( normalize )
  }

  fn cross(self, b: Vector) -> Vector {
    Vector{x: self.y * b.z - self.z * b.y, y: self.z * b.x - self.x * b.z, z: self.x * b.y - self.y * b.x}
  }

  fn dot(&self, other: &Vector) -> f64 {
    (*self).x * (*other).x + (*self).y * (*other).y + (*self).z * (*other).z
  }
}

#[derive(Debug, Copy, Clone, Default)]
struct Ray {
  o: Vector,
  d: Vector
}

#[derive(Debug, Copy, Clone, Default)]
struct Material {
  color: Vector,
  diffuse: f64,
  reflection: f64,
  refraction: f64,
  emmisive: f64,
}

#[derive(Debug, Copy, Clone, Default)]
struct Intersection {
  cross: bool,
  position: Vector,
  t: f64,
  normal: Vector,
  material: Material,
}

trait Shape {
  fn intersect(self, r: Ray) -> Intersection;
}

#[derive(Debug, Copy, Clone, Default)]
struct Sphere {
  radius: f64,
  position: Vector,
  material: Material,
}

impl Shape for Sphere {
  fn intersect(self, r: Ray) -> Intersection {
    let mut i: Intersection = Default::default();
    let co = &r.o - &self.position;
    let cod = co.dot(&r.d);
    let det = cod * cod - co.dot(&co) + self.radius * self.radius;

    if det < 0.0 {
      i.cross = false;
      return i;
    }
    let t = -cod - det.sqrt();
    if t < 0.0 {
      i.cross = false;
      return i;
    }
    i.cross = true;
    i.t = t;
    i.position = &r.o + &r.d.smul(t);
    i.normal = (&i.position - &self.position).norm();
    i.material = self.material;
    return i;
  }
}

#[derive(Debug, Copy, Clone, Default)]
struct Triangle {
  position0: Vector,
  position1: Vector,
  position2: Vector,
  normal: Vector,
  material: Material,
}

impl Shape for Triangle {
  fn intersect(self, r: Ray) -> Intersection {
    let mut i: Intersection = Default::default();
    let dn = r.d.dot(&self.normal);
    if dn >= 0.0 {
      i.cross = false;
      return i;
    }
    let t = (&self.position0 - &r.o).dot(&self.normal) / dn;
    if t < 0.0 {
      i.cross = false;
      return i;
    }
    let p = &r.o + &r.d.smul(t);
    let c0 = (&self.position1 - &self.position0).cross(&p - &self.position0);
    if c0.dot(&self.normal) < 0.0 {
      i.cross = false;
      return i;
    }
    let c1 = (&self.position2 - &self.position1).cross(&p - &self.position1);
    if c1.dot(&self.normal) < 0.0 {
      i.cross = false;
      return i;
    }
    let c2 = (&self.position0 - &self.position2).cross(&p - &self.position2);
    if c2.dot(&self.normal) < 0.0 {
      i.cross = false;
      return i;
    }
    i.cross = true;
    i.t = t;
    i.normal = self.normal;
    i.position = p;
    i.material = self.material;
    return i;
  }
}

#[derive(Debug, Copy, Clone)]
enum Mesh {
  Sphere(Sphere),
  Triangle(Triangle),
}

impl Mesh {
  fn intersect(&self, r: Ray) -> Intersection {
    match *self {
      Mesh::Sphere(i) => i.intersect(r),
      Mesh::Triangle(i) => i.intersect(r),
    }
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

fn get_light(r: Ray, depth: usize) -> Vector{
  if depth >= MAX_DEPTH {
    return Vector{x: 0.0, y: 0.0, z: 0.0};
  }
  let i = get_intersect(r);
  if !i.cross {
    return BG_COLOR;
  }
  let mut cd = i.material.diffuse;
  let mut cr = i.material.reflection;
  let mut cn = i.material.refraction;
  let mut ce = i.material.emmisive;
  let roulette = (cd + cr + cn + ce) * rand::random::<f64>();
  let mut t = 3; // emmisive
  if roulette < cd {
    // diffuse
    t = 0;
  } else if roulette < cd + cr {
    // reflection
    t = 1;
  } else if roulette < cd + cr + cn {
    // refraction
    t = 2;
  }

  // diffuse
  let mut diffuse_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if t == 0 {
    // let theta = PI * rand::random::<f64>();
    // let phi = 2.0 * PI * rand::random::<f64>();
    // let mut d = Vector{x: theta.sin() * phi.cos(), y: theta.sin() * phi.sin(), z: theta.cos()};
    // let mut dn = d.dot(&i.normal);
    // if dn < 0.0 {
    //   dn = -dn;
    //   d = d.smul(-1.0);
    // }
    let r1: f64 = 2.0 * PI* rand::random::<f64>();
    let r2: f64 = rand::random::<f64>();
    let r2s: f64 = r2.sqrt();

    let w = i.normal;
    let u = if w.x.abs() > 0.1 { Vector{x: 0.0, y: 1.0, z: 0.0} } else { Vector{x: 1.0, y: 0.0, z: 0.0 } }.cross(w).norm();
    let v = w.cross(u);

    let d = (&(&u.smul(r1.cos() * r2s) + &v.smul(r1.sin() * r2s)) + &w.smul((1.0 - r2).sqrt())).norm();
    let dn = d.dot(&i.normal);
    let new_ray = Ray{d: d, o: &i.position + &d.smul(0.01)};
    diffuse_color = &get_light(new_ray, depth + 1) * &i.material.color.smul(dn);
  }

  // reflection
  let mut reflection_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if t == 1 {
    let d = &r.d - &i.normal.smul(2.0 * r.d.dot(&i.normal));
    let new_ray = Ray{d: d, o: &i.position + &d.smul(0.01)};
    let color = get_light(new_ray, depth);
    reflection_color = color;
  }

  // refraction
  let mut refraction_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if t == 2 {
  }

  // emmisive
  let mut emmisive_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if t == 3 {
    emmisive_color = i.material.color;
  }

  return if t == 0 {
    diffuse_color.smul(cd + cr + cn)
  } else if t == 1 {
    reflection_color.smul(cd + cr + cn)
  } else if t == 2 {
    refraction_color.smul(cd + cr + cn)
  } else if t == 3 {
    emmisive_color.smul(ce)
  } else {
    Vector{x: 0.0, y: 0.0, z: 0.0}
  }
}

fn get_intersect(r: Ray) -> Intersection {
  let mut intersect: Intersection = Default::default();
  intersect.cross = false;
  for (_, obj) in OBJECTS.iter().enumerate() {
    let i = obj.intersect(r.clone());
    if i.cross && (!intersect.cross || intersect.t > i.t) {
      intersect = i;
    }
  }
  return intersect;
}

const YELLOW_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 1.0, y: 1.0, z: 0.4}};
const BLUE_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 0.4, y: 0.4, z: 1.0}};
const WHITE_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 1.0, y: 1.0, z: 1.0}};
const REFLECTION_MATERIAL: Material = Material{diffuse: 0.0, reflection: 1.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 1.0, y: 1.0, z: 1.0}};
const EMMISIVE_MATERIAL: Material = Material{diffuse: 0.0, reflection: 0.0, refraction: 0.0, emmisive: 1.0, color: Vector{x: 1.0 * 100.0, y: 1.0 * 100.0, z: 1.0 * 100.0}};

const OBJECTS: [Mesh; 16] = [
  // Mesh::Sphere(Sphere{radius: 1.0, position: Vector{x: 0.0, y: 0.0, z: 0.0}, material: WHITE_MATERIAL}),
  // Mesh::Triangle(Triangle{position0: Vector{x: 1.0, y: 1.0, z: 0.0}, position1: Vector{x: -1.0, y: 1.0, z: 0.0}, position2: Vector{x: 0.0, y: 0.0, z: 0.0}, normal: Vector{x: 0.0, y: 0.0, z: 1.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: 5.0, z: 0.0}, position1: Vector{x: -5.0, y: -5.0, z: 0.0}, position2: Vector{x: -5.0, y: 5.0, z: -10.0}, normal: Vector{x: 1.0, y: 0.0, z: 0.0}, material: YELLOW_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: -5.0, z: 0.0}, position1: Vector{x: -5.0, y: -5.0, z: -10.0}, position2: Vector{x: -5.0, y: 5.0, z: -10.0}, normal: Vector{x: 1.0, y: 0.0, z: 0.0}, material: YELLOW_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: 5.0, y: -5.0, z: 0.0}, position1: Vector{x: 5.0, y: 5.0, z: 0.0}, position2: Vector{x: 5.0, y: 5.0, z: -10.0}, normal: Vector{x: -1.0, y: 0.0, z: 0.0}, material: BLUE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: 5.0, y: -5.0, z: -10.0}, position1: Vector{x: 5.0, y: -5.0, z: 0.0}, position2: Vector{x: 5.0, y: 5.0, z: -10.0}, normal: Vector{x: -1.0, y: 0.0, z: 0.0}, material: BLUE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: 5.0, z: -10.0}, position1: Vector{x: -5.0, y: -5.0, z: -10.0}, position2: Vector{x: 5.0, y: 5.0, z: -10.0}, normal: Vector{x: 0.0, y: 0.0, z: 1.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: -5.0, z: -10.0}, position1: Vector{x: 5.0, y: -5.0, z: -10.0}, position2: Vector{x: 5.0, y: 5.0, z: -10.0}, normal: Vector{x: 0.0, y: 0.0, z: 1.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: -5.0, z: 0.0}, position1: Vector{x: -5.0, y: 5.0, z: 0.0}, position2: Vector{x: 5.0, y: 5.0, z: 0.0}, normal: Vector{x: 0.0, y: 0.0, z: -1.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: 5.0, y: -5.0, z: 0.0}, position1: Vector{x: -5.0, y: -5.0, z: 0.0}, position2: Vector{x: 5.0, y: 5.0, z: 0.0}, normal: Vector{x: 0.0, y: 0.0, z: -1.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: -5.0, z: -10.0}, position1: Vector{x: -5.0, y: -5.0, z: 0.0}, position2: Vector{x: 5.0, y: -5.0, z: -10.0}, normal: Vector{x: 0.0, y: 1.0, z: 0.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: -5.0, z: 0.0}, position1: Vector{x: 5.0, y: -5.0, z: 0.0}, position2: Vector{x: 5.0, y: -5.0, z: -10.0}, normal: Vector{x: 0.0, y: 1.0, z: 0.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -5.0, y: 5.0, z: 0.0}, position1: Vector{x: -5.0, y: 5.0, z: -10.0}, position2: Vector{x: 5.0, y: 5.0, z: -10.0}, normal: Vector{x: 0.0, y: -1.0, z: 0.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: 5.0, y: 5.0, z: 0.0}, position1: Vector{x: -5.0, y: 5.0, z: 0.0}, position2: Vector{x: 5.0, y: 5.0, z: -10.0}, normal: Vector{x: 0.0, y: -1.0, z: 0.0}, material: WHITE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: -1.0, y: 4.99, z: -4.0}, position1: Vector{x: -1.0, y: 4.99, z: -5.0}, position2: Vector{x: 1.0, y: 4.99, z: -5.0}, normal: Vector{x: 0.0, y: -1.0, z: 0.0}, material: EMMISIVE_MATERIAL}),
  Mesh::Triangle(Triangle{position0: Vector{x: 1.0, y: 4.99, z: -4.0}, position1: Vector{x: -1.0, y: 4.99, z: -4.0}, position2: Vector{x: 1.0, y: 4.99, z: -5.0}, normal: Vector{x: 0.0, y: -1.0, z: 0.0}, material: EMMISIVE_MATERIAL}),
  Mesh::Sphere(Sphere{position: Vector{x: -2.0, y: -3.5, z: -6.0}, radius: 1.5, material: REFLECTION_MATERIAL}),
  Mesh::Sphere(Sphere{position: Vector{x: 2.0, y: -3.5, z: -3.0}, radius: 1.5, material: WHITE_MATERIAL}),
];

const MAX_DEPTH: usize = 3;
const WIDTH: usize = 256;
const HEIGHT: usize = 256;
const PI: f64 = 3.14159265358979323846264338327950288_f64;
const BG_COLOR: Vector = Vector{x: 0.0, y: 0.0, z: 0.0};

fn main() {
  let pool = ThreadPool::new(num_cpus::get());
  let (tx, rx): (Sender<(usize, usize, Vector)>, Receiver<(usize, usize, Vector)>) = channel();

  let samples: usize = 5000;
  let mut output = box [[Vector{x: 0.0, y: 0.0, z: 0.0}; WIDTH]; HEIGHT];
  let min_rsl: f64 = cmp::min(WIDTH, HEIGHT) as f64;

  for i in 0..HEIGHT {
    for j in 0..WIDTH {
  // for i in 100..101 {
  //   for j in 100..101 {
      let tx = tx.clone();
      pool.execute(move || {
        let mut r: Vector = Default::default();
        for _ in 0..samples {
          let mut ray: Ray = Default::default();
          ray.o = Vector{x: 0.0, y: 0.0, z: 15.0};
          ray.d = Vector{x: ((j as f64) * 2.0 - (WIDTH as f64)) / min_rsl, y: ((i as f64) * 2.0 - (HEIGHT as f64)) / min_rsl, z: -3.0}.norm();
          r = &r + &get_light(ray, 0).smul(1.0 / samples as f64);
        }
        tx.send((i, j, Vector{x: clamp(r.x), y: clamp(r.y), z: clamp(r.z)})).unwrap();
      });
    }
  }

  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());

  for p in 0..WIDTH * HEIGHT - 1 {
    print!("\rRaytracing... ({:.0}/{:.0} : {:.0}%)", p, WIDTH * HEIGHT, (p as f64) / ((WIDTH * HEIGHT) as f64) * 100.0);
    let (i, j, color) = rx.recv().unwrap();
    output[i][j] = color;
  }

  println!("\nWriting Image...");
  let mut f = File::create(format!("image_{}.ppm", time::now().strftime("%Y%m%d%H%M%S").unwrap())).unwrap();
  f.write_all( format!("P3\n{} {}\n{}\n", WIDTH, HEIGHT, 255).as_bytes() ).ok();
  for i in 0..HEIGHT {
    for j in 0..WIDTH {
      f.write_all( format!("{} {} {} ", to_int(output[HEIGHT - i - 1][j].x), to_int(output[HEIGHT - i - 1][j].y), to_int(output[HEIGHT - i - 1][j].z)).as_bytes() ).ok();
    }
  }

  let end_time = time::now();
  println!("end: {}", end_time.strftime("%+").unwrap());
  println!("elapse: {}s", (end_time - start_time).num_milliseconds() as f64 / 1000.0);
}
