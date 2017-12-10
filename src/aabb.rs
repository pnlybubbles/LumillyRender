extern crate ordered_float;

use math::vector::Vector3;
use ray::Ray;
use constant::*;
use self::ordered_float::OrderedFloat;

#[derive(Clone)]
pub struct AABB {
  pub min: Vector3,
  pub max: Vector3,
  pub center: Vector3,
}

impl AABB {
  pub fn side(&self) -> Vector3 {
    Vector3::new(
      (self.max.x - self.min.x).abs(),
      (self.max.y - self.min.y).abs(),
      (self.max.z - self.min.z).abs(),
    )
  }

  pub fn merge(list: &Vec<&AABB>) -> AABB {
    let min = Vector3::new(
      *list.iter().map(|v| OrderedFloat(v.min.x)).min().unwrap(),
      *list.iter().map(|v| OrderedFloat(v.min.y)).min().unwrap(),
      *list.iter().map(|v| OrderedFloat(v.min.z)).min().unwrap(),
    );
    let max = Vector3::new(
      *list.iter().map(|v| OrderedFloat(v.max.x)).max().unwrap(),
      *list.iter().map(|v| OrderedFloat(v.max.y)).max().unwrap(),
      *list.iter().map(|v| OrderedFloat(v.max.z)).max().unwrap(),
    );
    AABB {
      min: min,
      max: max,
      center: (min + max) / 2.0,
    }
  }

  pub fn is_intersect(&self, ray: &Ray) -> bool {
    let mut min = -INF;
    let mut max = INF;
    for i in 0..3 {
      let t1 = (self.min[i] - ray.origin[i]) / ray.direction[i];
      let t2 = (self.max[i] - ray.origin[i]) / ray.direction[i];
      min = min.max(t1.min(t2));
      max = max.min(t1.max(t2));
      if min > max { return false }
    }
    true
  }
}
