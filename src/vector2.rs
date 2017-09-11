use std::ops::{Neg, Add, Sub, Mul, Div};
use vector::{Vector, VectorFloat};

#[derive(Debug, Clone, Copy, Default)]
pub struct Vector2<T> {
  pub x: T,
  pub y: T,
}

impl<T> Vector2<T> {
  pub fn new(x: T, y: T) -> Vector2<T> {
    Vector2 { x: x, y: y }
  }

  pub fn to_array(self) -> [T; 2] {
    [self.x, self.y]
  }
}

impl<T> Neg for Vector2<T>
    where T: Copy + Neg<Output = T> {
  type Output = Vector2<T>;

  fn neg(self) -> Vector2<T> {
    Vector2 { x: -self.x, y: -self.y }
  }
}

impl<T> Add for Vector2<T>
    where T: Copy + Add<Output = T> {
  type Output = Vector2<T>;

  fn add(self, others: Vector2<T>) -> Vector2<T> {
    Vector2 { x: self.x + others.x, y: self.y + others.y }
  }
}

impl<T> Sub for Vector2<T>
    where T: Copy + Sub<Output = T> {
  type Output = Vector2<T>;

  fn sub(self, others: Vector2<T>) -> Vector2<T> {
    Vector2 { x: self.x - others.x, y: self.y - others.y }
  }
}

impl<T> Mul<Vector2<T>> for Vector2<T>
    where T: Copy + Mul<Output = T> {
  type Output = Vector2<T>;

  fn mul(self, others: Vector2<T>) -> Vector2<T> {
    Vector2 { x: self.x * others.x, y: self.y * others.y }
  }
}

impl<T> Mul<T> for Vector2<T>
    where T: Copy + Mul<Output = T> {
  type Output = Vector2<T>;

  fn mul(self, others: T) -> Vector2<T> {
    Vector2 { x: self.x * others, y: self.y * others }
  }
}

impl<T> Div<Vector2<T>> for Vector2<T>
    where T: Copy + Div<Output = T> {
  type Output = Vector2<T>;

  fn div(self, others: Vector2<T>) -> Vector2<T> {
    Vector2 { x: self.x / others.x, y: self.y / others.y }
  }
}

impl<T> Div<T> for Vector2<T>
    where T: Copy + Div<Output = T> {
  type Output = Vector2<T>;

  fn div(self, others: T) -> Vector2<T> {
    Vector2 { x: self.x / others, y: self.y / others }
  }
}

impl<T> Vector<T> for Vector2<T>
    where T: Copy + Neg<Output = T> + Add<Output = T> + Sub<Output = T> + Mul<Output = T> {

  fn dot(self, others: Vector2<T>) -> T {
    self.x * others.x + self.y * others.y
  }
}

impl VectorFloat<f64> for Vector2<f64> {
  fn len(self) -> f64{
    self.sqr_len().sqrt()
  }
}

