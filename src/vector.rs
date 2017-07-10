use std::ops::{Neg, Add, Sub, Mul, Div};

pub trait Vector<T> : Neg<Output = Self> + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Mul<T, Output = Self>
    where Self: Sized + Copy {
  fn dot(self, Self) -> T;

  fn sqr_len(self) -> T {
    self.dot(self)
  }
}

pub trait VectorFloat<T> : Vector<T> + Div<Output = Self> + Div<T, Output = Self> {
  fn len(self) -> T;

  fn norm(self) -> Self {
    self / self.len()
  }
}
