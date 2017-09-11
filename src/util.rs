extern crate rand;

use vector3::Vector3;
use vector::*;
use constant::*;

pub trait OrthonormalBasis<T> : VectorFloat<T> {
  fn orthonormal_basis(&self) -> (Self, Self);
}

impl OrthonormalBasis<f64> for Vector3<f64> {
  // 自身(normal)を基準として正規直交基底を生成 (正規化済み前提)
  fn orthonormal_basis(&self) -> (Vector3<f64>, Vector3<f64>) {
    let tangent = if self.x.abs() > EPS {
      Vector3::new(0.0, 1.0, 0.0)
    } else {
      Vector3::new(1.0, 0.0, 0.0)
    }.cross(*self).norm();
    let bionrmal = self.cross(tangent);
    (tangent, bionrmal)
  }
}

pub trait BoundaryResponse<T> : VectorFloat<T> {
  fn reflect(&self, Self) -> Self;
}

impl BoundaryResponse<f64> for Vector3<f64> {
  fn reflect(&self, normal: Vector3<f64>) -> Vector3<f64> {
    *self + normal * ((-*self).dot(normal) * 2.0)
  }
}

pub struct Sampler;

impl Sampler {
  pub fn hemisphere_cos_importance() -> Vector3<f64> {
    // 乱数を生成
    // (cosにしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f64>();
    let r2 = rand::random::<f64>();
    let r2s = r2.sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    Vector3::new(r1.cos() * r2s, r1.sin() * r2s, (1.0 - r2).sqrt())
  }
}

