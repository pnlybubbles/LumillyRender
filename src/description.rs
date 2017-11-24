extern crate tobj;

use std::sync::Arc;
use camera::*;
use vector::*;
use material::*;
// use constant::*;
use scene::Scene;
// use sphere::Sphere;
use shape::SurfaceShape;
use triangle::Triangle;
use objects::Objects;
use std::path::Path;
use sky::*;

pub fn camera(width: usize, height: usize) -> Box<Camera + Send + Sync> {
  let w = width as f64;
  let h = height as f64;
  let pos = Vector::new(278.0, 273.0, -800.0);
  let dir = Vector::new(0.0, 0.0, 0.035);
  box LensCamera::new(
    // sensor position
    pos - dir,
    // aperture position
    pos,
    // sensor size
    [0.025 * w / h, 0.025],
    // sensor resolution
    [width, height],
    // aperture radius
    1e-8,
    // focus_distance
    pos.norm(),
  )
}

pub fn scene() -> Scene {
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector::new(0.75, 0.75, 0.75),
    emission: Vector::zero(),
  });
  let mut objects: Vec<Arc<SurfaceShape + Send + Sync>> = Vec::new();
  let models = vec![
    Path::new("models/simple/cornell_box.obj"),
    // Path::new("models/bunny/cornell_box-bunny.obj"),
  ];
  for path in models {
    let obj = tobj::load_obj(path);
    assert!(obj.is_ok());
    let (models, materials) = obj.unwrap();
    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    let material = materials.iter().map( |v|
      Arc::new(LambertianMaterial {
        emission: Vector::from_index( |i| v.ambient[i] as f64 ),
        albedo: Vector::from_index( |i| v.diffuse[i] as f64 ),
      })
    ).collect::<Vec<_>>();
    for (i, m) in models.iter().enumerate() {
      let mesh = &m.mesh;
      println!("model[{}].name = \'{}\'", i, m.name);
      println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);
      println!("Size of model[{}].indices: {}", i, mesh.indices.len());
      println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
      let mat = mesh.material_id.map( |v|
        material[v].clone()
      ).unwrap_or(white_mat.clone());
      for f in 0..mesh.indices.len() / 3 {
        let mut polygon = [Vector::zero(); 3];
        for i in 0..3 {
          let index: usize = f * 3 + i;
          polygon[i] = Vector::new(
            mesh.positions[mesh.indices[index] as usize * 3] as f64,
            mesh.positions[mesh.indices[index] as usize * 3 + 1] as f64,
            mesh.positions[mesh.indices[index] as usize * 3 + 2] as f64,
          );
        }
        objects.push(Arc::new(Triangle::new(polygon[0], polygon[1], polygon[2], mat.clone())));
      }
    }
  }
  // let sky = box IBLSky::new("ibl.hdr", 1500);
  let sky = box UniformSky { emission: Vector::zero() };
  Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: Objects::new(objects),
  }
}
