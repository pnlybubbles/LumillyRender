use std::sync::Arc;
use vector::Vector;
use material::Material;

pub struct Intersection {
  pub position: Vector,
  pub distance: f64,
  pub normal: Vector,
  pub material: Arc<Material + Send + Sync>,
}
