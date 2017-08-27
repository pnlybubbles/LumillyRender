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
  Arc::new(LensCamera::new(
    // sensor position
    Vector3::new(0.0, 2.5, 9.0),
    // aperture position
    Vector3::new(0.0, 2.4, 8.0),
    // sensor size
    Vector2::new(1.1 * w / h, 1.1),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    0.05,
    // focus_distance
    7.0,
  ))
}

pub fn scene() -> Arc<Scene> {
  let red_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.75, 0.25, 0.25),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let blue_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.25, 0.25, 0.75),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.75, 0.75, 0.75),
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  let light_mat = Arc::new(LambertianMaterial {
    albedo: Vector3::new(0.0, 0.0, 0.0),
    emission: Vector3::new(36.0, 36.0, 36.0),
  });
  let glass_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(0.99, 0.99, 0.99),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: 1.5,
  });
  let rough_mat = Arc::new(CookTorranceMaterial {
    absorptance: Vector3::new(0.75, 0.75, 0.75),
    reflectance: Vector3::new(1.0, 1.0, 1.0),
    ior: 1.5,
    roughness: 0.5,
  });
  let mirror_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(0.99, 0.99, 0.99),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: INF, // sin(0) = 1.0 / ior
  });
  let spheres = vec![
    Sphere { radius: 1.9, position: Vector3::new(4.1, 1.0, 0.0), material: mirror_mat.clone() },
    Sphere { radius: 1.9, position: Vector3::new(0.0, 1.0, 0.0), material: rough_mat.clone() },
    Sphere { radius: 1.9, position: Vector3::new(-4.1, 1.0, 0.0), material: white_mat.clone() },
    Sphere { radius: 1e5, position: Vector3::new(0.0, -1e5 - 1.0, 0.0), material: white_mat.clone() },
  ];
  let objects = Objects {
    objects: spheres,
  };
  let sky = Arc::new(IBLSky::new("ibl.hdr", 2500));
  Arc::new(Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: objects,
  })
}
