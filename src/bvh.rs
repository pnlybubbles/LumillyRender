extern crate ordered_float;

use std::sync::Arc;
use aabb::AABB;
use intersection::Intersection;
use ray::Ray;
use vector::Vector;
use self::ordered_float::OrderedFloat;

pub struct BVH {
  node: BVHNode,
}

impl BVH {
  pub fn new(bb_array: Vec<AABB>) -> BVH {
    let bb_list: Vec<(usize, &AABB)> = bb_array.iter().enumerate().collect();
    let mut bb_sorted = [bb_list.clone(), bb_list.clone(), bb_list.clone()];
    bb_sorted[0].sort_unstable_by_key( |&(_, bb)| OrderedFloat(bb.center.x) );
    bb_sorted[1].sort_unstable_by_key( |&(_, bb)| OrderedFloat(bb.center.y) );
    bb_sorted[2].sort_unstable_by_key( |&(_, bb)| OrderedFloat(bb.center.z) );
    let node = Self::construct(&bb_sorted);
    BVH {
      node: node,
    }
  }

  fn construct(bb_sorted: &[Vec<(usize, &AABB)>; 3]) -> BVHNode {
    let bb_list_no_index: Vec<&AABB> = bb_sorted[0].iter().map( |&(_, bb)| bb ).collect();
    let bb_wrap = AABB::merge(&bb_list_no_index);
    let side = bb_wrap.side().to_array();
    let (axis, _) = side.iter().enumerate().max_by_key( |&(_, &s)| OrderedFloat(s) ).unwrap();
    let max = bb_sorted[axis];
    let pivot = max.len() / 2;
    let partitioned_max = (&max[0..pivot].to_vec(), &max[pivot..].to_vec());
    let fst = partitioned_max.0.iter().map( |&(i, _)| i );
    let partitioned = bb_sorted.iter().enumerate().map( |(a, v)| {
      if a == axis {
        partitioned_max
      } else {

      }
    } ).unzip();
    BVHNode {
      aabb: bb_wrap,
      children: None,
    }
  }

  // fn partition<'a>(bb_sorted: &'a [Vec<(usize, &'a AABB)>; 3], axis: usize, pivot: usize) -> (&'a [Vec<(usize, &'a AABB)>; 3], &'a [Vec<(usize, &'a AABB)>; 3]) {
  //   partitioned.iter().unzip()
  // }

  // pub intersect(&self, ray: &Ray) -> Option(Intersection) {
  // }
}

struct BVHNode {
  aabb: AABB,
  children: Option<[Arc<BVHNode>; 2]>,
}

