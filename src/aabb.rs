use vector3::Vector3;

#[derive(Debug)]
pub struct AABB {
  pub min: Vector3<f64>,
  pub max: Vector3<f64>,
}
