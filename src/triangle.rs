extern crate test;

use std::sync::Arc;
use intersection::Intersection;
use shape::*;
use constant::*;
use ray::Ray;
use material::Material;
use vector::Vector;
use vector::*;
use aabb::AABB;

pub struct Triangle {
  pub p0: Vector,
  pub p1: Vector,
  pub p2: Vector,
  pub normal: Vector,
  pub material: Arc<Material + Send + Sync>,
}

impl Triangle {
  pub fn new(
    p0: Vector,
    p1: Vector,
    p2: Vector,
    material: Arc<Material + Send + Sync>,
  ) -> Triangle {
    Triangle {
      p0: p0,
      p1: p1,
      p2: p2,
      normal: (p1 - p0).cross(p2 - p0).normalize(),
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
    let min = Vector::new(
      self.p0.x.min(self.p1.x).min(self.p2.x),
      self.p0.y.min(self.p1.y).min(self.p2.y),
      self.p0.z.min(self.p1.z).min(self.p2.z),
    );
    let max = Vector::new(
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

impl Surface for Triangle {
  fn emission(&self) -> Vector {
    self.material.emission()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use material::*;

  #[test]
  fn intersect_mt_front() {
    let t = Triangle::new(
      Vector::new(5.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector::zero(), albedo: Vector::zero() }),
    );
    let ray = Ray {
      origin: Vector::new(1.0, 5.0, 1.0),
      direction: Vector::new(0.0, -1.0, 0.0).normalize(),
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
      Vector::new(5.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector::zero(), albedo: Vector::zero() }),
    );
    let ray = Ray {
      origin: Vector::new(1.0, -5.0, 1.0),
      direction: Vector::new(0.0, 1.0, 0.0).normalize(),
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
      Vector::new(5.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector::zero(), albedo: Vector::zero() }),
    );
    let ray = Ray {
      origin: Vector::new(1.0, 5.0, 1.0),
      direction: Vector::new(0.0, -1.0, 0.0).normalize(),
    };
    let i1 = t.intersect_3c(&ray).unwrap();
    let near_ray = Ray {
      origin: i1.position,
      direction: Vector::new(0.0, 1.0, 0.0),
    };
    let i2 = t.intersect_3c(&near_ray);
    assert!(i2.is_none());
  }

  #[test]
  fn intersect_mt_near() {
    let t = Triangle::new(
      Vector::new(5.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 0.0),
      Vector::new(0.0, 0.0, 5.0),
      Arc::new(LambertianMaterial { emission: Vector::zero(), albedo: Vector::zero() }),
    );
    let ray = Ray {
      origin: Vector::new(1.0, 5.0, 1.0),
      direction: Vector::new(0.0, -1.0, 0.0).normalize(),
    };
    let i1 = t.intersect_mt(&ray).unwrap();
    let near_ray = Ray {
      origin: i1.position,
      direction: Vector::new(0.0, 1.0, 0.0),
    };
    let i2 = t.intersect_mt(&near_ray);
    assert!(i2.is_none());
  }
}
