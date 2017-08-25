use std::sync::Arc;
use camera::Camera;
use constant::*;
use vector::*;
use vector3::Vector3;
use vector2::Vector2;
use img::Img;
use material::*;
use scene::Scene;
use sphere::Sphere;
use objects::Objects;
use sky::*;

pub fn camera() -> Camera {
  let w = Img::width() as f64;
  let h = Img::height() as f64;
  let cam_pos = Vector3::new(50.0, 52.0, 220.0);
  let cam_dir = Vector3::new(0.0, -0.04, -1.0).norm();
  Camera::new(
    // sensor position
    cam_pos - cam_dir * 40.0,
    // aperture position
    cam_pos,
    // sensor size
    Vector2::new(30.0 * w / h, 30.0),
    // sensor resolution
    Vector2::new(Img::width(), Img::height()),
    // aperture radius
    1e-5,
  )
}

pub fn scene() -> Scene {
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
  let mirror_mat = Arc::new(IdealRefractionMaterial {
    albedo: Vector3::new(0.99, 0.99, 0.99),
    emission: Vector3::new(0.0, 0.0, 0.0),
    ior: INF, // sin(0) = 1.0 / ior
  });
  let spheres = vec![
    Sphere { radius: 1e5, position: Vector3::new(1e5+1.0, 40.8, 81.6),   material: red_mat.clone() },
    Sphere { radius: 1e5, position: Vector3::new(-1e5+99.0, 40.8, 81.6), material: blue_mat.clone() },
    Sphere { radius: 1e5, position: Vector3::new(50.0, 40.8, 1e5),       material: white_mat.clone() },
    Sphere { radius: 1e5, position: Vector3::new(50.0, 1e5, 81.6),       material: white_mat.clone() },
    Sphere { radius: 1e5, position: Vector3::new(50.0, -1e5+81.6, 81.6), material: white_mat.clone() },
    Sphere { radius: 16.5,position: Vector3::new(27.0, 16.5, 47.0),      material: mirror_mat.clone() },
    Sphere { radius: 16.5,position: Vector3::new(77.0, 16.5, 78.0),      material: glass_mat.clone() },
    Sphere { radius: 15.0,position: Vector3::new(50.0, 90.0, 81.6),      material: light_mat.clone() },
  ];
  let objects = Objects {
    objects: spheres,
  };
  let sky = Arc::new(UniformSky {
    emission: Vector3::new(0.0, 0.0, 0.0),
  });
  Scene {
    depth: 4,
    depth_limit: 64,
    sky: sky,
    objects: objects,
  }
}
