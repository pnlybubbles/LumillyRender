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

