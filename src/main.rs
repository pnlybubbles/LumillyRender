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
  emission: Vector,
  diffuse: f64,
  reflection: f64,
  refraction: f64,
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
    let t1 = -cod - det.sqrt();
    let t2 = -cod + det.sqrt();
    if t1 < EPS && t2 < EPS {
      i.cross = false;
      return i;
    }
    if t1 > EPS {
      i.t = t1;
    } else {
      i.t = t2;
    }
    i.cross = true;
    i.position = &r.o + &r.d.smul(i.t);
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
    if t < EPS {
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
  emission_triangles: Vec<Triangle>,
  emission_triangles_area: Vec<f64>,
  emission_triangles_area_total: f64,
}

impl Objects {
  fn new(triangles: &[Triangle], spheres: &[Sphere]) -> Objects {
    let mut emission_triangles: Vec<Triangle> = Default::default();
    let mut emission_triangles_area: Vec<f64> = Default::default();
    let mut all_triangles: Vec<Triangle> = Default::default();
    let mut all_spheres: Vec<Sphere> = Default::default();
    let mut emission_triangles_area_total = 0.0;
    for v in triangles {
      if v.material.emission.dot(&v.material.emission) > 0.0 {
        emission_triangles.push(*v);
        let cr = (&v.position1 - &v.position0).cross(&v.position2 - &v.position0);
        let area = 0.5 * cr.dot(&cr).sqrt();
        emission_triangles_area.push(area);
        emission_triangles_area_total += area;
      }
      all_triangles.push(*v);
    }
    for v in spheres {
      all_spheres.push(*v);
    }
    Objects {
      triangles: all_triangles,
      spheres: all_spheres,
      emission_triangles: emission_triangles,
      emission_triangles_area: emission_triangles_area,
      emission_triangles_area_total: emission_triangles_area_total,
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

  // fn get_emission_point(&self) -> Vector {
  //   let roulette = &self.emission_triangles_area_total * rand::random::<f64>();
  //   let mut area = 0.0;
  //   let mut ret: Vector = Default::default();
  //   for (i, obj) in (&self.emission_triangles).iter().enumerate() {
  //     area += (&self.emission_triangles_area)[i];
  //     if roulette <= area {
  //       let mut s = rand::random::<f64>();
  //       let mut t = rand::random::<f64>();
  //       if s + t > 1.0 {
  //         s = 1.0 - s;
  //         t = 1.0 - t;
  //       }
  //       ret = Vector{
  //         x: (1.0 - s - t) * obj.position0.x + s * obj.position1.x + t * obj.position2.x,
  //         y: (1.0 - s - t) * obj.position0.y + s * obj.position1.y + t * obj.position2.y,
  //         z: (1.0 - s - t) * obj.position0.z + s * obj.position1.z + t * obj.position2.z,
  //       };
  //     }
  //   }
  //   return ret;
  // }

  // fn get_emission_solid_angle(&self, position: Vector) -> f64 {
  //   let mut solid_angle = 0.0;
  //   for obj in &self.emission_triangles {
  //     let pe0 = (&obj.position0 - &position).norm();
  //     let pe1 = (&obj.position1 - &position).norm();
  //     let pe2 = (&obj.position2 - &position).norm();
  //     let cr = (&pe1 - &pe0).cross(&pe2 - &pe0);
  //     solid_angle += cr.dot(&cr).sqrt();
  //   }
  //   return solid_angle;
  // }
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
  return (clamp(x).powf(1.0 / 2.2) * 255.0) as i64
}

fn radiance(r: Ray, depth: usize) -> Vector{
  // すべてのオブジェクトと当たり判定を行う
  let i = OBJECTS.get_intersect(r);
  // 当たらなかった場合は背景色を返す
  if !i.cross {
    return BG_COLOR;
  }
  if i.material.emission.dot(&i.material.emission) > 0.0 {
    return i.material.emission;
  }
  // 再帰抑制用のロシアンルーレットの確率を決定する
  // 鏡面反射、屈折の時は必ず次のパスをトレースする
  // 拡散反射の時は各色の反射率のうち最大のものを使う
  let mut continue_rr_prob = if i.material.reflection > 0.0 || i.material.refraction > 0.0 { 1.0 } else {
    i.material.color.x.max(i.material.color.y).max(i.material.color.z)
  };
  // スタックオーバーフロー対策のために反射回数の限界値を超えたら極端に確率を下げる
  if depth > DEPTH_LIMIT {
    continue_rr_prob *= (0.5f64).powi((depth - DEPTH_LIMIT) as i32);
  }
  // 最初の数回の反射では必ず次のパスをトレースするようにする
  if depth <= DEPTH {
    continue_rr_prob = 1.0;
  }
  // ロシアンルーレットで再帰を抑制
  if continue_rr_prob != 1.0 && rand::random::<f64>() >= continue_rr_prob {
    return i.material.emission;
  }
  // if depth > DEPTH {
  //   return i.material.emission;
  // }
  // 拡散反射、鏡面反射、屈折のそれぞれの割合からどの処理を行うかをロシアンルーレットで決定する
  let brdf_type_rr = (i.material.diffuse + i.material.reflection + i.material.refraction) * rand::random::<f64>();
  let mut brdf_type_rr_prob = i.material.refraction;
  let mut brdf_type = 2;
  if brdf_type_rr < i.material.diffuse {
    brdf_type_rr_prob = i.material.diffuse;
    brdf_type = 0;
  } else if brdf_type_rr < i.material.diffuse + i.material.reflection {
    brdf_type_rr_prob = i.material.reflection;
    brdf_type = 1;
  }
  // 拡散反射
  if brdf_type == 0 {
    // 乱数を生成(半球上で一様にサンプル)
    // let theta = PI * rand::random::<f64>() * 0.5;
    // let phi = 2.0 * PI * rand::random::<f64>();
    // 乱数を生成(cosにしたがって重点的にサンプル)
    let r1: f64 = 2.0 * PI* rand::random::<f64>();
    let r2: f64 = rand::random::<f64>();
    let r2s: f64 = r2.sqrt();

    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = i.normal;
    let u = if w.x.abs() > EPS { Vector{x: 0.0, y: 1.0, z: 0.0} } else { Vector{x: 1.0, y: 0.0, z: 0.0 } }.cross(w);
    let v = w.cross(u);

    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // let d = &(&u.smul(theta.sin() * phi.cos()) + &v.smul(theta.sin() * phi.sin())) + &w.smul(theta.cos());
    let d = &(&u.smul(r1.cos() * r2s) + &v.smul(r1.sin() * r2s)) + &w.smul((1.0 - r2).sqrt());
    // cos項を計算
    let dn = d.dot(&i.normal);
    // 新しいレイを作る
    let new_ray = Ray{d: d, o: i.position};
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると σ / π
    let brdf = i.material.color.smul(1.0 / PI);
    // 確率密度関数はサンプルした乱数が半球上で一様分布なので 1 / 2π
    // let pdf = 1.0 / (2.0 * PI);
    // 確率密度関数はサンプルした乱数がcosに依存した分布なので cosθ / π
    let pdf = dn / PI;
    // レンダリング方程式にしたがって放射輝度を計算する
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (BRDFタイプ用と再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    // return &i.material.emission + &(&brdf * &radiance(new_ray, depth + 1).smul(dn / (pdf * continue_rr_prob * brdf_type_rr_prob)));
    return &i.material.emission + &(&i.material.color * &radiance(new_ray, depth + 1).smul(1.0 / continue_rr_prob));
  }
  return Vector{x: 0.0, y: 0.0, z: 0.0};
}

fn radiance_old(r: Ray, depth: usize, no_emmisive: bool) -> Vector{
  let i = OBJECTS.get_intersect(r);
  if !i.cross {
    return BG_COLOR;
  }
  if depth >= DEPTH {
    return i.material.emission;
  }
  if i.material.emission.dot(&i.material.emission) > 0.0 {
    return i.material.emission;
  }

  let brdf_type_rr = (i.material.diffuse + i.material.reflection + i.material.refraction) * rand::random::<f64>();
  let mut brdf_type_rr_prob = i.material.refraction;
  let mut brdf_type = 2;
  if brdf_type_rr < i.material.diffuse {
    brdf_type_rr_prob = i.material.diffuse / (i.material.diffuse + i.material.reflection + i.material.refraction);
    brdf_type = 0;
  } else if brdf_type_rr < i.material.diffuse + i.material.reflection {
    brdf_type_rr_prob = i.material.reflection / (i.material.diffuse + i.material.reflection + i.material.refraction);
    brdf_type = 1;
  }

  // diffuse
  let mut diffuse_color = Vector{x: 0.0, y: 0.0, z: 0.0};
  if brdf_type == 0 {
    // 乱数を生成(半球上で一様にサンプル)
    // let theta = PI * rand::random::<f64>() * 0.5;
    // let phi = 2.0 * PI * rand::random::<f64>();
    // 乱数を生成(cosにしたがって重点的にサンプル)
    let r1: f64 = 2.0 * PI* rand::random::<f64>();
    let r2: f64 = rand::random::<f64>();
    let r2s: f64 = r2.sqrt();

    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = i.normal;
    let u = if w.x.abs() > EPS { Vector{x: 0.0, y: 1.0, z: 0.0} } else { Vector{x: 1.0, y: 0.0, z: 0.0 } }.cross(w);
    let v = w.cross(u);

    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // let d = &(&u.smul(theta.sin() * phi.cos()) + &v.smul(theta.sin() * phi.sin())) + &w.smul(theta.cos());
    let d = &(&u.smul(r1.cos() * r2s) + &v.smul(r1.sin() * r2s)) + &w.smul((1.0 - r2).sqrt());
    // cos項を計算
    let dn = d.dot(&i.normal);
    // 新しいレイを作る
    let new_ray = Ray{d: d, o: i.position};
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると σ / π
    let brdf = i.material.color.smul(1.0 / PI);
    // 確率密度関数はサンプルした乱数が半球上で一様分布なので 1 / 2π
    // let pdf = 1.0 / (2.0 * PI);
    // 確率密度関数はサンプルした乱数がcosに依存した分布なので cosθ / π
    let pdf = dn / PI;
    // レンダリング方程式にしたがって放射輝度を計算する
    // ロシアンルーレットを用いた評価で期待値を満たすために確率で割る (BRDFタイプ用と再帰抑制用)
    // L_e + BRDF * L_i * cosθ / (PDF * RR_prob)
    // return &i.material.emission + &(&brdf * &radiance(new_ray, depth + 1).smul(dn / (pdf * continue_rr_prob * brdf_type_rr_prob)));
    return &i.material.emission + &(&i.material.color * &radiance(new_ray, depth + 1));
  }

  return Vector{x: 0.0, y: 0.0, z: 0.0};
}

const YELLOW_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector{x: 0.0, y: 0.0, z: 0.0}, color: Vector{x: 0.75, y: 0.75, z: 0.25}};
const BLUE_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector{x: 0.0, y: 0.0, z: 0.0}, color: Vector{x: 0.25, y: 0.25, z: 0.75}};
const WHITE_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector{x: 0.0, y: 0.0, z: 0.0}, color: Vector{x: 0.75, y: 0.75, z: 0.75}};
// const REFLECTION_MATERIAL: Material = Material{diffuse: 0.0, reflection: 1.0, refraction: 0.0, emission: Vector{x: 0.0, y: 0.0, z: 0.0}, color: Vector{x: 0.0, y: 0.0, z: 0.0}};
// const REFRACTION_MATERIAL: Material = Material{diffuse: 0.0, reflection: 0.0, refraction: 1.0, emission: Vector{x: 0.0, y: 0.0, z: 0.0}, color: Vector{x: 0.0, y: 0.0, z: 0.0}};
const EMISSION_MATERIAL: Material = Material{diffuse: 1.0, reflection: 0.0, refraction: 0.0, emission: Vector{x: 12.0, y: 12.0, z: 12.0}, color: Vector{x: 1.0, y: 1.0, z: 1.0}};

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
    Triangle::new(Vector{x: -1.5, y: 4.99, z: -3.5}, Vector{x: -1.5, y: 4.99, z: -6.5}, Vector{x: 1.5, y: 4.99, z: -6.5}, EMISSION_MATERIAL),
    Triangle::new(Vector{x: 1.5, y: 4.99, z: -3.5}, Vector{x: -1.5, y: 4.99, z: -3.5}, Vector{x: 1.5, y: 4.99, z: -6.5}, EMISSION_MATERIAL),
  ];

  static ref SPHERE_OBJECTS: [Sphere; 2] = [
    Sphere::new(Vector{x: -2.0, y: 2.0, z: -7.0}, 1.8, WHITE_MATERIAL),
    Sphere::new(Vector{x: 2.0, y: -3.2, z: -3.0}, 1.8, WHITE_MATERIAL),
  ];

  static ref OBJECTS: Objects = Objects::new(&*TRIANGLE_OBJECTS, &*SPHERE_OBJECTS);
}

