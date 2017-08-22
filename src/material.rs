use vector3::Vector3;

#[derive(Debug, Default, Clone)]
pub struct Material {
  pub emission: Vector3<f64>,
  pub diffuse: Vector3<f64>,
}
