use std::ops::{Neg, Add, Sub, Mul};
use super::vector::*;

pub struct Matrix4 {
  v: Vec<f32>,
}

impl Matrix4 {
  pub fn translate(v: Vector3) -> Matrix4 {
    Matrix4 {
      v: vec![
        1.0, 0.0, 0.0, v.x,
        0.0, 1.0, 0.0, v.y,
        0.0, 0.0, 1.0, v.z,
        0.0, 0.0, 0.0, 1.0,
      ]
    }
  }

  pub fn scale(v: Vector3) -> Matrix4 {
    Matrix4 {
      v: vec![
        v.x, 0.0, 0.0, 0.0,
        0.0, v.y, 0.0, 0.0,
        0.0, 0.0, v.z, 0.0,
        0.0, 0.0, 0.0, 1.0,
      ]
    }
  }

  pub fn axis_angle(a: Vector3, t: f32) -> Matrix4 {
    // ロドリゲスの回転公式 (Rodrigues' rotation formula)
    let c = t.cos();
    let s = t.sin();
    Matrix4 {
      v: vec![
        c + a.x * a.x * (1.0 - c), a.x * a.y * (1.0 - c) - a.z * s, a.x * a.z * (1.0 - c) + a.y * s, 0.0,
        a.y * a.x * (1.0 - c) + a.z * s, c + a.y * a.y * (1.0 - c), a.y * a.z * (1.0 - c) - a.x * s, 0.0,
        a.z * a.x * (1.0 - c) - a.y * s, a.z * a.y * (1.0 - c) + a.x * s, c + a.z * a.z * (1.0 - c), 0.0,
        0.0, 0.0, 0.0, 1.0,
      ]
    }
  }

  pub fn col(&self, x: usize) -> Vector4 {
    (0..4).map( |i| self.v[x + i * 4]).collect::<Vec<_>>().into()
  }

  pub fn row(&self, y: usize) -> Vector4 {
    (0..4).map( |i| self.v[4 * y + i]).collect::<Vec<_>>().into()
  }
}

impl Neg for Matrix4 {
  type Output = Matrix4;

  fn neg(self) -> Matrix4 {
    Matrix4 {
      v: self.v.iter().map( |v| -v ).collect()
    }
  }
}

impl Add for Matrix4 {
  type Output = Matrix4;

  fn add(self, rhs: Matrix4) -> Matrix4 {
    Matrix4 {
      v: self.v.iter().zip(rhs.v).map( |(v1, v2)| v1 + v2 ).collect()
    }
  }
}

impl Sub for Matrix4 {
  type Output = Matrix4;

  fn sub(self, rhs: Matrix4) -> Matrix4 {
    Matrix4 {
      v: self.v.iter().zip(rhs.v).map( |(v1, v2)| v1 - v2 ).collect()
    }
  }
}

impl Mul<f32> for Matrix4 {
  type Output = Matrix4;

  fn mul(self, rhs: f32) -> Matrix4 {
    Matrix4 {
      v: self.v.iter().map( |v| v * rhs ).collect()
    }
  }
}

impl Mul<Matrix4> for f32 {
  type Output = Matrix4;

  fn mul(self, rhs: Matrix4) -> Matrix4 {
    Matrix4 {
      v: rhs.v.iter().map( |v| self * v ).collect()
    }
  }
}

impl Mul<Vector4> for Matrix4 {
  type Output = Vector4;

  fn mul(self, rhs: Vector4) -> Vector4 {
    (0..4).map( |i| self.row(i).dot(rhs) ).collect::<Vec<_>>().into()
  }
}

impl Mul<Vector3> for Matrix4 {
  type Output = Vector3;

  fn mul(self, rhs: Vector3) -> Vector3 {
    (0..4).map( |i| self.row(i).dot(rhs.into()) ).collect::<Vec<_>>().into()
  }
}

impl Mul for Matrix4 {
  type Output = Matrix4;

  fn mul(self, rhs: Matrix4) -> Matrix4 {
    let r = (0..4).map( |i| self.row(i) ).collect::<Vec<_>>();
    let c = (0..4).map( |i| rhs.col(i) ).collect::<Vec<_>>();
    Matrix4 {
      v: (0..4).flat_map( |y| (0..4).map( |x| r[y].dot(c[x]) ).collect::<Vec<_>>() ).collect()
    }
  }
}
