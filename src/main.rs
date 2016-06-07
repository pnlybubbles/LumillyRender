#![feature(box_syntax)]

extern crate rand;
extern crate threadpool;
extern crate num_cpus;
extern crate time;

#[macro_use]
extern crate lazy_static;

use std::io::prelude::*;
use std::fs::File;
use std::ops::{Add, Sub, Mul};
// use std::num::Float;
use std::default::Default;
// use std::rand::random;
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

impl Sphere {
  fn new(position: Vector, radius: f64, material: Material) -> Sphere {
    Sphere {
      radius: radius,
      position: position,
      material: material,
    }
  }
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

impl Triangle {
  fn new(position0: Vector, position1: Vector, position2: Vector, material: Material) -> Triangle {
    Triangle {
      position0: position0,
      position1: position1,
      position2: position2,
      normal: (&position1 - &position0).cross(&position2 - &position0).norm(),
      material: material,
    }
  }
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

#[derive(Debug, Clone, Default)]
struct Objects {
  triangles: Vec<Triangle>,
  spheres: Vec<Sphere>,
  emmisive_triangles: Vec<Triangle>,
  emmisive_triangles_area: Vec<f64>,
  emmisive_triangles_area_total: f64,
}

impl Objects {
  fn new(triangles: &[Triangle], spheres: &[Sphere]) -> Objects {
    let mut emmisive_triangles: Vec<Triangle> = Default::default();
    let mut emmisive_triangles_area: Vec<f64> = Default::default();
    let mut all_triangles: Vec<Triangle> = Default::default();
    let mut all_spheres: Vec<Sphere> = Default::default();
    let mut emmisive_triangles_area_total = 0.0;
    for v in triangles {
      if v.material.emmisive > 0.0 {
        emmisive_triangles.push(*v);
        let cr = (&v.position1 - &v.position0).cross(&v.position2 - &v.position0);
        let area = 0.5 * cr.dot(&cr).sqrt();
        emmisive_triangles_area.push(area);
        emmisive_triangles_area_total += area;
      }
      all_triangles.push(*v);
    }
    for v in spheres {
      all_spheres.push(*v);
    }
    Objects {
      triangles: all_triangles,
      spheres: all_spheres,
      emmisive_triangles: emmisive_triangles,
      emmisive_triangles_area: emmisive_triangles_area,
      emmisive_triangles_area_total: emmisive_triangles_area_total,
    }
  }

  fn get_intersect(&self, r: Ray) -> Intersection {
    let mut intersect: Intersection = Default::default();
    intersect.cross = false;
    for obj in &self.triangles {
      let i = obj.intersect(r);
      if i.cross && (!intersect.cross || intersect.t > i.t) {
        intersect = i;
      }
    }
    for obj in &self.spheres {
      let i = obj.intersect(r);
      if i.cross && (!intersect.cross || intersect.t > i.t) {
        intersect = i;
      }
    }
    return intersect;
  }

  fn get_emmisive_point(&self) -> Vector {
    let roulette = &self.emmisive_triangles_area_total * rand::random::<f64>();
    let mut area = 0.0;
    let mut ret: Vector = Default::default();
    for (i, obj) in (&self.emmisive_triangles).iter().enumerate() {
      area += (&self.emmisive_triangles_area)[i];
      if roulette <= area {
        let mut s = rand::random::<f64>();
        let mut t = rand::random::<f64>();
        if s + t > 1.0 {
          s = 1.0 - s;
          t = 1.0 - t;
        }
        ret = Vector{
          x: (1.0 - s - t) * obj.position0.x + s * obj.position1.x + t * obj.position2.x,
          y: (1.0 - s - t) * obj.position0.y + s * obj.position1.y + t * obj.position2.y,
          z: (1.0 - s - t) * obj.position0.z + s * obj.position1.z + t * obj.position2.z,
        };
      }
    }
    return ret;
  }

