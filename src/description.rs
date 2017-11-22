extern crate tobj;

use std::sync::Arc;
use camera::*;
use vector::*;
use material::*;
use constant::*;
use scene::Scene;
// use sphere::Sphere;
use shape::Shape;
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
    0.5e-5,
    // focus_distance
    pos.norm(),
  )
}

pub fn scene() -> Scene {
  let white_mat = Arc::new(LambertianMaterial {
    albedo: Vector::new(0.75, 0.75, 0.75),
    emission: Vector::zero(),
  });
  // let glass_mat = Arc::new(IdealRefractionMaterial {
  //   albedo: Vector::new(0.99, 0.99, 0.99),
  //   emission: Vector::zero(),
  //   ior: 1.5,
  // });
  // let blue_mat = Arc::new(LambertianMaterial {
  //   albedo: Vector::new(0.25, 0.25, 0.75),
  //   emission: Vector::zero(),
  // });
  // let red_mat = Arc::new(LambertianMaterial {
  //   albedo: Vector::new(0.75, 0.25, 0.25),
  //   emission: Vector::zero(),
  // });
  // let mirror_mat = Arc::new(IdealRefractionMaterial {
  //   albedo: Vector::new(0.99, 0.99, 0.99),
  //   emission: Vector::zero(),
  //   ior: INF, // sin(0) = 1.0 / ior
  // });
  // let rough_mat = Arc::new(PhongMaterial {
  //   reflectance: Vector::new(0.75, 0.75, 0.75),
  //   roughness: 10.0,
  // });
  let mut objects: Vec<Box<Shape + Send + Sync>> = vec![
    // box Sphere {
    //   radius: 2.0,
    //   position: Vector::new(2.5, -3.0, 2.5),
    //   material: glass_mat.clone(),
    // },
    // box Sphere {
    //   radius: 1.5,
    //   position: Vector::new(2.0, -3.5, -2.0),
    //   material: blue_mat.clone(),
    // },
    // box Sphere {
    //   radius: 1.0,
    //   position: Vector::new(-2.0, -4.0, 2.0),
    //   material: red_mat.clone(),
    // },
    // box Sphere {
    //   radius: 1.5,
    //   position: Vector::new(2.0, -0.5, -2.0),
    //   material: mirror_mat.clone(),
    // },
    // box Sphere {
    //   radius: 1.0,
    //   position: Vector::new(-2.0, -2.0, 2.0),
    //   material: glass_mat.clone(),
    // },
    // box Sphere {
    //   radius: 2.0,
    //   position: Vector::new(-2.5, -3.0, -2.5),
    //   material: rough_mat.clone(),
    // },
    // box Sphere { // center sphere
    //   radius: 1.0,
    //   position: Vector::new(0.0, -4.0, 0.0),
    //   material: white_mat.clone(),
    // },
    // box Triangle::new(
    //   Vector::new(-8.0, -5.0, -8.0),
    //   Vector::new(-8.0, -5.0, 8.0),
    //   Vector::new(8.0, -5.0, -8.0),
    //   white_mat.clone()
    // ),
    // box Triangle::new(
    //   Vector::new(-8.0, -5.0, 8.0),
    //   Vector::new(8.0, -5.0, 8.0),
    //   Vector::new(8.0, -5.0, -8.0),
    //   white_mat.clone()
    // ),
  ];
  // let mut objects: Vec<Box<Shape + Send + Sync>> = Vec::new();
  let obj = tobj::load_obj(&Path::new("models/simple/cornell_box.obj"));
  assert!(obj.is_ok());
  let (models, materials) = obj.unwrap();
  println!("# of models: {}", models.len());
  println!("# of materials: {}", materials.len());
  // let x_rotate = PI / 3.3;
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
        // polygon[i] = Vector::new(
        //   polygon[i].x,
        //   x_rotate.cos() * polygon[i].y - x_rotate.sin() * polygon[i].z,
        //   x_rotate.sin() * polygon[i].y + x_rotate.cos() * polygon[i].z,
        // )
      }
      // println!("{:?}", polygon);
      objects.push(box Triangle::new(polygon[0], polygon[1], polygon[2], mat.clone()));
    }
  }
  let sky = box IBLSky::new("ibl.hdr", 1500);
  Scene {
    depth: 5,
    depth_limit: 64,
    sky: sky,
    objects: Objects::new(objects),
  }
}
