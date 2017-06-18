use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Copy, Clone, Default)]
pub struct Vector {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Vector {
  pub fn new(x: f64, y: f64, z: f64) -> Vector {
    return Vector{x: x, y: y, z: z};
  }

  pub fn dot(self, other: Vector) -> f64 {
    return self.x * other.x + self.y * other.y + self.z * other.z;
  }

  pub fn cross(self, other: Vector) -> Vector {
    return Vector::new(self.y * other.z - self.z * other.y, self.z * other.x - self.x * other.z, self.x * other.y - self.y * other.x);
  }

  pub fn sqr_len(self) -> f64 {
    return self.dot(self);
  }

  pub fn len(self) -> f64 {
    return self.sqr_len().sqrt();
  }

  pub fn norm(self) -> Vector {
    return self * (1.0 / self.len());
  }

  pub fn to_array(self) -> [f64; 3] {
    return [self.x, self.y, self.z];
  }
}

impl Add for Vector {
  type Output = Vector;

  fn add(self, other: Vector) -> Vector {
    return Vector { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z };
  }
}

impl Sub for Vector {
  type Output = Vector;

  fn sub(self, other: Vector) -> Vector {
    return Vector::new(self.x - other.x, self.y - other.y, self.z - other.z);
  }
}

impl Mul<Vector> for Vector {
  type Output = Vector;

  fn mul(self, other: Vector) -> Vector {
    return Vector::new(self.x * other.x, self.y * other.y, self.z * other.z)
  }
}

impl Mul<f64> for Vector {
  type Output = Vector;

  fn mul(self, other: f64) -> Vector {
    return Vector::new(self.x * other, self.y * other, self.z * other)
  }
}

impl Div<Vector> for Vector {
  type Output = Vector;

  fn div(self, other: Vector) -> Vector {
    return Vector::new(self.x / other.x, self.y / other.y, self.z / other.z);
  }
}

impl Div<f64> for Vector {
  type Output = Vector;

  fn div(self, other: f64) -> Vector {
    return Vector::new(self.x / other, self.y / other, self.z / other);
  }
}