  fn get_emmisive_solid_angle(&self, position: Vector) -> f64 {
    let mut solid_angle = 0.0;
    for obj in &self.emmisive_triangles {
      let pe0 = (&obj.position0 - &position).norm();
      let pe1 = (&obj.position1 - &position).norm();
      let pe2 = (&obj.position2 - &position).norm();
      let cr = (&pe1 - &pe0).cross(&pe2 - &pe0);
      solid_angle += cr.dot(&cr).sqrt();
    }
    return solid_angle;
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

fn get_light(r: Ray, depth: usize, no_emmisive: bool) -> Vector{
  if depth >= MAX_DEPTH {
    return Vector{x: 0.0, y: 0.0, z: 0.0};
  }
  let i = OBJECTS.get_intersect(r);
  if !i.cross {
    return BG_COLOR;
  }
  let cd = i.material.diffuse;
  let cr = i.material.reflection;
  let mut cn = i.material.refraction;
  let ce = i.material.emmisive;
  let roulette = (cd + cr + cn + ce) * rand::random::<f64>();
  let mut tp = 3; // emmisive
  if roulette < cd {
    // diffuse
    tp = 0;
  } else if roulette < cd + cr {
    // reflection
    tp = 1;
  } else if roulette < cd + cr + cn {
    // refraction
    tp = 2;
  }

  // diffuse
  let mut diffuse_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if tp == 0 {
    // emmisive direction
    let emmisive_position = OBJECTS.get_emmisive_point();
    let ip_ep = &emmisive_position - &i.position;
    let d_e = ip_ep.norm();
    let test_ray = Ray{
      o: &i.position + &d_e.smul(0.01),
      d: d_e,
    };
    let test_i = OBJECTS.get_intersect(test_ray);
    if test_i.cross && test_i.material.emmisive == 1.0 {
      let prob = OBJECTS.get_emmisive_solid_angle(i.position) / (2.0 * PI);
      let mut ip_ep_n = d_e.dot(&i.normal);
      if ip_ep_n < 0.0 {
        ip_ep_n = 0.0;
      }
      diffuse_color = (&test_i.material.color * &i.material.color).smul(prob * ip_ep_n);
    }
    // other direction

    // orthonormal coordinate with cos importance sampling
    // let r1: f64 = 2.0 * PI* rand::random::<f64>();
    // let r2: f64 = rand::random::<f64>();
    // let r2s: f64 = r2.sqrt();

    // let w = i.normal;
    // let u = if w.x.abs() > 0.1 { Vector{x: 0.0, y: 1.0, z: 0.0} } else { Vector{x: 1.0, y: 0.0, z: 0.0 } }.cross(w).norm();
    // let v = w.cross(u);

    // let d = (&(&u.smul(r1.cos() * r2s) + &v.smul(r1.sin() * r2s)) + &w.smul((1.0 - r2).sqrt())).norm();
    // let new_ray = Ray{d: d, o: &i.position + &d.smul(0.01)};
    // diffuse_color = &diffuse_color + &(&get_light(new_ray, depth + 1, true) * &i.material.color);

    // orthonormal coordinate
    let theta = PI * rand::random::<f64>() * 0.5;
    let phi = 2.0 * PI * rand::random::<f64>();

    let w = i.normal;
    let u = if w.x.abs() > 0.1 { Vector{x: 0.0, y: 1.0, z: 0.0} } else { Vector{x: 1.0, y: 0.0, z: 0.0 } }.cross(w);
    let v = w.cross(u);

    let d = (&(&u.smul(theta.sin() * phi.cos()) + &v.smul(theta.sin() * phi.sin())) + &w.smul(theta.cos())).norm();
    let dn = d.dot(&i.normal);
    let new_ray = Ray{d: d, o: &i.position + &d.smul(0.01)};
    diffuse_color = &diffuse_color + &(&get_light(new_ray, depth + 1, true) * &i.material.color).smul(dn / PI);

    // normal inversing
    // let theta = PI * rand::random::<f64>();
    // let phi = 2.0 * PI * rand::random::<f64>();
    // let mut d = Vector{x: theta.sin() * phi.cos(), y: theta.sin() * phi.sin(), z: theta.cos()};
    // let mut dn = d.dot(&i.normal);
    // if dn < 0.0 {
    //   dn = -dn;
    //   d = d.smul(-1.0);
    // }
    // let new_ray = Ray{d: d, o: &i.position + &d.smul(0.01)};
    // diffuse_color = &diffuse_color + &(&get_light(new_ray, depth + 1, true) * &i.material.color).smul(dn / PI);
  }

  // reflection
  let mut reflection_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if tp == 1 {
    let d = &r.d - &i.normal.smul(2.0 * r.d.dot(&i.normal));
    let new_ray = Ray{d: d, o: &i.position + &d.smul(0.01)};
    reflection_color = get_light(new_ray, depth, false);
  }

  // refraction
  let mut refraction_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if tp == 2 {
    let mut d = Vector{x: 0.0, y: 0.0, z: 0.0};
    let mut o = Vector{x: 0.0, y: 0.0, z: 0.0};
    let dn = r.d.dot(&i.normal);
    let eta = 1.5;
    if dn < 0.0{
      // incidence
      let det = 1.0 - (1.0 + dn) * (1.0 + dn) / (eta * eta);
      if det < 0.0 {
        cn = 0.0;
      } else {
        d = &i.normal.smul(-dn / eta - det.sqrt()) + &r.d.smul(1.0 / eta);
        o = &i.position - &i.normal.smul(0.01)
      }
    } else {
      // outgoing
      let det = 1.0 - (1.0 - dn) * (1.0 - dn) * (eta * eta);
      if det < 0.0 {
        cn = 0.0;
      } else {
        d = &i.normal.smul(-(dn * eta - det.sqrt())) + &r.d.smul(eta);
        o = &i.position + &i.normal.smul(0.01)
      }
    }
    if cn != 0.0 {
      let new_ray = Ray{d: d.norm(), o: o};
      refraction_color = get_light(new_ray, depth, false);
    }
  }

  // emmisive
  let mut emmisive_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if tp == 3 {
    if !no_emmisive {
      emmisive_color = i.material.color;
    }
  }

  return if tp == 0 {
    diffuse_color.smul(cd + cr + cn)
  } else if tp == 1 {
    reflection_color.smul(cd + cr + cn)
  } else if tp == 2 {
    refraction_color.smul(cd + cr + cn)
  } else if tp == 3 {
    emmisive_color.smul(ce)
  } else {
    Vector{x: 0.0, y: 0.0, z: 0.0}
  }
}

const YELLOW_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 1.0, y: 1.0, z: 0.5}};
const BLUE_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 0.5, y: 0.5, z: 1.0}};
const WHITE_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emmisive: 0.0, color: Vector{x: 1.0, y: 1.0, z: 1.0}};
const REFLECTION_MATERIAL: Material = Material{diffuse: 0.1, reflection: 0.9, refraction: 0.0, emmisive: 0.0, color: Vector{x: 0.0, y: 0.0, z: 0.0}};
const REFRACTION_MATERIAL: Material = Material{diffuse: 0.0, reflection: 0.2, refraction: 0.8, emmisive: 0.0, color: Vector{x: 0.0, y: 0.0, z: 0.0}};
const EMMISIVE_MATERIAL: Material = Material{diffuse: 0.0, reflection: 0.0, refraction: 0.0, emmisive: 1.0, color: Vector{x: 11.0, y: 11.0, z: 11.0}};

