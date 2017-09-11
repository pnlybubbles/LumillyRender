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
    cam_pos - screen_dir + screen_dir * 0.3,
    // aperture position
    cam_pos + screen_dir * 0.3,
    // sensor size
    Vector2::new(10.0 * w / h, 10.0),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    0.5,
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
  let blue_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.25, 0.25, 0.75),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let red_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.75, 0.25, 0.25),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  // let mirror_mat = Arc::new(IdealRefractionMaterial {
  //   albedo: Vector3::new(0.99, 0.99, 0.99),
  //   emission: Vector3::new(0.0, 0.0, 0.0),
  //   ior: INF, // sin(0) = 1.0 / ior
  // });
  let rough_mat = Arc::new(PhongMaterial {
    reflectance: Vector3::new(0.75, 0.75, 0.75),
    roughness: 100.0,
  });
  let objects = Objects::new(vec![
    Arc::new(Sphere { radius: 2.0, position: Vector3::new(2.0, -3.0, 2.0), material: glass_mat.clone() }),
    Arc::new(Sphere { radius: 2.0, position: Vector3::new(2.0, -3.0, -2.0), material: blue_mat.clone() }),
    Arc::new(Sphere { radius: 2.0, position: Vector3::new(-2.0, -3.0, 2.0), material: red_mat.clone() }),
    Arc::new(Sphere { radius: 2.0, position: Vector3::new(-2.0, -3.0, -2.0), material: rough_mat.clone() }),
    Arc::new(Triangle::new(Vector3::new(-8.0, -5.0, -8.0), Vector3::new(-8.0, -5.0, 8.0), Vector3::new(8.0, -5.0, -8.0), white_mat.clone())),
    Arc::new(Triangle::new(Vector3::new(-8.0, -5.0, 8.0), Vector3::new(8.0, -5.0, 8.0), Vector3::new(8.0, -5.0, -8.0), white_mat.clone())),
  ]);
  let sky = Arc::new(IBLSky::new("ibl.hdr", 1500));
  Arc::new(Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: objects,
  })
}
