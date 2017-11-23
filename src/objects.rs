use std::sync::Arc;
use ray::Ray;
use intersection::Intersection;
use shape::*;
use bvh::BVH;
use vector::*;

pub struct Objects {
  bvh: BVH,
  // objects: Vec<Box<Shape + Send + Sync>>,
  emission: Vec<Arc<Shape + Send + Sync>>,
}

impl Objects {
  pub fn new<T>(objects: Vec<Box<T>>) -> Objects
    where T: Shape + Surface + Send + Sync
  {
    let emission = objects
      .iter()
      .filter( |v| v.emission().sqr_norm() > 0.0 )
      .map( |v| Arc::new(**v) )
      .collect::<Vec<_>>();
    Objects {
      bvh: BVH::new(objects),
      emission: emission,
    }
  }

  // pub fn new(objects: Vec<Box<Shape + Send + Sync>>) -> Objects {
  //   Objects {
  //     objects: objects,
  //   }
  // }

  pub fn get_intersect(&self, ray: &Ray) -> Option<Intersection> {
    self.bvh.intersect(&ray)
  }

  // pub fn get_intersect(&self, ray: &Ray) -> Option<Intersection> {
  //   self.objects.iter().flat_map(|v| v.intersect(&ray)).min_by(
  //     |a, b| {
  //       a.distance.partial_cmp(&b.distance).unwrap()
  //     },
  //   )
  // }
}
