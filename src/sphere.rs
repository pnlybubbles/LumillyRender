use std::sync::Arc;
use math::vector::*;
use ray::Ray;
use material::material::Material;
use intersection::Intersection;
use constant::*;
use shape::*;
use aabb::AABB;
use sample::Sample;
use util::*;

pub struct Sphere {
  pub radius: f32,
  pub position: Vector3,
  pub material: Arc<dyn Material + Send + Sync>,
  aabb: AABB,
  area: f32,
}

impl Sphere {
  pub fn new(position: Vector3, radius: f32, material: Arc<Material + Send + Sync>) -> Sphere {
    Sphere {
      position: position,
      radius: radius,
      area: 4.0 * PI * radius.powi(2),
      material: material,
      aabb: Self::aabb(position, radius),
    }
  }

  fn aabb(position: Vector3, radius: f32) -> AABB {
    let r = Vector3::new(radius, radius, radius);
    AABB {
      min: position - r,
      max: position + r,
      center: position,
    }
  }
}

impl Shape for Sphere {
  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    let co = ray.origin - self.position;
    let cod = co.dot(ray.direction);
    let det = cod * cod - co.sqr_norm() + self.radius * self.radius;
    if det <= 0.0 {
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

  fn aabb(&self) -> &AABB {
    &self.aabb
  }
}

impl SurfaceShape for Sphere {
  fn material(&self) -> Arc<Material> {
    self.material.clone()
  }

  fn area(&self) -> f32 {
    self.area
  }

  fn sample(&self) -> Sample<Vector3> {
    Sample {
      value: self.position + self.radius * Sampler::sphere_uniform(),
      pdf: 1.0 / self.area,
    }
  }
}
