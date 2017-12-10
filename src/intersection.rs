use std::sync::Arc;
use math::vector::Vector3;
use material::Material;

pub struct Intersection {
  pub position: Vector3,
  pub distance: f32,
  pub normal: Vector3,
  pub material: Arc<Material + Send + Sync>,
}