lazy_static! {
  static ref TRIANGLE_OBJECTS: [Triangle; 14] = [
    Triangle::new(Vector{x: -5.0, y: 5.0, z: 0.0}, Vector{x: -5.0, y: -5.0, z: 0.0}, Vector{x: -5.0, y: 5.0, z: -10.0}, YELLOW_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: -5.0, z: 0.0}, Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: -5.0, y: 5.0, z: -10.0}, YELLOW_MATERIAL),
    Triangle::new(Vector{x: 5.0, y: -5.0, z: 0.0}, Vector{x: 5.0, y: 5.0, z: 0.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, BLUE_MATERIAL),
    Triangle::new(Vector{x: 5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: -5.0, z: 0.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, BLUE_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: 5.0, z: -10.0}, Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: -5.0, z: -10.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: -5.0, z: 0.0}, Vector{x: -5.0, y: 5.0, z: 0.0}, Vector{x: 5.0, y: 5.0, z: 0.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: 5.0, y: -5.0, z: 0.0}, Vector{x: -5.0, y: -5.0, z: 0.0}, Vector{x: 5.0, y: 5.0, z: 0.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: -5.0, z: -10.0}, Vector{x: -5.0, y: -5.0, z: 0.0}, Vector{x: 5.0, y: -5.0, z: -10.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: -5.0, z: 0.0}, Vector{x: 5.0, y: -5.0, z: 0.0}, Vector{x: 5.0, y: -5.0, z: -10.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: -5.0, y: 5.0, z: 0.0}, Vector{x: -5.0, y: 5.0, z: -10.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: 5.0, y: 5.0, z: 0.0}, Vector{x: -5.0, y: 5.0, z: 0.0}, Vector{x: 5.0, y: 5.0, z: -10.0}, WHITE_MATERIAL),
    Triangle::new(Vector{x: -1.5, y: 4.99, z: -3.5}, Vector{x: -1.5, y: 4.99, z: -6.5}, Vector{x: 1.5, y: 4.99, z: -6.5}, EMMISIVE_MATERIAL),
    Triangle::new(Vector{x: 1.5, y: 4.99, z: -3.5}, Vector{x: -1.5, y: 4.99, z: -3.5}, Vector{x: 1.5, y: 4.99, z: -6.5}, EMMISIVE_MATERIAL),
  ];

  static ref SPHERE_OBJECTS: [Sphere; 2] = [
    Sphere::new(Vector{x: -2.0, y: 2.0, z: -7.0}, 1.8, REFLECTION_MATERIAL),
    Sphere::new(Vector{x: 2.0, y: -3.2, z: -3.0}, 1.8, REFRACTION_MATERIAL),
  ];

  static ref OBJECTS: Objects = Objects::new(&*TRIANGLE_OBJECTS, &*SPHERE_OBJECTS);
}

