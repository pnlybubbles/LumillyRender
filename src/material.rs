use vector::Vector;

#[derive(Debug, Copy, Clone, Default)]
pub struct Material {
  pub color: Vector,
  pub emission: Vector,
  pub diffuse: f64,
  pub reflection: f64,
  pub refraction: f64,
}
