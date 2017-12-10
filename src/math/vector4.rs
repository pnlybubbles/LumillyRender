use std::fmt;
use std::ops::{Neg, Add, Sub, Mul, Div, Index};
use super::traits::{Dot, Zero};
use super::vector::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Vector4 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

impl Vector4 {
  pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
    Vector4 { x: x, y: y, z: z, w: w }
  }
}

impl Zero for Vector4 {
  fn zero() -> Vector4 {
    Vector4::new(0.0, 0.0, 0.0, 0.0)
  }
}

impl<'a> From<&'a [f32]> for Vector4 {
  fn from(v: &[f32]) -> Vector4 {
    if v.len() > 4 { panic!("Slice must have length more than 4.") }
    Vector4 { x: v[0], y: v[1], z: v[2], w: v[3] }
  }
}

impl From<Vec<f32>> for Vector4 {
  fn from(v: Vec<f32>) -> Vector4 {
    if v.len() > 4 { panic!("Vec must have length more than 4.") }
    Vector4 { x: v[0], y: v[1], z: v[2], w: v[3] }
  }
}

impl From<Vector3> for Vector4 {
  fn from(v: Vector3) -> Vector4 {
    Vector4 { x: v.x, y: v.y, z: v.z, w: 1.0 }
  }
}

impl Into<[f32; 4]> for Vector4 {
  fn into(self) -> [f32; 4] {
    [self.x, self.y, self.z, self.w]
  }
}

impl fmt::Display for Vector4 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
  }
}

impl Index<usize> for Vector4 {
  type Output = f32;

  fn index(&self, i: usize) -> &f32 {
    match i {
      0 => &self.x,
      1 => &self.y,
      2 => &self.z,
      3 => &self.w,
      _ => panic!("Vector4 index out of bounds."),
    }
  }
}

impl Dot for Vector4 {
  fn dot(self, rhs: Vector4) -> f32 {
    self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
  }
}

impl Neg for Vector4 {
  type Output = Vector4;

  fn neg(self) -> Vector4 {
    Vector4::new(-self.x, -self.y, -self.z, -self.w)
  }
}

impl Add for Vector4 {
  type Output = Vector4;

  fn add(self, rhs: Vector4) -> Vector4 {
    Vector4::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
  }
}

impl Sub for Vector4 {
  type Output = Vector4;

  fn sub(self, rhs: Vector4) -> Vector4 {
    Vector4::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w)
  }
}

impl Mul<f32> for Vector4 {
  type Output = Vector4;

  fn mul(self, rhs: f32) -> Vector4 {
    Vector4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
  }
}

impl Mul<Vector4> for f32 {
  type Output = Vector4;

  fn mul(self, rhs: Vector4) -> Vector4 {
    Vector4::new(self * rhs.x, self * rhs.y, self * rhs.z, self * rhs.w)
  }
}

impl Mul for Vector4 {
  type Output = Vector4;

  fn mul(self, rhs: Vector4) -> Vector4 {
    Vector4::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z, self.w * rhs.w)
  }
}

impl Div<f32> for Vector4 {
  type Output = Vector4;

  fn div(self, rhs: f32) -> Vector4 {
    Vector4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
  }
}

impl Div<Vector4> for f32 {
  type Output = Vector4;

  fn div(self, rhs: Vector4) -> Vector4 {
    Vector4::new(self / rhs.x, self / rhs.y, self / rhs.z, self / rhs.w)
  }
}

impl Div for Vector4 {
  type Output = Vector4;

  fn div(self, rhs: Vector4) -> Vector4 {
    Vector4::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z, self.w / rhs.w)
  }
}
