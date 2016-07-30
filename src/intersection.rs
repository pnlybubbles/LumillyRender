use vector::Vector;
// use ray::Ray;
use material::Material;

#[derive(Debug, Copy, Clone, Default)]
pub struct Intersection {
  pub is_intersect: bool,
  pub distance: f64,
  pub position: Vector,
  pub normal: Vector,
  pub material: Material,
}
