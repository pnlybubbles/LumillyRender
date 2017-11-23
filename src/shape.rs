use intersection::Intersection;
use ray::Ray;
use aabb::AABB;
use vector::Vector;

pub trait Surface {
  fn emission(&self) -> Vector;
}

pub trait Shape {
  fn intersect(&self, ray: &Ray) -> Option<Intersection>;
  fn aabb(&self) -> AABB;
}
