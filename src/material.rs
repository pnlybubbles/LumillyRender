extern crate rand;

use vector::*;
use vector3::Vector3;
use ray::Ray;
use sample::Sample;
use constant::*;
use intersection::Intersection;

pub trait Material {
  fn emission(&self) -> Vector3<f64>;
  // 入射ベクトル, 出射ベクトル, 法線ベクトル
  fn brdf(&self, Vector3<f64>, Vector3<f64>, Vector3<f64>) -> Vector3<f64>;
  fn sample(&self, &Ray, &Intersection) -> (Sample<Ray>, f64);
  fn rr_weight(&self) -> f64;
}

pub struct LambertianMaterial {
  pub emission: Vector3<f64>,
  pub albedo: Vector3<f64>,
}

impl LambertianMaterial {
  pub fn orienting_normal(&self, in_: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(in_) > 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }
}

impl Material for LambertianMaterial {
  fn emission(&self) -> Vector3<f64> {
    self.emission
  }

  fn rr_weight(&self) -> f64 {
    // 拡散反射の時は各色の反射率のうち最大のものを使う
    self.albedo.x.max(self.albedo.y).max(self.albedo.z)
  }

  fn brdf(&self, _: Vector3<f64>, _: Vector3<f64>, _: Vector3<f64>) -> Vector3<f64> {
    // BRDFは半球全体に一様に散乱するDiffuse面を考えると ρ / π
    self.albedo / PI
  }

  fn sample(&self, in_ray: &Ray, i: &Intersection) -> (Sample<Ray>, f64) {
    // 拡散反射
    let normal = self.orienting_normal(in_ray.direction, i.normal);
    // 乱数を生成
    // (cosにしたがって重点的にサンプル)
    let r1: f64 = 2.0 * PI * rand::random::<f64>();
    let r2: f64 = rand::random::<f64>();
    let r2s: f64 = r2.sqrt();
    // 反射点での法線方向を基準にした正規直交基底を生成
    let w = normal;
    let u = if w.x.abs() > EPS { Vector3::new(0.0, 1.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) }.cross(w).norm();
    let v = w.cross(u);
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    let d = u * (r1.cos() * r2s) + v * (r1.sin() * r2s) + w * (1.0 - r2).sqrt();
    // 新しいレイを作る
    let new_ray = Ray { direction: d, origin: i.position };
    // cos項
    let cos_term = d.dot(normal);
    // 確率密度関数
    // (cosにしたがって重点的にサンプル) cosθ / π
    let pdf = cos_term / PI;
    (
      Sample {
        value: new_ray,
        pdf: pdf,
      },
      cos_term
    )
  }
}
