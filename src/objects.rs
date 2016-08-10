extern crate rand;

use vector::Vector;
use ray::Ray;
use intersection::Intersection;
use shape::Shape;
use triangle::Triangle;
use sphere::Sphere;
use constant::*;

#[derive(Debug, Clone, Default)]
pub struct Objects {
  pub triangles: Vec<Triangle>,
  pub spheres: Vec<Sphere>,
  pub emission_area_total: f64,
  emission_triangles: Vec<Triangle>,
  emission_triangles_area: Vec<f64>,
}

impl Objects {
  pub fn new(triangles: Vec<Triangle>, spheres: Vec<Sphere>) -> Objects {
    let mut emission_triangles: Vec<Triangle> = Default::default();
    let mut emission_triangles_area: Vec<f64> = Default::default();
    let mut all_triangles: Vec<Triangle> = Default::default();
    let mut all_spheres: Vec<Sphere> = Default::default();
    let mut emission_triangles_area_total = 0.0;
    for v in triangles {
      if v.material.emission.sqr_len() > 0.0 {
        emission_triangles.push(v);
        let cr = (v.position1 - v.position0).cross(v.position2 - v.position0);
        let area = 0.5 * cr.len();
        emission_triangles_area.push(area);
        emission_triangles_area_total += area;
      }
      all_triangles.push(v);
    }
    for v in spheres {
      all_spheres.push(v);
    }
    Objects {
      triangles: all_triangles,
      spheres: all_spheres,
      emission_area_total: emission_triangles_area_total,
      emission_triangles: emission_triangles,
      emission_triangles_area: emission_triangles_area,
    }
  }

  pub fn get_intersect(&self, r: Ray) -> Intersection {
    let mut intersect: Intersection = Default::default();
    intersect.is_intersect = false;
    for obj in &self.triangles {
      let i = obj.intersect(r);
      if i.is_intersect && (!intersect.is_intersect || intersect.distance > i.distance) {
        intersect = i;
      }
    }
    for obj in &self.spheres {
      let i = obj.intersect(r);
      if i.is_intersect && (!intersect.is_intersect || intersect.distance > i.distance) {
        intersect = i;
      }
    }
    if intersect.is_intersect {
      // マイクロファセット分布関数
      if intersect.material.roughness != 0.0 {
        // (cosθ)^nの確率密度関数でサンプル(規格化を含む)
        let a = 5.0 / intersect.material.roughness.powf(1.7) - 4.0;
        let theta = (1.0 - rand::random::<f64>()).powf(1.0 / (a + 1.0)).acos();
        let phi = 2.0 * PI * rand::random::<f64>();
        // 反射点での法線方向を基準にした正規直交基底を生成
        let w = intersect.normal;
        let u = if w.x.abs() > EPS { Vector::new(0.0, 1.0, 0.0) } else { Vector::new(1.0, 0.0, 0.0) }.cross(w);
        let v = w.cross(u);
        // Normal Density Function
        intersect.normal = u * (theta.sin() * phi.cos()) + v * (theta.sin() * phi.sin()) + w * (theta.cos());
      }
    }
    return intersect;
  }

  pub fn get_emission_point(&self) -> Vector {
    let roulette = self.emission_area_total * rand::random::<f64>();
    let mut area = 0.0;
    let mut ret: Vector = Default::default();
    for (i, obj) in (self.emission_triangles).iter().enumerate() {
      area += (self.emission_triangles_area)[i];
      if roulette <= area {
        let mut s = rand::random::<f64>();
        let mut t = rand::random::<f64>();
        if s + t > 1.0 {
          s = 1.0 - s;
          t = 1.0 - t;
        }
        ret = Vector{
          x: (1.0 - s - t) * obj.position0.x + s * obj.position1.x + t * obj.position2.x,
          y: (1.0 - s - t) * obj.position0.y + s * obj.position1.y + t * obj.position2.y,
          z: (1.0 - s - t) * obj.position0.z + s * obj.position1.z + t * obj.position2.z,
        };
      }
    }
    return ret;
  }
}
