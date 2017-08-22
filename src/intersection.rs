use vector3::Vector3;
use material::Material;

#[derive(Debug)]
pub struct Intersection<'a> {
  pub position: Vector3<f64>,
  pub distance: f64,
  pub normal: Vector3<f64>,
  pub material: &'a Material,
}

