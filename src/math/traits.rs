use std::ops::{Div};

pub trait Dot {
  fn dot(self, Self) -> f32;
}

pub trait Cross {
  fn cross(self, Self) -> Self;
}

pub trait Norm {
  fn norm(self) -> f32;
  fn sqr_norm(self) -> f32;
}

pub trait Normalize {
  fn normalize(self) -> Self;
}

pub trait Zero {
  fn zero() -> Self;
}

impl<T> Norm for T
  where T: Copy + Dot
{
  fn norm(self) -> f32 {
    self.sqr_norm().sqrt()
  }

  fn sqr_norm(self) -> f32 {
    self.dot(self)
  }
}

impl<T> Normalize for T
  where T: Copy + Norm + Div<f32, Output = Self>
{
  fn normalize(self) -> T {
    self / self.norm()
  }
}
