extern crate tobj;
extern crate time;

use std::sync::Arc;
use camera::*;
use math::vector::*;
use material::*;
use scene::Scene;
use shape::SurfaceShape;
use triangle::Triangle;
use sphere::Sphere;
use objects::Objects;
use std::path::Path;
use sky::*;

pub fn camera(width: usize, height: usize) -> Box<Camera + Send + Sync> {
  let w = width as f32;
  let h = height as f32;
  let pos = Vector3::new(278.0, 273.0, -800.0);
  let dir = Vector3::new(0.0, 0.0, 0.035);
  // let pos = Vector3::new(0.0, 75.0, -500.0);
  // let dir = Vector3::new(0.0, 0.0, 0.035);
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
    albedo: Vector3::new(1.0, 1.0, 1.0),
    emission: Vector3::zero(),
  });
  let models = vec![
    // Path::new("models/simple/cbox.obj"),
    // Path::new("models/simple/cbox_luminaire.obj"),
    Path::new("models/simple/cbox_floor.obj"),
    // Path::new("models/simple/cbox_largebox.obj"),
    // Path::new("models/simple/cbox_smallbox.obj"),
  ];
  let mut instances = models.iter().flat_map( |path|
    obj(path, white_mat.clone())
  ).collect::<Vec<_>>();
  instances.push(Arc::new(Sphere::new(
    Vector3::new(275.0, 150.0, 275.0),
    150.0,
    Arc::new(PhongMaterial {
      reflectance: Vector3::new(1.0, 1.0, 1.0),
      roughness: 100.0,
    }),
  )));
  let sky = box IBLSky::new("ibl.hdr", 1500);
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
    no_direct_emitter: true,
  }
}

fn obj(path: &Path, default_material: Arc<Material + Sync + Send>) -> Vec<Arc<SurfaceShape + Sync + Send>> {
  let (models, materials) = tobj::load_obj(&path).unwrap();
  let material = materials.iter().map( |v|
    if v.ambient.iter().sum::<f32>() > 0.0 {
      Arc::new(LambertianMaterial {
        emission: v.ambient[..].into(),
        albedo: v.diffuse[..].into(),
      }) as Arc<Material + Sync + Send>
    } else {
      Arc::new(PhongMaterial {
        reflectance: v.diffuse[..].into(),
        roughness: 100.0,
      })
    }
  ).collect::<Vec<_>>();
  let mut instances: Vec<Arc<SurfaceShape + Sync + Send>> = Vec::with_capacity(
    models.iter().map( |m| m.mesh.indices.len() / 3).sum()
  );
  for m in models {
    println!("{}: {} ploygon", m.name, m.mesh.indices.len() / 3);
    let mat = m.mesh.material_id.map( |v|
      material[v].clone()
    ).unwrap_or(default_material.clone());
    for f in 0..m.mesh.indices.len() / 3 {
      let mut polygon = [Vector3::zero(); 3];
      for i in 0..3 {
        let index: usize = f * 3 + i;
        polygon[i] = Vector3::new(
          m.mesh.positions[m.mesh.indices[index] as usize * 3],
          m.mesh.positions[m.mesh.indices[index] as usize * 3 + 1],
          m.mesh.positions[m.mesh.indices[index] as usize * 3 + 2],
        );
      }
      instances.push(Arc::new(Triangle::new(polygon[0], polygon[1], polygon[2], mat.clone())));
    }
  }
  instances
}

