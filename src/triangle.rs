use vector::Vector;
use ray::Ray;
use material::Material;
use intersection::Intersection;
use shape::Shape;
use constant::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct Triangle {
  pub position0: Vector,
  pub position1: Vector,
  pub position2: Vector,
  pub normal: Vector,
  pub material: Material,
}

impl Triangle {
  pub fn new(position0: Vector, position1: Vector, position2: Vector, material: Material) -> Triangle {
    let v0 = position1 - position2;
    let v1 = position2 - position0;
    let v2 = position0 - position1;
    Triangle {
      position0: position0,
      position1: position1,
      position2: position2,
      normal: (position1 - position0).cross(position2 - position0).norm(),
      material: material,
    }
  }
}

impl Shape for Triangle {
  fn intersect(self, ray: Ray) -> Intersection {
    let mut i: Intersection = Default::default();
    let dn = ray.direction.dot(self.normal);
    // if dn >= 0.0 {
    //   i.is_intersect = false;
    //   return i;
    // }
    let t = (self.position0 - ray.origin).dot(self.normal) / dn;
    if t < EPS {
      i.is_intersect = false;
      return i;
    }
    let p = ray.origin + ray.direction * t;
    let c0 = (self.position1 - self.position0).cross(p - self.position0);
    if c0.dot(self.normal) < 0.0 {
      i.is_intersect = false;
      return i;
    }
    let c1 = (self.position2 - self.position1).cross(p - self.position1);
    if c1.dot(self.normal) < 0.0 {
      i.is_intersect = false;
      return i;
    }
    let c2 = (self.position0 - self.position2).cross(p - self.position2);
    if c2.dot(self.normal) < 0.0 {
      i.is_intersect = false;
      return i;
    }
    i.is_intersect = true;
    i.distance = t;
    i.normal = self.normal;
    i.position = p;
    i.material = self.material;
    return i;
  }
}
