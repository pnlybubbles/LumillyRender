use std::sync::Arc;
use vector3::Vector3;
use material::Material;

pub struct Intersection {
  pub position: Vector3<f64>,
  pub distance: f64,
  pub normal: Vector3<f64>,
  pub material: Arc<Material + Send + Sync>,
}

