use std::sync::Arc;
use intersection::Intersection;
use ray::Ray;
use aabb::AABB;
use material::Material;
use vector::Vector;
use sample::Sample;

pub trait SurfaceShape: Shape {
  fn material(&self) -> Arc<Material>;
  fn area(&self) -> f32;
  fn sample(&self) -> Sample<Vector>;
}

pub trait Shape {
  fn intersect(&self, ray: &Ray) -> Option<Intersection>;
  fn aabb(&self) -> AABB;
}
