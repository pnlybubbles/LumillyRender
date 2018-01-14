extern crate rand;

use super::traits::Material;
use math::vector::*;
use sample::Sample;
use constant::*;
use util::{BoundaryResponse, OrthonormalBasis};

pub struct PhongMaterial {
  // 反射率
  pub reflectance: Vector3,
  // ラフネス
  pub roughness: f32,
}

impl Material for PhongMaterial {
  fn orienting_normal(&self, out_: Vector3, normal: Vector3) -> Vector3 {
    // 物体の内外を考慮した法線方向から拡散反射面としての法線方向を求める
    if normal.dot(out_) < 0.0 {
      normal * -1.0
    } else {
      normal
    }
  }

  fn emission(&self) -> Vector3 {
    Vector3::zero()
  }

  fn weight(&self) -> f32 {
    // 反射率のうち最大のものをつかう
    self.reflectance.x.max(self.reflectance.y).max(
      self.reflectance.z,
    )
  }

  fn brdf(&self, out_: Vector3, in_: Vector3, n: Vector3) -> Vector3 {
    if in_.dot(n) <= 0.0 { return Vector3::zero() }
    let r = out_.reflect(n);
    let cos = r.dot(in_);
    let a = self.roughness;
    // modified phong
    self.reflectance * ((a + 2.0) / (2.0 * PI) * cos.powf(a))
  }

  fn sample(&self, out_: Vector3, n: Vector3) -> Sample<Vector3> {
    let a = self.roughness;
    let r = out_.reflect(n);
    // 鏡面反射方向を基準にした正規直交基底を生成
    let w = r;
    let (u, v) = w.orthonormal_basis();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (brdfの分布にしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let t = r2.powf(1.0 / (a + 2.0));
    let ts = (1.0 - t * t).sqrt();
    let in_ = u * r1.cos() * ts + v * r1.sin() * ts + w * t;
    let cos = r.dot(in_);
    // 確率密度関数
    let pdf = (a + 2.0) / (2.0 * PI) * cos.powf(a);
    Sample {
      value: in_,
      pdf: pdf,
    }
  }
}
