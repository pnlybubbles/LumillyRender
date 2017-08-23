use vector::{Vector, VectorFloat};
use vector3::Vector3;
use ray::Ray;
use material::Material;
use intersection::Intersection;
use constant::*;

#[derive(Debug, Default, Clone)]
pub struct Sphere {
  pub radius: f64,
  pub position: Vector3<f64>,
  pub material: Material,
}

impl Sphere {
  pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    let co = ray.origin - self.position;
    let cod = co.dot(ray.direction);
    let det = cod * cod - co.dot(co) + self.radius * self.radius;
    if det < 0.0 {
      return None;
    }
    let t1 = -cod - det.sqrt();
    let t2 = -cod + det.sqrt();
    if t1 < EPS && t2 < EPS {
      return None;
    }
    let distance = if t1 > EPS { t1 } else { t2 };
    let position = ray.origin + ray.direction * distance;
    let outer_normal = (position - self.position).norm();
    let normal = if outer_normal.dot(ray.direction) > 0.0 { outer_normal * -1.0 } else { outer_normal };
    Some(Intersection {
      distance: distance,
      position: position,
      normal: normal,
      material: &(self.material),
    })
  }
}
