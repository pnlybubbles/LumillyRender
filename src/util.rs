extern crate rand;

use math::vector::*;
use constant::*;

pub trait OrthonormalBasis: Sized {
  fn orthonormal_basis(&self) -> (Self, Self);
}

impl OrthonormalBasis for Vector3 {
  // 自身(normal)を基準として正規直交基底を生成 (正規化済み前提)
  fn orthonormal_basis(&self) -> (Vector3, Vector3) {
    let tangent = if self.x.abs() > EPS {
      Vector3::new(0.0, 1.0, 0.0)
    } else {
      Vector3::new(1.0, 0.0, 0.0)
    }.cross(*self)
      .normalize();
    let bionrmal = self.cross(tangent);
    (tangent, bionrmal)
  }
}

pub trait BoundaryResponse where Self: Sized {
  fn reflect(&self, Self) -> Self;
  fn refract(&self, Self, f32) -> Option<Self>;
}

impl BoundaryResponse for Vector3 {
  fn reflect(&self, normal: Vector3) -> Vector3 {
    -*self + normal * ((*self).dot(normal) * 2.0)
  }

  fn refract(&self, normal: Vector3, from_per_to_ior: f32) -> Option<Vector3> {
    let dn = self.dot(normal);
    let cos2theta = 1.0 - from_per_to_ior.powi(2) * (1.0 - dn.powi(2));
    if cos2theta > 0.0 {
      Some(-*self * from_per_to_ior - normal * (from_per_to_ior * -dn + cos2theta.sqrt()))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn reflect_test() {
    let v = Vector3::new(1.0, 0.0, 1.0).normalize();
    let n = Vector3::new(0.0, 0.0, 1.0);
    let r = v.reflect(n);
    let expoect = Vector3::new(-1.0, 0.0, 1.0).normalize();
    assert!((r - expoect).norm() < EPS);
  }

  #[test]
  fn refract_total_reflection_test() {
    let v = Vector3::new(1.0, 0.0, 0.1).normalize();
    let n = Vector3::new(0.0, 0.0, 1.0);
    let r = v.refract(n, 1.5 / 1.0);
    assert!(r.is_none());
  }

  #[test]
  fn refract_test() {
    // 屈折率
    let n1 = 1.0;
    let n2 = 1.5;
    // 30 deg
    let t1 = 30.0 / 180.0 * PI;
    let sin_t1 = t1.sin();
    let v = Vector3::new(t1.tan(), 0.0, 1.0).normalize();
    let n = Vector3::new(0.0, 0.0, 1.0);
    let r = v.refract(n, 1.5 / 1.0).unwrap();
    let sin_t2 = r.cross(-n).norm();
    // sin(t1) / sin(t2) == n1 / n2
    assert!((sin_t1 / sin_t2 - n1 / n2).abs() < EPS, "{} {} {} {}", sin_t1, sin_t2, n1, n2);
    assert!((r.norm() - 1.0).abs() < EPS, "{} {}", r, r.norm());
  }
}

pub struct Sampler;

impl Sampler {
  pub fn hemisphere_cos_importance() -> Vector3 {
    // 乱数を生成
    // (cosにしたがって重点的にサンプル)
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let r2s = r2.sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (cosにしたがって重点的にサンプル)
    Vector3::new(r1.cos() * r2s, r1.sin() * r2s, (1.0 - r2).sqrt())
  }

  pub fn hemisphere_uniform() -> Vector3 {
    // 乱数を生成
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>();
    let r2s = (1.0 - r2 * r2).sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (一様サンプル)
    Vector3::new(r1.cos() * r2s, r1.sin() * r2s, r2.sqrt())
  }

  pub fn sphere_uniform() -> Vector3 {
    // 乱数を生成
    let r1 = 2.0 * PI * rand::random::<f32>();
    let r2 = rand::random::<f32>() * 2.0 - 1.0;
    let r2s = (1.0 - r2 * r2).sqrt();
    // 球面極座標を用いて反射点から単位半球面上のある一点へのベクトルを生成
    // (一様サンプル)
    Vector3::new(r1.cos() * r2s, r1.sin() * r2s, r2)
  }
}
