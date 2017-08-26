use std::sync::Arc;
use camera::*;
use constant::*;
use vector3::Vector3;
use vector2::Vector2;
use vector::*;
use img::Img;
use material::*;
use scene::Scene;
use sphere::Sphere;
use objects::Objects;
use sky::*;

pub fn camera() -> Arc<Camera + Send + Sync> {
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let cam_pos = Vector3::new(0.0, 2.0, -5.0);
  let screen_dir = (Vector3::new(0.0, 0.0, 0.0) - cam_pos).norm();
  Arc::new(LensCamera::new(
    // sensor position
    cam_pos - screen_dir,
    // aperture position
    cam_pos,
    // sensor size
    Vector2::new(2.0 * w / h, 2.0),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    EPS,
    // focus_distance
    3.0 + screen_dir.len(),
  ))
}

pub fn scene() -> Arc<Scene> {
  let red_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.75, 0.25, 0.25),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let blue_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(65.0 / 255.0, 105.0 / 255.0, 225.0 / 255.0),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.9, 0.9, 0.9),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let gray_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.5, 0.5, 0.5),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let light_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.0, 0.0, 0.0),
    emission: Vector3::new(36.0, 36.0, 36.0),
  });
  let glass_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(1.0, 1.0, 1.0),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: 1.5,
  });
  let mirror_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(1.0, 1.0, 1.0),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: INF, // sin(0) = 1.0 / ior
  });
  let spheres = vec![
    Sphere { radius: 1.0, position: Vector3::new(0.0, 0.0, 0.0), material: glass_mat.clone() },
    Sphere { radius: 1.0, position: Vector3::new(0.0, 0.0, -5.0), material: blue_mat.clone() },
    Sphere { radius: 1000.0, position: Vector3::new(0.0, -1001.0, 0.0), material: gray_mat.clone() },
    Sphere { radius: 1.0, position: Vector3::new(-3.0, 0.0, 0.0), material: white_mat.clone() },
    Sphere { radius: 1.0, position: Vector3::new(3.0, 0.0, 0.0), material: mirror_mat.clone() },
  ];
  let objects = Objects {
    objects: spheres,
  };
  let sky = Arc::new(IBLSky::new("ibl.hdr", 2500));
  Arc::new(Scene {
    depth: 10,
    depth_limit: 11,
    sky: sky,
    objects: objects,
  })
}
