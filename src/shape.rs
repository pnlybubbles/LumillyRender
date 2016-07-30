use ray::Ray;
use intersection::Intersection;

pub trait Shape {
  fn intersect(self, r: Ray) -> Intersection;
}
