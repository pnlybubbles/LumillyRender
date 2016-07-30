use vector::Vector;

#[derive(Debug, Copy, Clone, Default)]
pub struct Ray {
  pub origin: Vector,
  pub direction: Vector
}
