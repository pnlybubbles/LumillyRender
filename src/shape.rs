use intersection::Intersection;
use ray::Ray;

pub trait Shape {
  fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}
