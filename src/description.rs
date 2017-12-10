extern crate tobj;
extern crate time;

use std::sync::Arc;
use camera::*;
use vector::*;
use material::*;
// use constant::*;
use scene::Scene;
use sphere::Sphere;
use shape::SurfaceShape;
use triangle::Triangle;
use objects::Objects;
use std::path::Path;
use sky::*;

pub fn camera(width: usize, height: usize) -> Box<Camera + Send + Sync> {
  let w = width as f32;
  let h = height as f32;
  let pos = Vector::new(0.0, 75.0, -500.0);
  let dir = Vector::new(0.0, 0.0, 0.035);
  box IdealPinholeCamera::new(
    // sensor position
    pos - dir,
    // aperture position
    pos,
    // sensor size
    [0.025 * w / h, 0.025],
    // sensor resolution
    [width, height],
  )
}

pub fn scene() -> Scene {
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector::new(1.0, 1.0, 1.0),
    emission: Vector::zero(),
  });
  let light_mat = Arc::new(LambertianMaterial {
    albedo: Vector::zero(),
    emission: Vector::new(20.0, 20.0, 20.0),
  });
  let mut instances: Vec<Arc<SurfaceShape + Send + Sync>> = Vec::new();
  instances.push(Arc::new(Sphere::new(
    Vector::new(0.0, 100.0, 0.0),
    20.0,
    light_mat.clone(),
  )));
  let models = vec![
    Path::new("models/debug/quad.obj"),
    // Path::new("models/debug/light.obj"),
    // Path::new("models/simple/cbox.obj"),
    // Path::new("models/simple/cbox_smallbox.obj"),
    // Path::new("models/simple/cbox_largebox.obj"),
    // Path::new("models/simple/cbox_luminaire.obj"),
    // Path::new("models/happy_vrip/cbox_happy_vrip.obj"),
  ];
  for path in models {
    let obj = tobj::load_obj(path);
    assert!(obj.is_ok());
    let (models, materials) = obj.unwrap();
    let material = materials.iter().map( |v|
      Arc::new(LambertianMaterial {
        emission: Vector::from_index( |i| v.ambient[i] as f32 ),
        albedo: Vector::from_index( |i| v.diffuse[i] as f32 ),
      })
    ).collect::<Vec<_>>();
    for m in models {
      let mesh = &m.mesh;
      println!("{}: {} ploygon", m.name, mesh.indices.len() / 3);
      let mat = mesh.material_id.map( |v|
        material[v].clone()
      ).unwrap_or(white_mat.clone());
      for f in 0..mesh.indices.len() / 3 {
        let mut polygon = [Vector::zero(); 3];
        for i in 0..3 {
          let index: usize = f * 3 + i;
          polygon[i] = Vector::new(
            mesh.positions[mesh.indices[index] as usize * 3] as f32,
            mesh.positions[mesh.indices[index] as usize * 3 + 1] as f32,
            mesh.positions[mesh.indices[index] as usize * 3 + 2] as f32,
          );
        }
        instances.push(Arc::new(Triangle::new(polygon[0], polygon[1], polygon[2], mat.clone())));
      }
    }
  }
  // let sky = box IBLSky::new("ibl.hdr", 1500);
  let sky = box UniformSky { emission: Vector::zero() };
  let start_time = time::now();
  let objects = Objects::new(instances);
  let end_time = time::now();
  println!(
    "bvh construction: {}s",
    (end_time - start_time).num_milliseconds() as f32 / 1000.0
  );
  Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: objects,
  }
}
