use vector::Vector;
use ray::Ray;
use material::Material;
use intersection::Intersection;
use shape::Shape;
use constant::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct Sphere {
  pub radius: f64,
  pub position: Vector,
  pub material: Material,
}

impl Sphere {
  pub fn new(position: Vector, radius: f64, material: Material) -> Sphere {
    Sphere {
      radius: radius,
      position: position,
      material: material,
    }
  }
}

impl Shape for Sphere {
  fn intersect(self, ray: Ray) -> Intersection {
    let mut i: Intersection = Default::default();
    let co = ray.origin - self.position;
    let cod = co.dot(ray.direction);
    let det = cod * cod - co.dot(co) + self.radius * self.radius;

    if det < 0.0 {
      i.is_intersect = false;
      return i;
    }
    let t1 = -cod - det.sqrt();
    let t2 = -cod + det.sqrt();
    if t1 < EPS && t2 < EPS {
      i.is_intersect = false;
      return i;
    }
    if t1 > EPS {
      i.distance = t1;
    } else {
      i.distance = t2;
    }
    i.is_intersect = true;
    i.position = ray.origin + ray.direction * i.distance;
    i.normal = (i.position - self.position).norm();
    i.material = self.material;
    return i;
  }
}
