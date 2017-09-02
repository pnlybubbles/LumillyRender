use ray::Ray;
use intersection::Intersection;
use shape::Shape;
use std::sync::Arc;

pub struct Objects {
  pub objects: Vec<Arc<Shape + Send + Sync>>,
}

impl Objects {
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
