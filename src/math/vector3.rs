use std::fmt;
use std::ops::{Neg, Add, Sub, Mul, Div, Index};
use super::traits::{Dot, Cross, Zero};
use super::vector::Vector4;

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl Vector3 {
  pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x: x, y: y, z: z }
  }
}

impl Zero for Vector3 {
  fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
  }
}

impl<'a> From<&'a [f32]> for Vector3 {
  fn from(v: &[f32]) -> Vector3 {
    if v.len() > 4 { panic!("Slice must have length more than 3.") }
    Vector3 { x: v[0], y: v[1], z: v[2] }
  }
}

impl From<Vec<f32>> for Vector3 {
  fn from(v: Vec<f32>) -> Vector3 {
    if v.len() > 4 { panic!("Vec must have length more than 3.") }
    Vector3 { x: v[0], y: v[1], z: v[2] }
  }
}

impl Into<[f32; 3]> for Vector3 {
  fn into(self) -> [f32; 3] {
    [self.x, self.y, self.z]
  }
}

impl From<Vector4> for Vector3 {
  fn from(v: Vector4) -> Vector3 {
    Vector3 { x: v.x, y: v.y, z: v.z }
  }
}

impl fmt::Display for Vector3 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {}, {})", self.x, self.y, self.z)
  }
}

impl Index<usize> for Vector3 {
  type Output = f32;

  fn index(&self, i: usize) -> &f32 {
    match i {
      0 => &self.x,
      1 => &self.y,
      2 => &self.z,
      _ => panic!("Vector3 index out of bounds."),
    }
  }
}

impl Dot for Vector3 {
  fn dot(self, rhs: Vector3) -> f32 {
    self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
  }
}

impl Cross for Vector3 {
  fn cross(self, rhs: Vector3) -> Vector3 {
    Vector3::new(
      self.y * rhs.z - self.z * rhs.y,
      self.z * rhs.x - self.x * rhs.z,
      self.x * rhs.y - self.y * rhs.x,
    )
  }
}

impl Neg for Vector3 {
  type Output = Vector3;

  fn neg(self) -> Vector3 {
    Vector3::new(-self.x, -self.y, -self.z)
  }
}

impl Add for Vector3 {
  type Output = Vector3;

  fn add(self, rhs: Vector3) -> Vector3 {
    Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
  }
}

impl Sub for Vector3 {
  type Output = Vector3;

  fn sub(self, rhs: Vector3) -> Vector3 {
    Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
  }
}

impl Mul<f32> for Vector3 {
  type Output = Vector3;

  fn mul(self, rhs: f32) -> Vector3 {
    Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
  }
}

impl Mul<Vector3> for f32 {
  type Output = Vector3;

  fn mul(self, rhs: Vector3) -> Vector3 {
    Vector3::new(self * rhs.x, self * rhs.y, self * rhs.z)
  }
}

impl Mul for Vector3 {
  type Output = Vector3;

  fn mul(self, rhs: Vector3) -> Vector3 {
    Vector3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
  }
}

impl Div<f32> for Vector3 {
  type Output = Vector3;

  fn div(self, rhs: f32) -> Vector3 {
    Vector3::new(self.x / rhs, self.y / rhs, self.z / rhs)
  }
}

impl Div<Vector3> for f32 {
  type Output = Vector3;

  fn div(self, rhs: Vector3) -> Vector3 {
    Vector3::new(self / rhs.x, self / rhs.y, self / rhs.z)
  }
}

impl Div for Vector3 {
  type Output = Vector3;

  fn div(self, rhs: Vector3) -> Vector3 {
    Vector3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
  }
}
