extern crate rand;

use std::sync::Arc;
use ray::Ray;
use intersection::Intersection;
use shape::*;
use bvh::BVH;
use vector::*;
use sample::Sample;
use aabb::AABB;

pub struct Objects {
  bvh: BVH,
  // objects: Vec<Box<Shape + Send + Sync>>,
  emission: Vec<Arc<SurfaceShape + Send + Sync>>,
  emission_area: f64,
}

impl Objects {
  pub fn new(objects: Vec<Arc<SurfaceShape + Send + Sync>>) -> Objects {
    let emission = objects
      .iter()
      .filter( |v| v.material().emission().sqr_norm() > 0.0 )
      .cloned()
      .collect::<Vec<_>>();
    let emission_area = emission.iter().map( |v| v.area() ).sum();
    Objects {
      bvh: BVH::new(objects),
      emission: emission,
      emission_area: emission_area,
    }
  }

  // pub fn new(objects: Vec<Box<Shape + Send + Sync>>) -> Objects {
  //   Objects {
  //     objects: objects,
  //   }
  // }

  pub fn sample_emission(&self) -> Sample<Vector> {
    let roulette = self.emission_area * rand::random::<f64>();
    let mut area = 0.0;
    for obj in &self.emission {
      area += obj.area();
      if roulette <= area {
        let sample = obj.sample();
        return Sample {
          value: sample.value,
          pdf: sample.pdf * obj.area() / self.emission_area,
        };
      }
    }
    unreachable!();
  }

  pub fn has_emission(&self) -> bool {
    self.emission_area > 0.0
  }
}

impl Shape for Objects {
  fn aabb(&self) -> AABB {
    self.bvh.aabb()
  }

  fn intersect(&self, ray: &Ray) -> Option<Intersection> {
    self.bvh.intersect(&ray)
  }

  // pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
  //   self.objects.iter().flat_map(|v| v.intersect(&ray)).min_by(
  //     |a, b| {
  //       a.distance.partial_cmp(&b.distance).unwrap()
  //     },
  //   )
  // }
}
