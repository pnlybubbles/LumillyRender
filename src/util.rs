extern crate rand;

use vector::Vector;
use vector::*;
use constant::*;

pub trait OrthonormalBasis: Sized {
  fn orthonormal_basis(&self) -> (Self, Self);
}

impl OrthonormalBasis for Vector {
  // 自身(normal)を基準として正規直交基底を生成 (正規化済み前提)
  fn orthonormal_basis(&self) -> (Vector, Vector) {
    let tangent = if self.x.abs() > EPS {
      Vector::new(0.0, 1.0, 0.0)
    } else {
      Vector::new(1.0, 0.0, 0.0)
    }.cross(*self)
      .normalize();
    let bionrmal = self.cross(tangent);
    (tangent, bionrmal)
  }
}

pub trait BoundaryResponse {
  fn reflect(&self, Self) -> Self;
}

impl BoundaryResponse for Vector {
  fn reflect(&self, normal: Vector) -> Vector {
    *self + normal * ((-*self).dot(normal) * 2.0)
  }
}

pub struct Sampler;

impl Sampler {
  pub fn hemisphere_cos_importance() -> Vector {
    // 乱数を生成
    // (cosにしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let r2s = r2.sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    Vector::new(r1.cos() * r2s, r1.sin() * r2s, (1.0 - r2).sqrt())
  }

  pub fn hemisphere_uniform() -> Vector {
    // 乱数を生成
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let r2s = (1.0 - r2 * r2).sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (一様サンプル)
    Vector::new(r1.cos() * r2s, r1.sin() * r2s, r2.sqrt())
  }

  pub fn sphere_uniform() -> Vector {
    // 乱数を生成
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>() * 2.0 - 1.0;
    let r2s = (1.0 - r2 * r2).sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (一様サンプル)
    Vector::new(r1.cos() * r2s, r1.sin() * r2s, r2)
  }
}
