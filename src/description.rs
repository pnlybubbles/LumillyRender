extern crate tobj;
extern crate time;
extern crate toml;

use std::sync::Arc;
use camera::*;
use math::vector::*;
use math::matrix::*;
use material::material::*;
use scene::Scene;
use shape::SurfaceShape;
use triangle::Triangle;
use sphere::Sphere;
use objects::Objects;
use std::path::Path;
use sky::*;
use std::fs::File;
use std::collections::HashMap;
use std::io::prelude::*;
use scene_loader::{Config, HasTransform};
use scene_loader::Mesh as CMesh;
use scene_loader::Material as CMaterial;
use scene_loader::Sky as CSky;
use scene_loader::Camera as CCamera;

pub struct Description {
  pub config: Config,
}

impl Description {
  pub fn new(path: &str) -> Description {
    let p = Path::new(path);
    if !p.is_file() { panic!(format!("File `{}` is not found.", path)) }
    let mut file = File::open(p).unwrap();
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str).unwrap();
    let config: Config = toml::from_str(toml_str.as_str()).unwrap();
    Description {
      config: config,
    }
  }

  pub fn camera(&self) -> Box<Camera + Send + Sync> {
    let width = self.config.film.resolution.0;
    let height = self.config.film.resolution.1;
    let matrix = self.config.camera.matrix();
    box match self.config.camera {
      CCamera::IdealPinhole { fov, .. } => IdealPinholeCamera::new(matrix, fov, [width, height]),
    }
  }

  pub fn scene(&self) -> Scene {
    let loader = Loader::new(&self.config);
    let sky = self.config.sky.as_ref().map( |v| match *v {
      CSky::Uniform { color } => box UniformSky {
        emission: color.into(),
      } as Box<Sky + Send + Sync>,
      CSky::Ibl { ref path } => box IBLSky::new(path, 0),
    } ).unwrap_or(box UniformSky {
      emission: Vector3::zero(),
    });
    println!("polygons: {}", loader.instances.len());
    let start_time = time::now();
    let objects = Objects::new(loader.instances);
    let end_time = time::now();
    println!(
      "bvh construction: {}s",
      (end_time - start_time).num_milliseconds() as f32 / 1000.0
    );
    Scene {
      depth: self.config.renderer.depth.unwrap_or(5),
      depth_limit: self.config.renderer.depth_limit.unwrap_or(64),
      sky: sky,
      objects: objects,
      no_direct_emitter: self.config.renderer.no_direct_emitter.unwrap_or(false),
    }
  }
}

struct Loader {
  instances: Vec<Arc<SurfaceShape + Send + Sync>>,
}

impl Loader {
  fn new(config: &Config) -> Loader {
    let mut instances = Vec::new();
    let obj = Self::load_obj(config.object().iter().map( |o| o.mesh ).collect());
    for o in config.object() {
      let transform = o.matrix();
      let emission = o.emission.unwrap_or(Vector3::zero());
      let material = o.material.map( |m| {
        match *m  {
          CMaterial::Lambert { albedo, .. } => {
            Arc::new(LambertianMaterial {
              albedo: albedo.into(),
              emission: emission,
            }) as Arc<Material + Send + Sync>
          },
          CMaterial::Phong { reflectance, alpha, .. } => {
            Arc::new(PhongMaterial {
              reflectance: reflectance.into(),
              roughness: alpha,
            })
          },
          CMaterial::BlinnPhong { reflectance, alpha, .. } => {
            Arc::new(BlinnPhongMaterial {
              reflectance: reflectance.into(),
              roughness: alpha,
            })
          },
          CMaterial::Ggx { reflectance, roughness, ior, .. } => {
            Arc::new(GGXMaterial {
              reflectance: reflectance.into(),
              roughness: roughness,
              ior: ior,
            })
          },
        }
      });
      match *o.mesh {
        CMesh::Obj { ref name, .. } => {
          let value = obj.get(name).unwrap();
          let mut m = Self::obj(&value.0, &value.1, &transform, material, emission);
          instances.append(&mut m);
        },
        CMesh::Sphere { ref radius, ref name } => {
          let position = transform * Vector3::zero();
          let mat = material.ok_or(format!("Material must be specified for object `{}`", name)).unwrap();
          let sphere = Sphere::new(position, *radius, mat);
          instances.push(Arc::new(sphere));
        },
      }
    }
    Loader {
      instances: instances,
    }
  }

  fn load_obj(mesh: Vec<&CMesh>) -> HashMap<String, (Vec<tobj::Model>, Vec<tobj::Material>)> {
    let mut obj = HashMap::new();
    for m in mesh {
      match *m {
        CMesh::Obj { ref name, ref path } => {
          let path = Path::new(&path);
          obj.insert(name.clone(), tobj::load_obj(&path).unwrap());
        },
        _ => {},
      }
    }
    obj
  }

  fn obj(models: &Vec<tobj::Model>, materials: &Vec<tobj::Material>, transform: &Matrix4, default_material: Option<Arc<Material + Sync + Send>>, emission: Vector3) -> Vec<Arc<SurfaceShape + Sync + Send>> {
    let material = materials.iter().map( |v|
      Arc::new(LambertianMaterial {
        emission: emission,
        albedo: v.diffuse[..].into(),
      }) as Arc<Material + Sync + Send>
    ).collect::<Vec<_>>();
    let mut instances: Vec<Arc<SurfaceShape + Sync + Send>> = Vec::with_capacity(
      models.iter().map( |m| m.mesh.indices.len() / 3).sum()
    );
    for m in models {
      let mat = match default_material.clone() {
        None => m.mesh.material_id
          .map( |v| material[v].clone() )
          .ok_or("Specified material is not found in mlt file.")
          .unwrap(),
        Some(v) => v,
      };
      for f in 0..m.mesh.indices.len() / 3 {
        let mut polygon = [Vector3::zero(); 3];
        for i in 0..3 {
          let index: usize = f * 3 + i;
          let potition = Vector3::new(
            m.mesh.positions[m.mesh.indices[index] as usize * 3],
            m.mesh.positions[m.mesh.indices[index] as usize * 3 + 1],
            m.mesh.positions[m.mesh.indices[index] as usize * 3 + 2],
          );
          polygon[i] = transform * potition;
        }
        instances.push(Arc::new(Triangle::new(polygon[0], polygon[1], polygon[2], mat.clone())));
      }
    }
    instances
  }
}