const MAX_DEPTH: usize = 5;
const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const PI: f64 = 3.14159265358979323846264338327950288_f64;
const BG_COLOR: Vector = Vector{x: 0.0, y: 0.0, z: 0.0};

fn main() {
  let cpu_count = num_cpus::get();
  println!("cpu: {}", cpu_count);
  let pool = ThreadPool::new(cpu_count);
  let (tx, rx): (Sender<(usize, usize, Vector)>, Receiver<(usize, usize, Vector)>) = channel();

  let samples: usize = 10;
  let mut output = box [[Vector{x: 0.0, y: 0.0, z: 0.0}; WIDTH]; HEIGHT];
  let min_rsl: f64 = cmp::min(WIDTH, HEIGHT) as f64;

  for i in 0..HEIGHT {
    for j in 0..WIDTH {
      let tx = tx.clone();
      pool.execute(move || {
        let mut r: Vector = Default::default();
        for _ in 0..samples {
          let mut ray: Ray = Default::default();
          ray.o = Vector{x: 0.0, y: 0.0, z: 15.0};
          ray.d = Vector{
            x: ((j as f64 + rand::random::<f64>()) * 2.0 - (WIDTH as f64 + 1.0)) / min_rsl,
            y: ((i as f64 + rand::random::<f64>()) * 2.0 - (HEIGHT as f64 + 1.0)) / min_rsl,
            z: -3.0,
          }.norm();
          r = &r + &get_light(ray, 0, false).smul(1.0 / samples as f64);
        }
        tx.send((i, j, Vector{x: clamp(r.x), y: clamp(r.y), z: clamp(r.z)})).unwrap();
      });
    }
  }

  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());

  for p in 0..WIDTH * HEIGHT - 1 {
    print!("\rraytracing... ({:.0}/{:.0} : {:.0}%)", p, WIDTH * HEIGHT, (p as f64) / ((WIDTH * HEIGHT) as f64) * 100.0);
    let (i, j, color) = rx.recv().unwrap();
    output[i][j] = color;
  }

  println!("\nwriting image...");
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
