use std::sync::Arc;
use vector::{Vector, VectorFloat};
use vector3::Vector3;
use ray::Ray;
use material::Material;
use intersection::Intersection;
use constant::*;
use shape::Shape;
use aabb::AABB;

pub struct Sphere {
  pub radius: f64,
  pub position: Vector3<f64>,
  pub material: Arc<Material + Send + Sync>,
}

impl Shape for Sphere {
  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    let co = ray.origin - self.position;
    let cod = co.dot(ray.direction);
    let det = cod * cod - co.sqr_len() + self.radius * self.radius;
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
    Some(Intersection {
      distance: distance,
      position: position,
      normal: outer_normal,
      material: self.material.clone(),
    })
  }

  fn aabb(&self) -> AABB {
    let r = Vector3::new(self.radius, self.radius, self.radius);
    AABB {
      min: self.position - r,
      max: self.position + r,
      center: self.position,
    }
  }
}
