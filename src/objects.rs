use ray::Ray;
use intersection::Intersection;
use shape::Shape;
use std::sync::Arc;
// use bvh::BVH;

pub struct Objects {
  pub objects: Vec<Arc<Shape + Send + Sync>>,
  // pub bvh: BVH,
}

impl Objects {
  pub fn new(objects: Vec<Arc<Shape + Send + Sync>>) -> Objects {
    // let aabb = (&objects)
    //   .into_iter()
    //   .map(|v| { v.aabb() })
    //   .collect();
    // let bvh = BVH::new(aabb);
    Objects {
      objects: objects,
      // bvh: bvh,
    }
  }

  pub fn get_intersect(&self, r: &Ray) -> Option<Intersection> {
    self.objects.iter().fold(None, |maybe_intersect, obj| {
      match obj.intersect(r) {
        None => maybe_intersect,
        Some(i) => {
          Some(match maybe_intersect {
            None => i,
            Some(i_) => if i.distance < i_.distance { i } else { i_ },
          })
        }
      }
    })
  }
}
