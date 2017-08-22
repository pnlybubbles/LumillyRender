use vector3::Vector3;
use sphere::Sphere;
use ray::Ray;
use intersection::Intersection;

#[derive(Debug, Default, Clone)]
pub struct Objects {
  pub objects: Vec<Sphere>,
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
