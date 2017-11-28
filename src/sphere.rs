use std::sync::Arc;
use vector::*;
use ray::Ray;
use material::Material;
use intersection::Intersection;
use constant::*;
use shape::*;
use aabb::AABB;
use sample::Sample;
use util::*;

pub struct Sphere {
  pub radius: f64,
  pub position: Vector,
  pub material: Arc<Material + Send + Sync>,
  area: f64,
}

impl Sphere {
  pub fn new(position: Vector, radius: f64, material: Arc<Material + Send + Sync>) -> Sphere {
    Sphere {
      position: position,
      radius: radius,
      area: 4.0 * PI * radius.powi(2),
      material: material,
    }
  }
}

impl Shape for Sphere {
  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    let co = ray.origin - self.position;
    let cod = co.dot(ray.direction);
    let det = cod * cod - co.sqr_norm() + self.radius * self.radius;
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
    let outer_normal = (position - self.position).normalize();
    Some(Intersection {
      distance: distance,
      position: position,
      normal: outer_normal,
      material: self.material.clone(),
    })
  }

  fn aabb(&self) -> AABB {
    let r = Vector::new(self.radius, self.radius, self.radius);
    AABB {
      min: self.position - r,
      max: self.position + r,
      center: self.position,
    }
  }
}

impl SurfaceShape for Sphere {
  fn material(&self) -> Arc<Material> {
    self.material.clone()
  }

  fn area(&self) -> f64 {
    self.area
  }

  fn sample(&self) -> Sample<Vector> {
    Sample {
      value: self.position + self.radius * Sampler::sphere_uniform(),
      pdf: 1.0 / self.area,
    }
  }
}
