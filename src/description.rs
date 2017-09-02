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
use triangle::Triangle;
use objects::Objects;
use sky::*;

pub fn camera() -> Arc<Camera + Send + Sync> {
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let cam_pos = Vector3::new(-11.5, 1.0, 13.0);
  let screen_dir = Vector3::new(8.18, -2.0, -9.0);
  Arc::new(LensCamera::new(
    // sensor position
    cam_pos - screen_dir,
    // aperture position
    cam_pos,
    // sensor size
    Vector2::new(10.0 * w / h, 10.0),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    0.3,
    // focus_distance
    3.0 + screen_dir.len(),
  ))
}

pub fn scene() -> Arc<Scene> {
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.75, 0.75, 0.75),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let glass_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(0.99, 0.99, 0.99),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: 1.5,
  });
  let mirror_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(0.99, 0.99, 0.99),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: INF, // sin(0) = 1.0 / ior
  });
  let objects = Objects {
    objects: vec![
      Arc::new(Sphere { radius: 1.8, position: Vector3::new(-4.0, -3.2, 0.5), material: glass_mat.clone() }),
      Arc::new(Sphere { radius: 1.8, position: Vector3::new(0.8, -3.2, -0.5), material: white_mat.clone() }),
      Arc::new(Sphere { radius: 1.8, position: Vector3::new(2.0, -3.2, 4.0), material: mirror_mat.clone() }),
      Arc::new(Triangle::new(Vector3::new(-8.0, -5.0, -8.0), Vector3::new(-8.0, -5.0, 8.0), Vector3::new(8.0, -5.0, -8.0), white_mat.clone())),
      Arc::new(Triangle::new(Vector3::new(-8.0, -5.0, 8.0), Vector3::new(8.0, -5.0, 8.0), Vector3::new(8.0, -5.0, -8.0), white_mat.clone())),
    ],
  };
  let sky = Arc::new(IBLSky::new("ibl.hdr", 2500));
  Arc::new(Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: objects,
  })
}