const DEPTH: usize = 5;
const DEPTH_LIMIT: usize = 5;
const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const PI: f64 = 3.14159265358979323846264338327950288_f64;
const EPS: f64 = 1e-6;
const BG_COLOR: Vector = Vector{x: 0.0, y: 0.0, z: 0.0};

const CROP_OFFSET_BOTTOM: usize = 340;
const CROP_OFFSET_LEFT: usize = 60;
const CROP_HEIGHT: usize = 128;
const CROP_WIDTH: usize = 128;

fn main() {
  let cpu_count = num_cpus::get();
  println!("cpu: {}", cpu_count);
  let pool = ThreadPool::new(cpu_count);
  let (tx, rx): (Sender<(usize, usize, Vector)>, Receiver<(usize, usize, Vector)>) = channel();

  let samples: usize = 500;
  let mut output = box [[Vector{x: 0.0, y: 0.0, z: 0.0}; WIDTH]; HEIGHT];
  let min_rsl: f64 = cmp::min(WIDTH, HEIGHT) as f64;

  for i in CROP_OFFSET_BOTTOM..(CROP_OFFSET_BOTTOM + CROP_HEIGHT) {
    for j in CROP_OFFSET_LEFT..(CROP_OFFSET_LEFT + CROP_WIDTH) {
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
          r = &r + &radiance(ray, 0).smul(1.0 / samples as f64);
          // r = &r + &radiance_old(ray, 0, false).smul(1.0 / samples as f64);
        }
        tx.send((i, j, Vector{x: clamp(r.x), y: clamp(r.y), z: clamp(r.z)})).unwrap();
      });
    }
  }

  let start_time = time::now();
  println!("start: {}", start_time.strftime("%+").unwrap());

  for p in 0..CROP_WIDTH * CROP_HEIGHT - 1 {
    print!("\rraytracing... ({:.0}/{:.0} : {:.0}%)", p, CROP_WIDTH * CROP_HEIGHT, (p as f64) / ((CROP_WIDTH * CROP_HEIGHT) as f64) * 100.0);
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
