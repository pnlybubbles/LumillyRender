extern crate ordered_float;

use vector::Vector;
use self::ordered_float::OrderedFloat;

#[derive(Debug)]
pub struct AABB {
  pub min: Vector,
  pub max: Vector,
  pub center: Vector,
}

impl AABB {
  pub fn merge(bb_list: &Vec<&AABB>) -> AABB {
    let min = Vector::new(
      *bb_list.iter().map(|v| OrderedFloat(v.max.x)).max().unwrap(),
      *bb_list.iter().map(|v| OrderedFloat(v.max.y)).max().unwrap(),
      *bb_list.iter().map(|v| OrderedFloat(v.max.z)).max().unwrap(),
    );
    let max = Vector::new(
      *bb_list.iter().map(|v| OrderedFloat(v.min.x)).min().unwrap(),
      *bb_list.iter().map(|v| OrderedFloat(v.min.y)).min().unwrap(),
      *bb_list.iter().map(|v| OrderedFloat(v.min.z)).min().unwrap(),
    );
    AABB {
      min: min,
      max: max,
      center: (min + max) / 2.0,
    }
  }

  pub fn side(&self) -> Vector {
    self.max - self.min
  }
}
