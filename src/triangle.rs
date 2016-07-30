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
  checker: bool,
  basis: Vector,
  vertical0: Vector,
  vertical1: Vector,
}

impl Triangle {
  pub fn new(position0: Vector, position1: Vector, position2: Vector, material: Material) -> Triangle {
    let v0 = position1 - position2;
    let v1 = position2 - position0;
    let v2 = position0 - position1;
    let mut checker = true;
    let mut basis: Vector = Default::default();
    let mut vertical0: Vector = Default::default();
    let mut vertical1: Vector = Default::default();
    // if material.emission.sqr_len() == 0.0 {
    //   if v0.dot(v1).abs() < EPS {
    //     basis = position2;
    //     vertical0 = v0.norm();
    //     vertical1 = (v1 * (-1.0)).norm();
    //   } else if v1.dot(v2).abs() < EPS {
    //     basis = position0;
    //     vertical0 = v1.norm();
    //     vertical1 = (v2 * (-1.0)).norm();
    //   } else if v2.dot(v0).abs() < EPS {
    //     basis = position1;
    //     vertical0 = v2.norm();
    //     vertical1 = (v0 * (-1.0)).norm();
    //   } else {
    //     checker = false;
    //   }
    // } else {
      checker = false;
    // }
    Triangle {
      position0: position0,
      position1: position1,
      position2: position2,
      normal: (position1 - position0).cross(position2 - position0).norm(),
      material: material,
      checker: checker,
      basis: basis,
      vertical0: vertical0,
      vertical1: vertical1,
    }
  }
}

impl Shape for Triangle {
  fn intersect(self, ray: Ray) -> Intersection {
    let mut i: Intersection = Default::default();
    let dn = ray.direction.dot(self.normal);
    if dn >= 0.0 {
      i.is_intersect = false;
      return i;
    }
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
    if self.checker {
      let comp0 = (p - self.basis).dot(self.vertical0);
      let comp1 = (p - self.basis).dot(self.vertical1);
        i.material = self.material;
      if (((comp0 / 1.0).floor() + (comp1 / 1.0).floor()) as i32) % 2 == 0 {
        i.material.color = i.material.color * 0.6;
      }
    } else {
      i.material = self.material;
    }
    return i;
  }
}
