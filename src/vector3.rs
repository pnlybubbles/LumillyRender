use std::ops::{Neg, Add, Sub, Mul, Div};
use vector::{Vector, VectorFloat};

#[derive(Debug, Clone, Copy, Default)]
struct Vector3<T> {
  x: T,
  y: T,
  z: T,
}

impl<T> Vector3<T>
    where T: Copy {
  pub fn new(x: T, y: T, z: T) -> Vector3<T> {
    Vector3 { x: x, y: y, z: z }
  }

  pub fn to_array(self) -> [T; 3] {
    [self.x, self.y, self.z]
  }

  pub fn cross(self, other: Vector3<T>) -> Vector3<T>
      where T: Sub<Output = T> + Mul<Output = T> {
    Vector3 {
      x: self.y * other.z - self.z * other.y,
      y: self.z * other.x - self.x * other.z,
      z: self.x * other.y - self.y * other.x,
    }
  }
}

impl<T> Neg for Vector3<T>
    where T: Copy + Neg<Output = T> {
  type Output = Vector3<T>;

  fn neg(self) -> Vector3<T> {
    Vector3 { x: -self.x, y: -self.y, z: -self.z }
  }
}

impl<T> Add for Vector3<T>
    where T: Copy + Add<Output = T> {
  type Output = Vector3<T>;

  fn add(self, others: Vector3<T>) -> Vector3<T> {
    Vector3 { x: self.x + others.x, y: self.y + others.y, z: self.z + others.z }
  }
}

impl<T> Sub for Vector3<T>
    where T: Copy + Sub<Output = T> {
  type Output = Vector3<T>;

  fn sub(self, others: Vector3<T>) -> Vector3<T> {
    Vector3 { x: self.x - others.x, y: self.y - others.y, z: self.z - others.z }
  }
}

impl<T> Mul<Vector3<T>> for Vector3<T>
    where T: Copy + Mul<Output = T> {
  type Output = Vector3<T>;

  fn mul(self, others: Vector3<T>) -> Vector3<T> {
    Vector3 { x: self.x * others.x, y: self.y * others.y, z: self.z * others.z }
  }
}

impl<T> Mul<T> for Vector3<T>
    where T: Copy + Mul<Output = T> {
  type Output = Vector3<T>;

  fn mul(self, others: T) -> Vector3<T> {
    Vector3 { x: self.x * others, y: self.y * others, z: self.z * others }
  }
}

impl<T> Div<Vector3<T>> for Vector3<T>
    where T: Copy + Div<Output = T> {
  type Output = Vector3<T>;

  fn div(self, others: Vector3<T>) -> Vector3<T> {
    Vector3 { x: self.x / others.x, y: self.y / others.y, z: self.z / others.z }
  }
}

impl<T> Div<T> for Vector3<T>
    where T: Copy + Div<Output = T> {
  type Output = Vector3<T>;

  fn div(self, others: T) -> Vector3<T> {
    Vector3 { x: self.x / others, y: self.y / others, z: self.z / others }
  }
}

impl<T> Vector<T> for Vector3<T>
    where T: Copy + Neg<Output = T> + Add<Output = T> + Sub<Output = T> + Mul<Output = T> {

  fn dot(self, others: Vector3<T>) -> T {
    self.x * others.x + self.y * others.y + self.z * others.z
  }
}

impl VectorFloat<f64> for Vector3<f64> {
  fn len(self) -> f64{
    self.sqr_len().sqrt()
  }
}

