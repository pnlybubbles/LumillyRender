use std::sync::Arc;
use camera::*;
use vector::*;
use material::*;
use constant::*;
use scene::Scene;
use sphere::Sphere;
use triangle::Triangle;
use objects::Objects;
use sky::*;

pub fn camera(width: usize, height: usize) -> Box<Camera + Send + Sync> {
  let w = width as f64;
  let h = height as f64;
  let cam_pos = Vector::new(-10.5, 1.0, 13.5);
  let screen_dir = Vector::new(7.18, -2.0, -9.0);
  box LensCamera::new(
    // sensor position
    cam_pos - screen_dir + screen_dir * 0.3,
    // aperture position
    cam_pos + screen_dir * 0.3,
    // sensor size
    [10.0 * w / h, 10.0],
    // sensor resolution
    [width, height],
    // aperture radius
    0.5,
    // focus_distance
    (cam_pos + screen_dir * 0.3).norm(),
  )
}

pub fn scene() -> Scene {
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector::new(0.75, 0.75, 0.75),
    emission: Vector::zero(),
  });
  let glass_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector::new(0.99, 0.99, 0.99),
    emission: Vector::zero(),
    ior: 1.5,
  });
  let blue_mat = Arc::new(LambertianMaterial {
    albedo: Vector::new(0.25, 0.25, 0.75),
    emission: Vector::zero(),
  });
  let red_mat = Arc::new(LambertianMaterial {
    albedo: Vector::new(0.75, 0.25, 0.25),
    emission: Vector::zero(),
  });
  let mirror_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector::new(0.99, 0.99, 0.99),
    emission: Vector::zero(),
    ior: INF, // sin(0) = 1.0 / ior
  });
  let rough_mat = Arc::new(PhongMaterial {
    reflectance: Vector::new(0.75, 0.75, 0.75),
    roughness: 10.0,
  });
  let objects = Objects::new(vec![
    box Sphere {
      radius: 2.0,
      position: Vector::new(2.5, -3.0, 2.5),
      material: glass_mat.clone(),
    },
    box Sphere {
      radius: 1.5,
      position: Vector::new(2.0, -3.5, -2.0),
      material: blue_mat.clone(),
    },
    box Sphere {
      radius: 1.0,
      position: Vector::new(-2.0, -4.0, 2.0),
      material: red_mat.clone(),
    },
    box Sphere {
      radius: 1.5,
      position: Vector::new(2.0, -0.5, -2.0),
      material: mirror_mat.clone(),
    },
    box Sphere {
      radius: 1.0,
      position: Vector::new(-2.0, -2.0, 2.0),
      material: glass_mat.clone(),
    },
    box Sphere {
      radius: 2.0,
      position: Vector::new(-2.5, -3.0, -2.5),
      material: rough_mat.clone(),
    },
    box Triangle::new(
      Vector::new(-8.0, -5.0, -8.0),
      Vector::new(-8.0, -5.0, 8.0),
      Vector::new(8.0, -5.0, -8.0),
      white_mat.clone()
    ),
    box Triangle::new(
      Vector::new(-8.0, -5.0, 8.0),
      Vector::new(8.0, -5.0, 8.0),
      Vector::new(8.0, -5.0, -8.0),
      white_mat.clone()
    ),
  ]);
  let sky = box IBLSky::new("ibl.hdr", 1500);
  Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: objects,
  }
}
