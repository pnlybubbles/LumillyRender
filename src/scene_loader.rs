use math::matrix::*;
use math::vector::*;
use constant::PI;

type Name = String;
type Vec3 = (f32, f32, f32);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Renderer {
  pub samples: usize,
  pub depth: Option<usize>,
  pub depth_limit: Option<usize>,
  pub no_direct_emitter: Option<bool>,
  pub threads: Option<usize>,
  pub integrator: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Film {
  pub resolution: (usize, usize),
  pub output: String,
  pub gamma: Option<f32>,
  pub sensitivity: Option<Vec3>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Sky {
  Uniform {
    color: Vec3,
  },
  Ibl {
    path: String,
  },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum Light {
  Area {
    object: Name,
    emission: Vec3,
  },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Object {
  name: Option<Name>,
  mesh: Name,
  material: Option<Name>,
  #[serde(default)]
  transform: Vec<Transform>,
}

impl HasTransform for Object {
  fn transform(&self) -> &Vec<Transform> {
    &self.transform
  }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Transform {
  Translate {
    vector: Vec3,
  },
  Scale {
    vector: Vec3,
  },
  AxisAngle {
    axis: Vec3,
    angle: f32,
  },
  LookAt {
    origin: Vec3,
    target: Vec3,
    up: Vec3,
  },
}

impl Transform {
  pub fn matrix(&self) -> Matrix4 {
    match *self {
      Transform::Translate { vector } => Matrix4::translate(vector.into()),
      Transform::Scale { vector } => Matrix4::scale(vector.into()),
      Transform::AxisAngle { axis, angle } => Matrix4::axis_angle(axis.into(), angle * PI / 180.0),
      Transform::LookAt { origin, target, up } => Matrix4::look_at(origin.into(), target.into(), up.into()),
    } 
  }
}

pub trait HasTransform {
  fn transform(&self) -> &Vec<Transform>;
  fn matrix(&self) -> Matrix4 {
    self.transform().iter().map( |t| t.matrix() ).fold(Matrix4::unit(), |p, c| c * p )
  }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Camera {
  IdealPinhole {
    fov: f32,
    #[serde(default)]
    transform: Vec<Transform>,
  },
  ThinLens {
    fov: f32,
    focus_distance: f32,
    f_number: f32,
    #[serde(default)]
    transform: Vec<Transform>,
  }
}

impl HasTransform for Camera {
  fn transform(&self) -> &Vec<Transform> {
    match *self {
      Camera::IdealPinhole { ref transform, .. } => transform,
      Camera::ThinLens { ref transform, .. } => transform,
    }
  }
}

pub trait HasName {
  fn name(&self) -> Name;
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Material {
  Lambert {
    name: Name,
    albedo: Vec3,
  },
  Phong {
    name: Name,
    reflectance: Vec3,
    alpha: f32,
  },
  BlinnPhong {
    name: Name,
    reflectance: Vec3,
    alpha: f32,
  },
  Ggx {
    name: Name,
    reflectance: Vec3,
    roughness: f32,
    ior: f32,
  },
}

impl HasName for Material {
  fn name(&self) -> Name {
    match *self {
      Material::Lambert { ref name, .. } => name.clone(),
      Material::Phong { ref name, .. } => name.clone(),
      Material::BlinnPhong { ref name, ..} => name.clone(),
      Material::Ggx { ref name, ..} => name.clone(),
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Mesh {
  Obj {
    name: Name,
    path: String,
  },
  Sphere {
    name: Name,
    radius: f32,
  },
}

impl HasName for Mesh {
  fn name(&self) -> Name {
    match *self {
      Mesh::Obj { ref name, .. } => name.clone(),
      Mesh::Sphere { ref name, .. } => name.clone(),
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
  pub renderer: Renderer,
  pub sky: Option<Sky>,
  pub film: Film,
  pub camera: Camera,
  #[serde(default)]
  light: Vec<Light>,
  #[serde(default)]
  object: Vec<Object>,
  #[serde(default)]
  material: Vec<Material>,
  #[serde(default)]
  mesh: Vec<Mesh>,
}

pub struct ObjectDescriptor<'a> {
  pub mesh: &'a Mesh,
  pub material: Option<&'a Material>,
  pub transform: &'a Vec<Transform>,
  pub emission: Option<Vector3>,
}

impl<'a> HasTransform for ObjectDescriptor<'a> {
  fn transform(&self) -> &Vec<Transform> {
    &self.transform
  }
}

impl Config {
  fn find_mesh_by_name(&self, name: &str) -> Result<&Mesh, String> {
    let mesh = self.mesh.iter().find( |m| m.name() == name );
    mesh.ok_or(format!("Mesh named `{}` is not found.", name))
  }

  fn find_material_by_name(&self, name: &str) -> Result<&Material, String> {
    let material = self.material.iter().find( |m| m.name() == name );
    material.ok_or(format!("Material named `{}` is not found.", name))
  }

  pub fn object(&self) -> Vec<ObjectDescriptor> {
    self.object.iter().map( |v| {
      let mesh = self.find_mesh_by_name(&v.mesh).unwrap();
      let material = v.material.as_ref().map( |name|
        self.find_material_by_name(name).unwrap()
      );
      let emission = self.light.iter().find( |l| match **l {
        Light::Area { ref object, .. } => {
          v.name.as_ref().map( |name| name.as_str() == object ).unwrap_or(false)
        },
      } ).map( |l| match *l {
        Light::Area { ref emission, .. } => (*emission).into(),
      });
      ObjectDescriptor {
        mesh: &mesh,
        material: material,
        transform: &v.transform,
        emission: emission,
      }
    }).collect()
  }
}
