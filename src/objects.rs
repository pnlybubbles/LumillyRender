use ray::Ray;
use intersection::Intersection;
use shape::Shape;
use bvh::BVH;

pub struct Objects {
  bvh: BVH,
  // objects: Vec<Box<Shape + Send + Sync>>,
}

impl Objects {
  pub fn new(objects: Vec<Box<Shape + Send + Sync>>) -> Objects {
    Objects {
      bvh: BVH::new(objects),
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
