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
    let e1 = self.position1 - self.position0;
    let e2 = self.position2 - self.position0;
    let q = ray.direction.cross(e2);
    let det = q.dot(e1);
    if det < EPS {
      i.is_intersect = false;
      return i;
    }
    let s = ray.origin - self.position0;
    let v = q.dot(s);
    if v < 0.0 || v > det {
      i.is_intersect = false;
      return i;
    }
    let r = s.cross(e1);
    let u = r.dot(ray.direction);
    if u < 0.0 || u + v > det {
      i.is_intersect = false;
      return i;
    }
    let t = r.dot(e2) / det;
    let p = ray.origin + ray.direction * t;
    i.is_intersect = true;
    i.distance = t;
    i.normal = self.normal;
    i.position = p;
    i.material = self.material;
    return i;
  }
}
