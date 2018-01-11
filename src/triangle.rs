extern crate test;
extern crate rand;

use std::sync::Arc;
use intersection::Intersection;
use shape::*;
use constant::*;
use ray::Ray;
use material::material::Material;
use math::vector::*;
use aabb::AABB;
use sample::Sample;

pub struct Triangle {
  pub p0: Vector3,
  pub p1: Vector3,
  pub p2: Vector3,
  pub normal: Vector3,
  pub area: f32,
  pub material: Arc<Material + Send + Sync>,
}

impl Triangle {
  pub fn new(
    p0: Vector3,
    p1: Vector3,
    p2: Vector3,
    material: Arc<Material + Send + Sync>,
  ) -> Triangle {
    Triangle {
      p0: p0,
      p1: p1,
      p2: p2,
      normal: (p1 - p0).cross(p2 - p0).normalize(),
      area: (p1 - p0).cross(p2 - p0).norm() * 0.5,
      material: material,
    }
  }

  fn intersect_3c(&self, ray: &Ray) -> Option<Intersection> {
    let dn = ray.direction.dot(self.normal);
    let t = (self.p0 - ray.origin).dot(self.normal) / dn;
    if t < EPS {
      return None;
    }
    let p = ray.origin + ray.direction * t;
    let c0 = (self.p1 - self.p0).cross(p - self.p0);
    if c0.dot(self.normal) < 0.0 {
      return None;
    }
    let c1 = (self.p2 - self.p1).cross(p - self.p1);
    if c1.dot(self.normal) < 0.0 {
      return None;
    }
    let c2 = (self.p0 - self.p2).cross(p - self.p2);
    if c2.dot(self.normal) < 0.0 {
      return None;
    }
    Some(Intersection {
      distance: t,
      normal: self.normal,
      position: p,
      material: self.material.clone(),
    })
  }

  fn intersect_mt(&self, ray: &Ray) -> Option<Intersection> {
    // Möller–Trumbore intersection algorithm
    let e1 = self.p1 - self.p0;
    let e2 = self.p2 - self.p0;
    let pv = ray.direction.cross(e2);
    let det = e1.dot(pv); // クラメルの分母
    if det.abs() < EPS {
      return None;
    }
    let invdet = 1.0 / det;
    let tv = ray.origin - self.p0;
    let u = tv.dot(pv) * invdet;
    if u < 0.0 || u > 1.0 {
      return None;
    }
    let qv = tv.cross(e1);
    let v = ray.direction.dot(qv) * invdet;
    if v < 0.0 || u + v > 1.0 {
      return None;
    }
    let t = e2.dot(qv) * invdet;
    if t < EPS {
      return None;
    }
    let p = ray.origin + ray.direction * t;
    Some(Intersection {
      distance: t,
      normal: self.normal,
      position: p,
      material: self.material.clone(),
    })
  }
}

impl Shape for Triangle {
  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    self.intersect_mt(&ray)
  }

  fn aabb(&self) -> AABB {
    let min = Vector3::new(
      self.p0.x.min(self.p1.x).min(self.p2.x),
      self.p0.y.min(self.p1.y).min(self.p2.y),
      self.p0.z.min(self.p1.z).min(self.p2.z),
    );
    let max = Vector3::new(
      self.p0.x.max(self.p1.x).max(self.p2.x),
      self.p0.y.max(self.p1.y).max(self.p2.y),
      self.p0.z.max(self.p1.z).max(self.p2.z),
    );
    AABB {
      min: min,
      max: max,
      center: (max + min) / 2.0,
    }
  }
}

impl SurfaceShape for Triangle {
  fn material(&self) -> Arc<Material> {
    self.material.clone()
  }

  fn area(&self) -> f32 {
    self.area
  }

  fn sample(&self) -> Sample<Vector3> {
    let u = rand::random::<f32>();
    let v = rand::random::<f32>();
    let min = u.min(v);
    let max = u.max(v);
    Sample {
      value: self.p0 * min + self.p1 * (1.0 - max) + self.p2 * (max - min),
      pdf: 1.0 / self.area,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use material::*;

  #[test]
  fn intersect_mt_front() {
    let t = Triangle::new(
      Vector3::new(5.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector3::zero(), albedo: Vector3::zero() }),
    );
    let ray = Ray {
      origin: Vector3::new(1.0, 5.0, 1.0),
      direction: Vector3::new(0.0, -1.0, 0.0).normalize(),
    };
    let i1 = t.intersect_3c(&ray).unwrap();
    let i2 = t.intersect_mt(&ray).unwrap();
    assert!((i1.normal - i2.normal).norm() < 1e-3);
    assert!((i1.position - i2.position).norm() < 1e-3);
    assert!((i1.distance - i2.distance).abs() < 1e-3);
  }

  #[test]
  fn intersect_mt_back() {
    let t = Triangle::new(
      Vector3::new(5.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector3::zero(), albedo: Vector3::zero() }),
    );
    let ray = Ray {
      origin: Vector3::new(1.0, -5.0, 1.0),
      direction: Vector3::new(0.0, 1.0, 0.0).normalize(),
    };
    let i1 = t.intersect_3c(&ray).unwrap();
    let i2 = t.intersect_mt(&ray).unwrap();
    assert!((i1.normal - i2.normal).norm() < 1e-3);
    assert!((i1.position - i2.position).norm() < 1e-3);
    assert!((i1.distance - i2.distance).abs() < 1e-3);
  }

  #[test]
  fn intersect_3c_near() {
    let t = Triangle::new(
      Vector3::new(5.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector3::zero(), albedo: Vector3::zero() }),
    );
    let ray = Ray {
      origin: Vector3::new(1.0, 5.0, 1.0),
      direction: Vector3::new(0.0, -1.0, 0.0).normalize(),
    };
    let i1 = t.intersect_3c(&ray).unwrap();
    let near_ray = Ray {
      origin: i1.position,
      direction: Vector3::new(0.0, 1.0, 0.0),
    };
    let i2 = t.intersect_3c(&near_ray);
    assert!(i2.is_none());
  }

  #[test]
  fn intersect_mt_near() {
    let t = Triangle::new(
      Vector3::new(5.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 0.0),
      Vector3::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector3::zero(), albedo: Vector3::zero() }),
    );
    let ray = Ray {
      origin: Vector3::new(1.0, 5.0, 1.0),
      direction: Vector3::new(0.0, -1.0, 0.0).normalize(),
    };
    let i1 = t.intersect_mt(&ray).unwrap();
    let near_ray = Ray {
      origin: i1.position,
      direction: Vector3::new(0.0, 1.0, 0.0),
    };
    let i2 = t.intersect_mt(&near_ray);
    assert!(i2.is_none());
  }
}
