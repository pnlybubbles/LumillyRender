use intersection::Intersection;
use shape::Shape;
use std::sync::Arc;
use constant::*;
use ray::Ray;
use material::Material;
use vector3::Vector3;
use vector::*;
use aabb::AABB;

pub struct Triangle {
  pub p0: Vector3<f64>,
  pub p1: Vector3<f64>,
  pub p2: Vector3<f64>,
  pub normal: Vector3<f64>,
  pub material: Arc<Material + Send + Sync>,
}

impl Triangle {
  pub fn new(p0: Vector3<f64>, p1: Vector3<f64>, p2: Vector3<f64>, material: Arc<Material + Send + Sync>) -> Triangle {
    Triangle {
      p0: p0,
      p1: p1,
      p2: p2,
      normal: (p1 - p0).cross(p2 - p0).norm(),
      material: material,
    }
  }
}

impl Shape for Triangle {
  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    // Möller–Trumbore intersection algorithm
    let e1 = self.p1 - self.p0;
    let e2 = self.p2 - self.p0;
    let q = ray.direction.cross(e2);
    let det = q.dot(e1); // クラメルの分母
    if det.abs() < EPS {
      return None;
    }
    let s = ray.origin - self.p0;
    let v = q.dot(s);
    if v < 0.0 || v > det {
      return None;
    }
    let r = s.cross(e1);
    let u = r.dot(ray.direction);
    if u < 0.0 || u + v > det {
      return None;
    }
    let t = r.dot(e2) / det;
    let p = ray.origin + ray.direction * t;
    Some(Intersection {
      distance: t,
      normal: self.normal,
      position: p,
      material: self.material.clone(),
    })
  }

  fn aabb(&self) -> AABB {
    AABB {
      min: Vector3::new(
        self.p0.x.min(self.p1.x).min(self.p2.x),
        self.p0.y.min(self.p1.y).min(self.p2.y),
        self.p0.z.min(self.p1.z).min(self.p2.z),
      ),
      max: Vector3::new(
        self.p0.x.max(self.p1.x).max(self.p2.x),
        self.p0.y.max(self.p1.y).max(self.p2.y),
        self.p0.z.max(self.p1.z).max(self.p2.z),
      ),
    }
  }
}
